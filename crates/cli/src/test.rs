use futures::TryStreamExt;
use tracing::log::*;
use unitycatalog_common::client::UnityCatalogClient;

use crate::{GlobalOpts, server::init_tracing};

pub(crate) async fn run(opts: &GlobalOpts) -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let client = cloud_client::CloudClient::new_unauthenticated();
    let client = UnityCatalogClient::new(
        client,
        url::Url::parse("http://0.0.0.0:8080/api/2.1/unity-catalog/").unwrap(),
    );

    catalog_simple(client.clone()).await?;
    catalog_with_schema(client.clone()).await?;

    Ok(())
}

async fn catalog_simple(client: UnityCatalogClient) -> Result<(), Box<dyn std::error::Error>> {
    // list catalogs
    let catalogs: Vec<_> = client.list_catalogs(None).try_collect().await?;
    for catalog in catalogs {
        info!("Catalog: {}", catalog.name);
    }

    // create a catalog
    let catalog_name = "my_catalog";

    let catalog = client
        .create_catalog(catalog_name, None::<String>, None::<String>, None)
        .await?;
    info!("Created catalog: {:?}", catalog);

    // update the catalog
    let updated_catalog = client
        .catalog(catalog_name)
        .update(None::<String>, Some("new comment"), None::<String>, None)
        .await?;
    info!("Updated catalog: {:?}", updated_catalog);

    // list catalogs
    let catalogs: Vec<_> = client.list_catalogs(None).try_collect().await?;
    for catalog in catalogs {
        info!("Catalog: {}", catalog.name);
    }

    client.catalog(catalog_name).delete(None).await?;
    info!("Deleted catalog: {}", catalog_name);

    Ok(())
}

async fn catalog_with_schema(client: UnityCatalogClient) -> Result<(), Box<dyn std::error::Error>> {
    info!("=== Starting catalog and schema creation test ===");

    // Create a catalog for schema testing
    let catalog_name = "test_catalog_with_schema";
    info!("Creating catalog: {}", catalog_name);

    let catalog = client
        .create_catalog(
            catalog_name,
            None::<String>,
            Some("Catalog for schema testing"),
            None,
        )
        .await?;
    info!(
        "✓ Successfully created catalog: {} (id: {})",
        catalog.name,
        catalog.id.as_deref().unwrap_or("unknown")
    );

    // Create a schema within the catalog
    let schema_name = "test_schema";
    info!(
        "Creating schema: {} in catalog: {}",
        schema_name, catalog_name
    );

    let schema = client
        .create_schema(
            catalog_name,
            schema_name,
            Some("Schema for testing purposes".to_string()),
        )
        .await?;
    info!(
        "✓ Successfully created schema: {} (full name: {})",
        schema.name,
        schema.full_name.as_deref().unwrap_or("unknown")
    );

    // List schemas to verify creation
    info!("Listing all schemas in catalog: {}", catalog_name);
    let schemas: Vec<_> = client
        .list_schemas(catalog_name, None)
        .try_collect()
        .await?;
    for schema in &schemas {
        info!(
            "  - Found schema: {} (comment: {})",
            schema.full_name.as_deref().unwrap_or("unknown"),
            schema.comment.as_deref().unwrap_or("No comment")
        );
    }
    info!(
        "Total schemas found in catalog {}: {}",
        catalog_name,
        schemas.len()
    );

    // Update the schema
    info!("Updating schema with new comment...");
    let updated_schema = client
        .schema(catalog_name, schema_name)
        .update(
            None::<String>,
            Some("Updated schema comment for testing".to_string()),
            None,
        )
        .await?;
    info!(
        "✓ Successfully updated schema: {} with comment: {}",
        updated_schema.name,
        updated_schema.comment.as_deref().unwrap_or("No comment")
    );

    // Clean up: delete schema first, then catalog
    info!("Cleaning up resources...");

    info!("Deleting schema: {}.{}", catalog_name, schema_name);
    client
        .schema(catalog_name, schema_name)
        .delete(None)
        .await?;
    info!("✓ Successfully deleted schema: {}", schema_name);

    info!("Deleting catalog: {}", catalog_name);
    client.catalog(catalog_name).delete(None).await?;
    info!("✓ Successfully deleted catalog: {}", catalog_name);

    info!("=== Catalog and schema creation test completed successfully ===");
    Ok(())
}
