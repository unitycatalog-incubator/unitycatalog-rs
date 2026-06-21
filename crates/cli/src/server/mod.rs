use std::io::IsTerminal;
use std::sync::{Arc, LazyLock};

use clap::Parser;
use comfy_table::{Cell, ContentArrangement, Table, presets::UTF8_FULL};
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_postgres::GraphStore;
use unitycatalog_server::api::RequestContext;
use unitycatalog_server::memory::InMemoryResourceStore;
use unitycatalog_server::policy::{ConstantPolicy, Policy};
use unitycatalog_server::{
    rest::AnonymousAuthenticator,
    services::{LocalStoragePolicy, ServerHandler, location::StorageLocationUrl},
};

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

    #[clap(long, help = "suppress the startup banner and summary")]
    quiet: bool,
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

    // The ASCII banner is decorative; only show it on an interactive terminal
    // and when not silenced, so piped/redirected output stays clean.
    if !args.quiet && std::io::stdout().is_terminal() {
        println!("{}", WELCOME.as_str());
    }

    let config = load_config(&args.config)?;

    let host = args
        .host
        .as_deref()
        .or(config.host.as_deref())
        .unwrap_or(DEFAULT_HOST);
    let port = args.port.or(config.port).unwrap_or(DEFAULT_PORT);

    if !args.quiet {
        print_startup_summary(host, port, &config);
    }

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

    // Build the local-storage allowlist from config. Empty ⇒ deny all file://.
    // A configured root that does not exist is a hard startup error.
    let local_storage_policy = LocalStoragePolicy::new(&config.local_storage.allowed_roots)
        .map_err(|e| Error::Generic(format!("invalid local_storage config: {e}")))?;

    // A configured metastore managed storage root must parse and, if it is a
    // local (file://) path, sit within an allowed local root — same governance
    // as catalog/schema roots. Validate at startup so a misconfigured root is a
    // hard error rather than surfacing later at catalog-create time.
    if let Some(root) = config
        .managed_storage_root
        .as_deref()
        .filter(|s| !s.is_empty())
    {
        let url = StorageLocationUrl::parse(root)
            .map_err(|e| Error::Generic(format!("invalid managed_storage_root '{root}': {e}")))?;
        local_storage_policy
            .check(&url)
            .map_err(|e| Error::Generic(format!("invalid managed_storage_root '{root}': {e}")))?;
    }

    let (handler, policy) = match &config.backend {
        Backend::InMemory => get_memory_handler(encryptor).await?,
        Backend::Postgres(pg) => get_db_handler(pg, encryptor).await?,
    };
    let handler = handler
        .with_local_storage_policy(local_storage_policy)
        .with_managed_storage_root(config.managed_storage_root.clone());

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

/// Print a concise, human-readable summary of how the server is configured,
/// just before it starts listening. The actual "listening on …" line is
/// emitted from [`run::run`] once the socket is bound (so it reflects the real
/// address even when `port = 0`).
fn print_startup_summary(host: &str, port: u16, config: &Config) {
    let base = format!("http://{host}:{port}");

    let backend = match &config.backend {
        Backend::InMemory => "in-memory".to_string(),
        Backend::Postgres(_) => "postgres".to_string(),
    };

    let routing = if config.routing.any_upstream() {
        format!(
            "hybrid (upstream: {})",
            config.routing.upstream_surfaces().join(", ")
        )
    } else {
        "local".to_string()
    };

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    for (key, value) in [
        ("Bind address", base.clone()),
        ("Backend", backend),
        ("Routing", routing),
        ("Unity Catalog API", format!("{base}/api/2.1/unity-catalog")),
        ("Delta Sharing API", format!("{base}/api/v1/delta-sharing")),
        ("Open Sharing API", format!("{base}/api/v1/open-sharing")),
        ("Swagger UI", format!("{base}/api/2.1/unity-catalog/")),
    ] {
        table.add_row(vec![
            Cell::new(key).add_attribute(comfy_table::Attribute::Bold),
            Cell::new(value),
        ]);
    }
    println!("{table}");
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
    // The Postgres store also implements `CommitCoordinator`, so Delta
    // catalog-managed commits are persisted in the database rather than memory.
    let handler = ServerHandler::try_new_tokio_with_coordinator(
        policy.clone(),
        store.clone(),
        store.clone(),
        store,
    )?;
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
