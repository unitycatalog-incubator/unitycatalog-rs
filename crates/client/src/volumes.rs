use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::volumes::v1::*;

use super::utils::stream_paginated;
use crate::Result;
use crate::codegen::volumes::builders::{CreateVolumeBuilder, UpdateVolumeBuilder};
pub(super) use crate::codegen::volumes::client::VolumeClient as VolumeClientBase;

impl VolumeClientBase {
    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
        include_browse: impl Into<Option<bool>>,
    ) -> BoxStream<'_, Result<VolumeInfo>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let schema_name = schema_name.into();
        let include_browse = include_browse.into();
        stream_paginated(
            (catalog_name, schema_name, max_results, include_browse),
            move |(catalog_name, schema_name, max_results, include_browse), page_token| async move {
                let request = ListVolumesRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name: schema_name.clone(),
                    max_results,
                    page_token,
                    include_browse,
                };
                let res = self.list_volumes(&request).await?;
                Ok((
                    res.volumes,
                    (catalog_name, schema_name, max_results, include_browse),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct VolumeClient {
    catalog_name: String,
    schema_name: String,
    name: String,
    client: VolumeClientBase,
}

impl VolumeClient {
    pub fn new(
        catalog_name: impl ToString,
        schema_name: impl ToString,
        name: impl ToString,
        client: VolumeClientBase,
    ) -> Self {
        Self {
            catalog_name: catalog_name.to_string(),
            schema_name: schema_name.to_string(),
            name: name.to_string(),
            client,
        }
    }

    pub fn new_from_full_name(full_name: impl ToString, client: VolumeClientBase) -> Self {
        let full_name = full_name.to_string();
        let parts: Vec<&str> = full_name.split('.').collect();
        if parts.len() != 3 {
            panic!("Invalid volume full name format. Expected: catalog.schema.volume");
        }
        Self {
            catalog_name: parts[0].to_string(),
            schema_name: parts[1].to_string(),
            name: parts[2].to_string(),
            client,
        }
    }

    pub fn full_name(&self) -> String {
        format!("{}.{}.{}", self.catalog_name, self.schema_name, self.name)
    }

    /// Create a new volume using the builder pattern.
    pub fn create(&self, volume_type: VolumeType) -> CreateVolumeBuilder {
        CreateVolumeBuilder::new(
            self.client.clone(),
            &self.catalog_name,
            &self.schema_name,
            &self.name,
            volume_type as i32,
        )
    }

    pub async fn get(&self, include_browse: impl Into<Option<bool>>) -> Result<VolumeInfo> {
        let request = GetVolumeRequest {
            name: self.full_name(),
            include_browse: include_browse.into(),
        };
        self.client.get_volume(&request).await
    }

    /// Update this volume using the builder pattern.
    pub fn update(&self) -> UpdateVolumeBuilder {
        UpdateVolumeBuilder::new(self.client.clone(), &self.full_name())
    }

    pub async fn delete(&self) -> Result<()> {
        let request = DeleteVolumeRequest {
            name: self.full_name(),
        };
        self.client.delete_volume(&request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unitycatalog_common::models::volumes::v1::VolumeType;

    #[test]
    fn test_volume_client_construction() {
        let client = VolumeClientBase::new(
            cloud_client::CloudClient::new_unauthenticated(),
            url::Url::parse("http://localhost:8080/").unwrap(),
        );

        let volume = VolumeClient::new("test_catalog", "test_schema", "test_volume", client);

        assert_eq!(volume.catalog_name, "test_catalog");
        assert_eq!(volume.schema_name, "test_schema");
        assert_eq!(volume.name, "test_volume");
        assert_eq!(volume.full_name(), "test_catalog.test_schema.test_volume");
    }

    #[test]
    fn test_volume_client_from_full_name() {
        let client = VolumeClientBase::new(
            cloud_client::CloudClient::new_unauthenticated(),
            url::Url::parse("http://localhost:8080/").unwrap(),
        );

        let volume = VolumeClient::new_from_full_name("catalog.schema.volume", client);

        assert_eq!(volume.catalog_name, "catalog");
        assert_eq!(volume.schema_name, "schema");
        assert_eq!(volume.name, "volume");
        assert_eq!(volume.full_name(), "catalog.schema.volume");
    }

    #[test]
    #[should_panic(expected = "Invalid volume full name format")]
    fn test_volume_client_from_invalid_full_name() {
        let client = VolumeClientBase::new(
            cloud_client::CloudClient::new_unauthenticated(),
            url::Url::parse("http://localhost:8080/").unwrap(),
        );

        VolumeClient::new_from_full_name("invalid.name", client);
    }

    #[test]
    fn test_volume_type_enum() {
        assert_eq!(VolumeType::Unspecified as i32, 0);
        assert_eq!(VolumeType::External as i32, 1);
        assert_eq!(VolumeType::Managed as i32, 2);
    }
}
