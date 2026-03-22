use unitycatalog_client::UnityCatalogClient;

// [snippet:list_catalogs]
pub async fn list_catalogs_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let response = client.list_catalogs().await.unwrap();
    for catalog in response.catalogs {
        println!("{}", catalog.name);
    }
}
// [/snippet:list_catalogs]

// [snippet:create_catalog]
pub async fn create_catalog_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let catalog = client
        .create_catalog("my_catalog")
        .with_comment("My first catalog".to_string())
        .await
        .unwrap();
    println!("Created: {}", catalog.name);
}
// [/snippet:create_catalog]

// [snippet:get_catalog]
pub async fn get_catalog_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let catalog = client.catalog("my_catalog").get().await.unwrap();
    println!("Got: {}", catalog.name);
}
// [/snippet:get_catalog]

// [snippet:update_catalog]
pub async fn update_catalog_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let catalog = client
        .catalog("my_catalog")
        .update()
        .with_comment("Updated comment".to_string())
        .await
        .unwrap();
    println!("Updated: {}", catalog.name);
}
// [/snippet:update_catalog]

// [snippet:delete_catalog]
pub async fn delete_catalog_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    client.catalog("my_catalog").delete().await.unwrap();
    println!("Deleted catalog");
}
// [/snippet:delete_catalog]
