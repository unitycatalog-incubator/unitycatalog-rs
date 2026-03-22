use unitycatalog_client::UnityCatalogClient;

// [snippet:list_schemas]
pub async fn list_schemas_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let response = client.list_schemas("my_catalog").await.unwrap();
    for schema in response.schemas {
        println!("{}", schema.name);
    }
}
// [/snippet:list_schemas]

// [snippet:create_schema]
pub async fn create_schema_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let schema = client
        .create_schema("my_catalog", "my_schema")
        .with_comment("My first schema".to_string())
        .await
        .unwrap();
    println!("Created: {}.{}", schema.catalog_name, schema.name);
}
// [/snippet:create_schema]

// [snippet:get_schema]
pub async fn get_schema_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let schema = client
        .schema("my_catalog", "my_schema")
        .get()
        .await
        .unwrap();
    println!("Got: {}.{}", schema.catalog_name, schema.name);
}
// [/snippet:get_schema]

// [snippet:update_schema]
pub async fn update_schema_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let schema = client
        .schema("my_catalog", "my_schema")
        .update()
        .with_comment("Updated comment".to_string())
        .await
        .unwrap();
    println!("Updated: {}.{}", schema.catalog_name, schema.name);
}
// [/snippet:update_schema]

// [snippet:delete_schema]
pub async fn delete_schema_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    client
        .schema("my_catalog", "my_schema")
        .delete()
        .await
        .unwrap();
    println!("Deleted schema");
}
// [/snippet:delete_schema]
