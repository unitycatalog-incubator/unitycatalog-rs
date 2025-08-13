use clap::{Args, Subcommand};
use futures::TryStreamExt;
use unitycatalog_common::client::UnityCatalogClient;

use crate::GlobalOpts;

#[derive(Debug, Args)]
pub struct ClientCommand {
    #[command(subcommand)]
    command: Option<ClientCommands>,
}

#[derive(Debug, Subcommand)]
enum ClientCommands {
    Catalogs(CatalogArgs),
    Schemas(SchemaArgs),
}

#[derive(Debug, Args)]
struct CatalogArgs {
    #[command(subcommand)]
    command: Option<CatalogCommands>,
}

#[derive(Debug, Subcommand)]
enum CatalogCommands {
    /// List the catalogs
    List,

    /// Create a new catalog
    Create {
        /// The name of the catalog to create
        #[clap(short, long)]
        name: String,
    },

    /// Get a catalog
    Get {
        /// The name of the catalog to get
        #[clap(short, long)]
        name: String,
    },

    /// Delete a catalog
    Delete {
        /// The name of the catalog to delete
        #[clap(short, long)]
        name: String,

        /// Whether to force delete the catalog
        #[clap(short, long)]
        force: Option<bool>,
    },
}

#[derive(Debug, Args)]
struct SchemaArgs {
    #[command(subcommand)]
    command: Option<SchemaCommands>,
}

#[derive(Debug, Subcommand)]
enum SchemaCommands {
    /// List the schemas in a catalog
    List {
        /// The name of the catalog to list the schemas from
        catalog_name: Option<String>,
    },

    /// Create a new schema in a catalog
    Create {
        /// The name of the catalog to create the schema in
        catalog_name: String,
        /// The name of the schema to create
        name: String,
    },

    /// Delete a schema from a catalog
    Delete {
        /// The name of the catalog to delete the schema from
        catalog_name: String,
        /// The name of the schema to delete
        name: String,
    },
}

pub async fn handle_client(
    cmd: &ClientCommand,
    opts: GlobalOpts,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = cloud_client::CloudClient::new_unauthenticated();
    let client = UnityCatalogClient::new(client, url::Url::parse(&opts.server).unwrap());
    match &cmd.command {
        Some(ClientCommands::Catalogs(args)) => match &args.command {
            Some(CatalogCommands::List) => {
                let catalogs = client.list_catalogs(None).try_collect::<Vec<_>>().await?;
                println!("List catalogs: {catalogs:?}");
            }
            Some(CatalogCommands::Create { name }) => {
                let catalog = client
                    .create_catalog(name, None::<String>, None::<String>, None)
                    .await?;
                println!("Create catalog: {catalog:?}");
            }
            Some(CatalogCommands::Get { name }) => {
                let catalog = client.catalog(name).get().await?;
                println!("Create catalog: {catalog:?}");
            }
            Some(CatalogCommands::Delete { name, force }) => {
                client.catalog(name).delete(*force).await?;
                println!("Deleted catalog: {name:?}");
            }
            None => {
                println!("No command provided: {:?}", args.command);
            }
        },
        Some(ClientCommands::Schemas(args)) => match &args.command {
            Some(SchemaCommands::List { catalog_name }) => {
                println!("List schemas: {catalog_name:?}");
            }
            Some(SchemaCommands::Create { catalog_name, name }) => {
                println!("Create schema: {name} in catalog: {catalog_name}");
            }
            Some(SchemaCommands::Delete { catalog_name, name }) => {
                println!("Delete schema: {name} from catalog: {catalog_name}");
            }
            None => {
                println!("No command provided: {:?}", args.command);
            }
        },
        _ => {
            println!("No command provided: {:?}", opts.server);
        }
    };
    Ok(())
}
