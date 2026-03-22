use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::tables::v1::{DataSourceFormat, TableType};

// [snippet:list_tables]
pub async fn list_tables_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let response = client.list_tables("my_catalog", "my_schema").await.unwrap();
    for table in response.tables {
        println!("{}", table.name);
    }
}
// [/snippet:list_tables]

// [snippet:create_table]
pub async fn create_table_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let table = client
        .create_table(
            "my_table",
            "my_schema",
            "my_catalog",
            TableType::Managed,
            DataSourceFormat::Delta,
        )
        .with_comment("My first table".to_string())
        .await
        .unwrap();
    println!("Created: {}", table.name);
}
// [/snippet:create_table]

// [snippet:get_table]
pub async fn get_table_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let table = client
        .table("my_catalog.my_schema.my_table")
        .get()
        .await
        .unwrap();
    println!("Got: {}", table.name);
}
// [/snippet:get_table]

// [snippet:delete_table]
pub async fn delete_table_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    client
        .table("my_catalog.my_schema.my_table")
        .delete()
        .await
        .unwrap();
    println!("Deleted table");
}
// [/snippet:delete_table]
