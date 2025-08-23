use clap::{Args, Parser, Subcommand};

use crate::client::{ClientCommand, handle_client};
use crate::error::Result;
use crate::server::{ServerArgs, handle_server};

mod client;
mod config;
mod error;
mod output;
mod server;
// mod test;

#[derive(Parser)]
#[command(name = "unity-catalog", version, about = "CLI to manage delta.sharing services.", long_about = None)]
struct Cli {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// Server URL
    #[clap(
        long,
        global = true,
        env = "UC_SERVER_URL",
        default_value = "http://localhost:8080"
    )]
    server: String,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(arg_required_else_help = true, about = "run a unity catalog server")]
    Server(ServerArgs),

    #[clap(
        arg_required_else_help = true,
        about = "execute requests against a sharing server"
    )]
    Client(ClientCommand),

    #[clap(about = "run database migrations")]
    Migrate,
}

#[derive(Parser)]
struct ClientArgs {
    #[clap(help = "Sets the server address")]
    endpoint: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match &args.command {
        Commands::Server(cmd) => handle_server(cmd).await?,
        Commands::Client(client_args) => {
            handle_client(client_args, args.global_opts).await?;
        }
        Commands::Migrate => todo!(),
    };
    Ok(())
}

// Handle the profile command.
// async fn handle_profile(args: &ProfileArgs) -> Result<()> {
//     let token_manager = TokenManager::new_from_secret(args.secret.as_bytes(), None);
//     let profile_manager = DeltaProfileManager::new(args.endpoint.clone(), 1, token_manager);
//
//     let exp = args
//         .validity
//         .and_then(|days| chrono::Utc::now().checked_add_days(Days::new(days)));
//     let shares = args
//         .shares
//         .split(',')
//         .map(|s| s.trim().to_ascii_lowercase())
//         .collect();
//     let claims = DefaultClaims {
//         sub: args.subject.clone(),
//         issued_at: chrono::Utc::now().timestamp(),
//         admin: args.admin,
//         exp: exp.as_ref().map(|dt| dt.timestamp() as u64),
//         shares,
//     };
//     let profile = profile_manager.issue_profile(&claims, exp).await?;
//     std::fs::write("profile.json", serde_json::to_string_pretty(&profile)?)?;
//     Ok(())
// }
