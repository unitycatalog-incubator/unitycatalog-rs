//! Comprehensive example demonstrating flexible builder APIs for Unity Catalog
//!
//! This example showcases how the Unity Catalog Rust client provides maximum flexibility
//! for both properties (HashMap-like fields) and vector/collection parameters through
//! generic IntoIterator and IntoIterator<Item = (K, V)> implementations.

use std::collections::HashMap;
use unitycatalog_client::catalogs::CatalogClient;
use unitycatalog_client::schemas::SchemaClient;
use unitycatalog_client::tables::TableClient;
use unitycatalog_client::shares::ShareClient;
use unitycatalog_client::sharing::SharingClient;
use unitycatalog_common::models::tables::v1::{ColumnInfo, TableType, DataSourceFormat};
use unitycatalog_common::models::shares::v1::DataObjectUpdate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Unity Catalog Flexible Builder APIs Example");
    println!("==========================================\n");

    // Create mock clients (in real usage, these would connect to a Unity Catalog server)
    let catalog_client = CatalogClient::new("http://localhost:8080".to_string());
    let schema_client = SchemaClient::new("http://localhost:8080".to_string());
    let table_client = TableClient::new("http://localhost:8080".to_string());
    let share_client = ShareClient::new("http://localhost:8080".to_string());
    let sharing_client = SharingClient::new("http://localhost:8080".to_string());

    println!("🔧 FLEXIBLE PROPERTIES API EXAMPLES");
    println!("===================================\n");

    // Properties API: Accept any iterable of key-value pairs

    println!("1. HashMap<String, String> properties");
    let mut owned_properties = HashMap::new();
    owned_properties.insert("environment".to_string(), "production".to_string());
    owned_properties.insert("team".to_string(), "data-engineering".to_string());

    let _catalog_builder1 = catalog_client
        .create_catalog("production_catalog")
        .with_properties(owned_properties);
    println!("   ✓ Built with HashMap<String, String>");

    println!("2. HashMap<&str, &str> properties (convenient for literals)");
    let mut str_properties = HashMap::new();
    str_properties.insert("environment", "staging");
    str_properties.insert("auto_vacuum", "enabled");

    let _catalog_builder2 = catalog_client
        .create_catalog("staging_catalog")
        .with_properties(str_properties);
    println!("   ✓ Built with HashMap<&str, &str>");

    println!("3. Vec<(String, String)> properties");
    let vec_properties = vec![
        ("table_type".to_string(), "fact".to_string()),
        ("partition_by".to_string(), "date".to_string()),
    ];

    let _schema_builder1 = schema_client
        .create_schema("analytics_schema", "production_catalog")
        .with_properties(vec_properties);
    println!("   ✓ Built with Vec<(String, String)>");

    println!("4. Array literal properties (most concise)");
    let _schema_builder2 = schema_client
        .create_schema("temp_schema", "staging_catalog")
        .with_properties([
            ("temporary", "true"),
            ("ttl_hours", "24"),
            ("auto_delete", "enabled"),
        ]);
    println!("   ✓ Built with array literals");

    println!("5. Iterator-generated properties");
    let dynamic_properties = (1..=3)
        .map(|i| (format!("partition_{}", i), format!("value_{}", i)));

    let _schema_builder3 = schema_client
        .create_schema("dynamic_schema", "production_catalog")
        .with_properties(dynamic_properties);
    println!("   ✓ Built with iterator-generated properties");

    println!("\n📋 FLEXIBLE VECTOR API EXAMPLES");
    println!("===============================\n");

    // Vector API: Accept any iterable of the target type

    println!("1. Vec<ColumnInfo> columns");
    let columns_vec = vec![
        ColumnInfo {
            name: "id".to_string(),
            type_text: "BIGINT".to_string(),
            type_name: Some(1),
            type_precision: Some(19),
            type_scale: Some(0),
            position: 0,
            comment: Some("Primary key".to_string()),
            nullable: Some(false),
            partition_index: None,
            type_interval_type: None,
        },
        ColumnInfo {
            name: "name".to_string(),
            type_text: "STRING".to_string(),
            type_name: Some(6),
            type_precision: None,
            type_scale: None,
            position: 1,
            comment: Some("Record name".to_string()),
            nullable: Some(true),
            partition_index: None,
            type_interval_type: None,
        },
    ];

    let _table_builder1 = table_client
        .create_table(
            "users_table",
            "analytics_schema",
            "production_catalog",
            TableType::Managed,
            DataSourceFormat::Delta,
        )
        .with_columns(columns_vec);
    println!("   ✓ Built with Vec<ColumnInfo>");

    println!("2. Array of ColumnInfo");
    let columns_array = [
        ColumnInfo {
            name: "transaction_id".to_string(),
            type_text: "STRING".to_string(),
            type_name: Some(6),
            type_precision: None,
            type_scale: None,
            position: 0,
            comment: Some("Transaction identifier".to_string()),
            nullable: Some(false),
            partition_index: None,
            type_interval_type: None,
        },
        ColumnInfo {
            name: "amount".to_string(),
            type_text: "DECIMAL".to_string(),
            type_name: Some(3),
            type_precision: Some(10),
            type_scale: Some(2),
            position: 1,
            comment: Some("Transaction amount".to_string()),
            nullable: Some(false),
            partition_index: None,
            type_interval_type: None,
        },
    ];

    let _table_builder2 = table_client
        .create_table(
            "transactions_table",
            "analytics_schema",
            "production_catalog",
            TableType::External,
            DataSourceFormat::Parquet,
        )
        .with_columns(columns_array);
    println!("   ✓ Built with [ColumnInfo; N] array");

    println!("3. Iterator of ColumnInfo");
    let columns_iter = (0..3).map(|i| ColumnInfo {
        name: format!("dynamic_col_{}", i),
        type_text: "STRING".to_string(),
        type_name: Some(6),
        type_precision: None,
        type_scale: None,
        position: i as i32,
        comment: Some(format!("Dynamic column {}", i)),
        nullable: Some(true),
        partition_index: None,
        type_interval_type: None,
    });

    let _table_builder3 = table_client
        .create_table(
            "dynamic_table",
            "analytics_schema",
            "production_catalog",
            TableType::Managed,
            DataSourceFormat::Iceberg,
        )
        .with_columns(columns_iter);
    println!("   ✓ Built with iterator-generated columns");

    println!("4. Vec<String> for string arrays");
    let predicate_hints_vec = vec![
        "partition_date > '2024-01-01'".to_string(),
        "status = 'active'".to_string(),
    ];

    let _query_builder1 = sharing_client
        .query_table("shared_table", "share_name")
        .with_predicate_hints(predicate_hints_vec);
    println!("   ✓ Built with Vec<String>");

    println!("5. Array of &str (most convenient for literals)");
    let _query_builder2 = sharing_client
        .query_table("analytics_table", "public_share")
        .with_predicate_hints([
            "year = 2024",
            "region IN ('US', 'EU')",
            "active = true",
        ]);
    println!("   ✓ Built with [&str; N] array");

    println!("6. Iterator of String");
    let dynamic_hints = (1..=3)
        .map(|i| format!("condition_{} = 'value_{}'", i, i));

    let _query_builder3 = sharing_client
        .query_table("filtered_table", "analytics_share")
        .with_predicate_hints(dynamic_hints);
    println!("   ✓ Built with iterator-generated strings");

    println!("7. Vec<DataObjectUpdate> for share updates");
    let updates_vec = vec![
        DataObjectUpdate {
            action: Some(1), // ADD
            data_object: Some("table1".to_string()),
            ..Default::default()
        },
        DataObjectUpdate {
            action: Some(2), // REMOVE
            data_object: Some("table2".to_string()),
            ..Default::default()
        },
    ];

    let _share_builder = share_client
        .update_share("my_share")
        .with_updates(updates_vec);
    println!("   ✓ Built with Vec<DataObjectUpdate>");

    println!("\n🔗 COMBINING FLEXIBLE APIS");
    println!("=========================\n");

    println!("Type-safe enum usage examples:");

    // Create additional clients for demonstration
    let credential_client = unitycatalog_client::credentials::CredentialClient::new("http://localhost:8080".to_string());
    let recipient_client = unitycatalog_client::recipients::RecipientClient::new("http://localhost:8080".to_string());
    let volume_client = unitycatalog_client::volumes::VolumeClient::new("http://localhost:8080".to_string());

    let _credential_builder = credential_client
        .create_credential("azure_cred", unitycatalog_common::models::credentials::v1::Purpose::Storage);
    println!("   ✓ Built credential with Purpose::Storage enum");

    let _recipient_builder = recipient_client
        .create_recipient(
            "data_recipient",
            unitycatalog_common::models::recipients::v1::AuthenticationType::Token,
            "admin@company.com"
        );
    println!("   ✓ Built recipient with AuthenticationType::Token enum");

    let _volume_builder = volume_client
        .create_volume(
            "production_catalog",
            "data_schema",
            "logs_volume",
            unitycatalog_common::models::volumes::v1::VolumeType::External
        );
    println!("   ✓ Built volume with VolumeType::External enum");

    println!();

    println!("Complex builder with both flexible properties and vectors:");
    let _comprehensive_builder = table_client
        .create_table(
            "comprehensive_table",
            "production_schema",
            "production_catalog",
            TableType::Managed,
            DataSourceFormat::Delta,
        )
        // Flexible properties from various sources
        .with_properties([
            ("created_by", "data_team"),
            ("environment", "production"),
            ("criticality", "high"),
        ])
        // Flexible columns from iterator
        .with_columns(
            ["id", "name", "email", "created_at"]
                .iter()
                .enumerate()
                .map(|(i, &col_name)| ColumnInfo {
                    name: col_name.to_string(),
                    type_text: if col_name == "id" { "BIGINT" } else { "STRING" }.to_string(),
                    type_name: Some(if col_name == "id" { 1 } else { 6 }),
                    type_precision: None,
                    type_scale: None,
                    position: i as i32,
                    comment: Some(format!("Column: {}", col_name)),
                    nullable: Some(col_name != "id"),
                    partition_index: None,
                    type_interval_type: None,
                })
        )
        .with_comment("Comprehensive table with flexible APIs")
        .with_storage_location("/data/warehouse/comprehensive");

    println!("   ✓ Combined flexible properties and vectors in one builder");

    println!("\n✨ KEY BENEFITS SUMMARY");
    println!("=====================\n");
    println!("📌 Properties API Benefits:");
    println!("   • Accept HashMap, Vec, arrays, and iterators");
    println!("   • Work with both owned (String) and borrowed (&str) data");
    println!("   • Enable convenient literal syntax: [('key', 'value')]");
    println!("   • Support dynamic property generation with iterators");
    println!("   • Maintain type safety with compile-time checks");

    println!("\n📌 Vector API Benefits:");
    println!("   • Accept Vec, arrays, and any iterator of the target type");
    println!("   • Enable efficient iteration without intermediate collections");
    println!("   • Support both owned and borrowed data where applicable");
    println!("   • Allow dynamic generation of vector contents");
    println!("   • Provide ergonomic array literal syntax");

    println!("\n📌 Enum Type Benefits:");
    println!("   • Type-safe alternatives to raw i32 values");
    println!("   • Prevent invalid enum values at compile time");
    println!("   • IDE autocomplete and documentation support");
    println!("   • Self-documenting API with meaningful names");
    println!("   • Backward compatible with automatic i32 conversion");

    println!("\n📌 Combined Benefits:");
    println!("   • Maximum flexibility without sacrificing type safety");
    println!("   • Excellent ergonomics for different use cases");
    println!("   • Zero-cost abstractions - no performance overhead");
    println!("   • Seamless integration with existing Rust ecosystem");
    println!("   • Future-proof API that works with new iterable types");

    println!("\n🚀 All builder patterns demonstrate compile-time flexibility!");
    println!("   In real applications, call .await to execute these builders.");

    Ok(())
}
