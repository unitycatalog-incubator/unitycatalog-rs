use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::volumes::v1::VolumeType;

// [snippet:list_volumes]
pub async fn list_volumes_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let response = client
        .list_volumes("my_catalog", "my_schema")
        .await
        .unwrap();
    for volume in response.volumes {
        println!("{}", volume.name);
    }
}
// [/snippet:list_volumes]

// [snippet:create_volume]
pub async fn create_volume_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let volume = client
        .create_volume("my_catalog", "my_schema", "my_volume", VolumeType::Managed)
        .with_comment("My first volume".to_string())
        .await
        .unwrap();
    println!("Created: {}", volume.name);
}
// [/snippet:create_volume]

// [snippet:get_volume]
pub async fn get_volume_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let volume = client
        .volume("my_catalog", "my_schema", "my_volume")
        .get()
        .await
        .unwrap();
    println!("Got: {}", volume.name);
}
// [/snippet:get_volume]

// [snippet:update_volume]
pub async fn update_volume_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    let volume = client
        .volume("my_catalog", "my_schema", "my_volume")
        .update()
        .with_comment("Updated comment".to_string())
        .await
        .unwrap();
    println!("Updated: {}", volume.name);
}
// [/snippet:update_volume]

// [snippet:delete_volume]
pub async fn delete_volume_example(base_url: url::Url) {
    let client = UnityCatalogClient::new_unauthenticated(base_url);
    client
        .volume("my_catalog", "my_schema", "my_volume")
        .delete()
        .await
        .unwrap();
    println!("Deleted volume");
}
// [/snippet:delete_volume]
