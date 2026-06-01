use std::sync::{Arc, LazyLock};

use clap::Parser;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_postgres::GraphStore;
use unitycatalog_server::api::RequestContext;
use unitycatalog_server::memory::InMemoryResourceStore;
use unitycatalog_server::policy::{ConstantPolicy, Policy};
use unitycatalog_server::{rest::AnonymousAuthenticator, services::ServerHandler};

use crate::config::{Backend, Config, PostgresBackendConfig};
use crate::error::{Error, Result};
use unitycatalog_common::services::encryption::EnvelopeEncryptor;

mod hybrid;
mod run;

/// A local server handler paired with the policy it was built with.
///
/// The policy is surfaced separately so the hybrid proxy can apply the *same*
/// authorization to surfaces it forwards upstream.
type LocalHandler = (
    ServerHandler<RequestContext>,
    Arc<dyn Policy<RequestContext>>,
);

const DEFAULT_HOST: &str = "0.0.0.0";
const DEFAULT_PORT: u16 = 8080;

#[derive(Debug, Parser)]
pub struct ServerArgs {
    #[clap(long)]
    host: Option<String>,

    #[clap(long, short)]
    port: Option<u16>,

    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    #[clap(long, help = "expose rest API", default_value_t = true)]
    rest: bool,

    #[clap(long, help = "expose rest gRPC", default_value_t = false)]
    grpc: bool,
}

fn load_config(path: &str) -> Result<Config> {
    let path = std::path::Path::new(path);
    if !path.exists() {
        tracing::info!(
            "config file not found at {}, using defaults",
            path.display()
        );
        return Ok(Config::default());
    }
    let contents = std::fs::read_to_string(path)
        .map_err(|e| Error::Generic(format!("reading config: {e}")))?;
    serde_yml::from_str(&contents).map_err(|e| Error::Generic(format!("parsing config: {e}")))
}

pub async fn handle_server(args: &ServerArgs) -> Result<()> {
    if args.rest {
        handle_rest(args).await
    } else if args.grpc {
        handle_grpc(args).await
    } else {
        Err(Error::Generic("No server protocol specified".to_string()))
    }
}

/// Handle the rest server command.
///
/// This function starts a delta-sharing server using the REST protocol.
async fn handle_rest(args: &ServerArgs) -> Result<()> {
    unitycatalog_server::telemetry::init_tracing();

    println!("{}", WELCOME.as_str());

    let config = load_config(&args.config)?;

    let host = args
        .host
        .as_deref()
        .or(config.host.as_deref())
        .unwrap_or(DEFAULT_HOST);
    let port = args.port.or(config.port).unwrap_or(DEFAULT_PORT);

    let encryptor = config
        .encryption
        .as_ref()
        .ok_or_else(|| {
            Error::Generic(
                "missing `encryption` configuration: an active KEK is required to store secrets"
                    .into(),
            )
        })?
        .build_encryptor()
        .map_err(Error::Generic)?;

    let (handler, policy) = match &config.backend {
        Backend::InMemory => get_memory_handler(encryptor).await?,
        Backend::Postgres(pg) => get_db_handler(pg, encryptor).await?,
    };

    if config.routing.any_upstream() {
        let unsupported = config.routing.unsupported_upstream();
        if !unsupported.is_empty() {
            return Err(Error::Generic(format!(
                "upstream routing is not yet implemented for: {}",
                unsupported.join(", ")
            )));
        }
        let upstream = config.upstream.as_ref().ok_or_else(|| {
            Error::Generic(
                "routing marks surfaces as upstream but no `upstream` config is set".to_string(),
            )
        })?;
        let upstream_url = upstream
            .url
            .parse()
            .map_err(|e| Error::Generic(format!("invalid upstream url: {e}")))?;
        // Upstream needs no auth; authorization is enforced locally via `policy`.
        let client = UnityCatalogClient::new_unauthenticated(upstream_url);

        hybrid::run_server_rest_hybrid(
            host,
            port,
            handler,
            policy,
            client,
            &config.routing,
            AnonymousAuthenticator,
        )
        .await
        .map_err(|_| Error::Generic("Server failed".to_string()))
    } else {
        run::run_server_rest(host, port, handler, AnonymousAuthenticator)
            .await
            .map_err(|_| Error::Generic("Server failed".to_string()))
    }
}

async fn handle_grpc(_args: &ServerArgs) -> Result<()> {
    unimplemented!()
}

async fn get_db_handler(
    pg: &PostgresBackendConfig,
    encryptor: EnvelopeEncryptor,
) -> Result<LocalHandler> {
    let db_url = pg
        .connection_string()
        .ok_or_else(|| Error::Generic("incomplete postgres backend configuration".into()))?;

    let store = Arc::new(
        GraphStore::connect(&db_url, encryptor)
            .await
            .map_err(|e| Error::Generic(format!("connecting to database: {e}")))?,
    );
    let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
    store
        .migrate()
        .await
        .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
    let handler = ServerHandler::try_new_tokio(policy.clone(), store.clone(), store)?;
    Ok((handler, policy))
}

async fn get_memory_handler(encryptor: EnvelopeEncryptor) -> Result<LocalHandler> {
    let store = Arc::new(InMemoryResourceStore::new(encryptor));
    let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
    let handler = ServerHandler::try_new_tokio(policy.clone(), store.clone(), store)?;
    Ok((handler, policy))
}

static WELCOME: LazyLock<String> = LazyLock::new(|| {
    format!(
        r#"
                     _ _                   _        _
         _   _ _ __ (_) |_ _   _  ___ __ _| |_ __ _| | ___   __ _       _ __ ___  v{}
        | | | | '_ \| | __| | | |/ __/ _` | __/ _` | |/ _ \ / _` |_____| '__/ __|
        | |_| | | | | | |_| |_| | (_| (_| | || (_| | | (_) | (_| |_____| |  \__ \
         \__,_|_| |_|_|\__|\__, |\___\__,_|\__\__,_|_|\___/ \__, |     |_|  |___/
                           |___/                            |___/
        "#,
        env!("CARGO_PKG_VERSION")
    )
});
