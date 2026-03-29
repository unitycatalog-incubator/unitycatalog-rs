use std::sync::{Arc, LazyLock};

use clap::Parser;
use unitycatalog_postgres::GraphStore;
use unitycatalog_server::api::RequestContext;
use unitycatalog_server::memory::InMemoryResourceStore;
use unitycatalog_server::policy::ConstantPolicy;
use unitycatalog_server::{rest::AnonymousAuthenticator, services::ServerHandler};

use crate::config::{Backend, Config, PostgresBackendConfig, SecretBackend};
use crate::error::{Error, Result};

mod run;

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

    let handler = match &config.backend {
        Backend::InMemory => get_memory_handler().await?,
        Backend::Postgres(pg) => get_db_handler(pg, &config.secret_backend).await?,
    };

    run::run_server_rest(host, port, handler, AnonymousAuthenticator)
        .await
        .map_err(|_| Error::Generic("Server failed".to_string()))
}

async fn handle_grpc(_args: &ServerArgs) -> Result<()> {
    unimplemented!()
}

async fn get_db_handler(
    pg: &PostgresBackendConfig,
    secret_backend: &Option<SecretBackend>,
) -> Result<ServerHandler<RequestContext>> {
    let db_url = pg
        .connection_string()
        .ok_or_else(|| Error::Generic("incomplete postgres backend configuration".into()))?;

    let encryption_key = match secret_backend {
        Some(SecretBackend::Postgres(cfg)) => cfg.encryption_key.value(),
        _ => None,
    };

    let store = Arc::new(
        GraphStore::connect(&db_url, encryption_key)
            .await
            .map_err(|e| Error::Generic(format!("connecting to database: {e}")))?,
    );
    let policy = Arc::new(ConstantPolicy::default());
    store
        .migrate()
        .await
        .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
    let handler = ServerHandler::try_new_tokio(policy, store.clone(), store)?;
    Ok(handler)
}

async fn get_memory_handler() -> Result<ServerHandler<RequestContext>> {
    let store = Arc::new(InMemoryResourceStore::new());
    let policy = Arc::new(ConstantPolicy::default());
    let handler = ServerHandler::try_new_tokio(policy, store.clone(), store)?;
    Ok(handler)
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
