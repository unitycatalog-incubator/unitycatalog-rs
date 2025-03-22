use std::sync::{Arc, LazyLock};

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use unitycatalog_common::kernel::KernelQueryHandler;
use unitycatalog_common::services::{ConstantPolicy, ServerHandler};
use unitycatalog_common::{memory::InMemoryResourceStore, rest::AnonymousAuthenticator};
use unitycatalog_postgres::GraphStore;

use crate::error::{Error, Result};

mod run;

#[derive(Debug, Parser)]
pub struct ServerArgs {
    #[clap(long, default_value = "0.0.0.0")]
    host: String,

    #[clap(long, short, default_value_t = 8080)]
    port: u16,

    #[arg(short, long, default_value = "config.yaml")]
    config: String,

    #[clap(long, help = "use database", default_value_t = false)]
    use_db: bool,

    #[clap(long, help = "expose rest API", default_value_t = true)]
    rest: bool,

    #[clap(long, help = "expose rest gRPC", default_value_t = false)]
    grpc: bool,
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
    init_tracing();

    println!("{}", WELCOME.as_str());

    if args.use_db {
        let handler = get_db_handler().await?;
        run::run_server_rest(
            args.host.clone(),
            args.port,
            handler,
            AnonymousAuthenticator,
        )
        .await
        .map_err(|_| Error::Generic("Server failed".to_string()))
    } else {
        let handler = get_memory_handler().await;
        run::run_server_rest(
            args.host.clone(),
            args.port,
            handler,
            AnonymousAuthenticator,
        )
        .await
        .map_err(|_| Error::Generic("Server failed".to_string()))
    }
}

async fn handle_grpc(_args: &ServerArgs) -> Result<()> {
    unimplemented!()
}

async fn get_db_handler() -> Result<ServerHandler> {
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| Error::Generic("missing DATABASE_URL".to_string()))?;
    let store = Arc::new(GraphStore::connect(&db_url).await.unwrap());
    let policy = Arc::new(ConstantPolicy::default());
    store.migrate().await.unwrap();
    let handler = ServerHandler {
        query: KernelQueryHandler::new_tokio_multi_threaded(
            store.clone(),
            Default::default(),
            policy.clone(),
        ),
        secrets: store.clone(),
        store,
        policy,
    };
    Ok(handler)
}

async fn get_memory_handler() -> ServerHandler {
    let store = Arc::new(InMemoryResourceStore::new());
    let policy = Arc::new(ConstantPolicy::default());
    ServerHandler {
        secrets: store.clone(),
        query: KernelQueryHandler::new_tokio_multi_threaded(
            store.clone(),
            Default::default(),
            policy.clone(),
        ),
        store,
        policy,
    }
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
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
