//! Example showing the new builder API for Unity Catalog client
//!
//! This example demonstrates how to use the builder pattern for creating
//! and updating Unity Catalog resources.

use unitycatalog_client::{UnityCatalogClient, Result};
use unitycatalog_common::{
    credentials::v1::Purpose,
    models::volumes::v1::VolumeType,
    models::tables::v1::{DataSourceFormat, TableType},
    recipients::v1::AuthenticationType,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the Unity Catalog client
    let client = UnityCatalogClient::new_unauthenticated(
        "http://localhost:8080/api/".parse().unwrap()
    );

    // Create a catalog using the builder pattern
    let catalog = client
        .create_catalog("analytics_catalog")
        .with_comment("Main analytics catalog for the data team")
        .with_properties([
            ("owner", "data_team"),
            ("environment", "production"),
            ("cost_center", "analytics"),
        ])
        .with_storage_root("s3://my-analytics-bucket/")
        .await?;

    println!("Created catalog: {}", catalog.name);

    // Create a schema using the builder pattern
    let schema = client
        .create_schema("analytics_catalog", "sales_data")
        .with_comment("Schema containing all sales-related data")
        .with_properties([
            ("department", "sales"),
            ("data_classification", "internal"),
        ])
        .await?;

    println!("Created schema: {}", schema.name);

    // Create a table using the schema client
    let table = client
        .schema("analytics_catalog", "sales_data")
        .create_table(
            "monthly_sales",
            TableType::Managed,
            DataSourceFormat::Delta
        )
        .with_comment("Monthly sales aggregation table")
        .with_storage_location("s3://my-analytics-bucket/tables/monthly_sales/")
        .with_properties([
            ("partition_columns", "year,month"),
            ("table_type", "fact"),
        ])
        .await?;

    println!("Created table: {}", table.name);

    // Create credentials using the builder pattern
    let credential = client
        .create_credential("s3_analytics_creds", Purpose::StorageAccess)
        .with_comment("Credentials for accessing analytics S3 bucket")
        .with_read_only(false)
        .with_skip_validation(false)
        .await?;

    println!("Created credential: {}", credential.name);

    // Create an external location using the builder pattern
    let external_location = client
        .create_external_location(
            "analytics_external",
            "s3://external-analytics-bucket/shared/",
            "s3_analytics_creds"
        )?
        .with_comment("External location for shared analytics data")
        .with_read_only(true)
        .with_skip_validation(false)
        .await?;

    println!("Created external location: {}", external_location.name);

    // Create a volume using the builder pattern
    let volume = client
        .create_volume(
            "analytics_catalog",
            "sales_data",
            "raw_files",
            VolumeType::External
        )
        .with_storage_location("s3://my-analytics-bucket/volumes/raw_files/")
        .with_comment("Volume for storing raw sales files")
        .await?;

    println!("Created volume: {}", volume.name);

    // Create a share using the builder pattern
    let share = client
        .create_share("analytics_share")
        .with_comment("Share for analytics data with external partners")
        .await?;

    println!("Created share: {}", share.name);

    // Create a recipient using the builder pattern
    let recipient = client
        .create_recipient(
            "partner_company",
            AuthenticationType::Token,
            "partner@company.com"
        )
        .with_comment("External partner for data sharing")
        .with_properties([
            ("company", "Partner Corp"),
            ("contact_email", "partner@company.com"),
        ])
        .await?;

    println!("Created recipient: {}", recipient.name);

    // Update resources using the builder pattern

    // Update the catalog
    let updated_catalog = client
        .catalog("analytics_catalog")
        .update()
        .with_comment("Updated analytics catalog with new data sources")
        .with_properties([
            ("owner", "senior_data_team"),
            ("last_updated", "2024-01-15"),
        ])
        .await?;

    println!("Updated catalog: {}", updated_catalog.name);

    // Update the schema
    let updated_schema = client
        .schema("analytics_catalog", "sales_data")
        .update()
        .with_comment("Updated sales data schema with new tables")
        .with_properties([
            ("table_count", "15"),
            ("last_modified", "2024-01-15"),
        ])
        .await?;

    println!("Updated schema: {}", updated_schema.name);

    // Update the share
    let updated_share = client
        .share("analytics_share")
        .update()
        .with_new_name("external_analytics_share")
        .with_comment("Renamed and updated analytics share")
        .with_owner("data_governance_team")
        .await?;

    println!("Updated share: {}", updated_share.name);

    // Update the recipient
    let updated_recipient = client
        .recipient("partner_company")
        .update()
        .with_comment("Updated partner information")
        .with_properties([
            ("status", "active"),
            ("tier", "premium"),
        ])
        .with_expiration_time(1735689600) // Jan 1, 2025
        .await?;

    println!("Updated recipient: {}", updated_recipient.name);

    // Update the credential
    let updated_credential = client
        .credential("s3_analytics_creds")
        .update()
        .with_comment("Updated S3 credentials with enhanced permissions")
        .with_read_only(false)
        .with_force(true)
        .await?;

    println!("Updated credential: {}", updated_credential.name);

    // Update the external location
    let updated_external_location = client
        .external_location("analytics_external")
        .update()
        .with_comment("Updated external location with new URL")
        .with_url("s3://new-external-analytics-bucket/shared/")?
        .with_read_only(false)
        .await?;

    println!("Updated external location: {}", updated_external_location.name);

    // Update the volume
    let updated_volume = client
        .volume("analytics_catalog", "sales_data", "raw_files")
        .update()
        .with_comment("Updated volume with new configuration")
        .with_owner("data_engineering_team")
        .with_include_browse(true)
        .await?;

    println!("Updated volume: {}", updated_volume.name);

    // Generate temporary credentials using the builder pattern
    let temp_table_creds = client
        .temporary_credentials()
        .temporary_table_credential(
            "table_12345",
            unitycatalog_client::TableOperation::ReadWrite
        )
        .await?;

    println!("Generated temporary table credentials");

    let temp_path_creds = client
        .temporary_credentials()
        .temporary_path_credential(
            "s3://my-bucket/temp/",
            unitycatalog_client::PathOperation::ReadWrite
        )?
        .with_dry_run(false)
        .await?;

    println!("Generated temporary path credentials");

    println!("All operations completed successfully!");

    Ok(())
}
