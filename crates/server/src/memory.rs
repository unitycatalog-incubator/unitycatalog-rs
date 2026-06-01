use std::sync::Arc;

use dashmap::DashMap;
use uuid::Uuid;

use unitycatalog_common::models::{AssociationLabel, ObjectLabel, PropertyMap, Resource};
use unitycatalog_common::models::{ResourceExt, ResourceIdent, ResourceName, ResourceRef};

use unitycatalog_common::{Error, Result};

use unitycatalog_common::services::encryption::EnvelopeEncryptor;

use crate::services::secrets::SecretManager;
use crate::store::{ResourceStore, ResourceStoreReader};

const MAX_PAGE_SIZE: usize = 10000;

/// An in-memory implementation of a resource store.
///
/// This store is not intended for production use, but is useful for testing and development.
#[derive(Debug, Clone)]
pub struct InMemoryResourceStore {
    resources: Arc<DashMap<Uuid, Resource>>,
    id_map: Arc<DashMap<ObjectLabel, DashMap<ResourceName, Uuid>>>,
    associations: Arc<DashMap<AssociationLabel, DashMap<Uuid, (Uuid, Option<PropertyMap>)>>>,
    /// Sealed secret blobs keyed by name. Encryption matches the production path so dev/test
    /// behaviour (and the on-disk format) is exercised here too.
    secrets: Arc<DashMap<String, bytes::Bytes>>,
    encryptor: EnvelopeEncryptor,
}

impl InMemoryResourceStore {
    pub fn new(encryptor: EnvelopeEncryptor) -> Self {
        Self {
            resources: DashMap::new().into(),
            id_map: DashMap::new().into(),
            associations: DashMap::new().into(),
            secrets: DashMap::new().into(),
            encryptor,
        }
    }

    fn get_uuid(&self, label: &ObjectLabel, name: &ResourceName) -> Option<Uuid> {
        self.id_map
            .get(label)
            .and_then(|map| map.value().get(name).map(|uuid| *uuid.value()))
    }

    fn remove_uuid(&self, label: &ObjectLabel, name: &ResourceName) -> Option<Uuid> {
        self.id_map
            .get(label)
            .and_then(|map| map.value().remove(name).map(|(_, uuid)| uuid))
    }

    fn new_uuid(&self, label: &ObjectLabel, name: &ResourceName) -> Result<Uuid> {
        if self.get_uuid(label, name).is_some() {
            return Err(Error::AlreadyExists);
        }
        let map = self.id_map.entry(*label).or_default();
        let uuid = Uuid::now_v7();
        map.insert(name.clone(), uuid);
        Ok(uuid)
    }
}

#[async_trait::async_trait]
impl ResourceStoreReader for InMemoryResourceStore {
    async fn get(&self, id: &ResourceIdent) -> Result<(Resource, ResourceRef)> {
        let resource = match id.as_ref() {
            ResourceRef::Uuid(uuid) => self.resources.get(uuid),
            ResourceRef::Name(name) => {
                let uuid = self.get_uuid(id.label(), name).ok_or(Error::NotFound)?;
                self.resources.get(&uuid)
            }
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        match resource {
            Some(resource) => Ok((resource.value().clone(), resource.key().into())),
            None => Err(Error::NotFound),
        }
    }

    async fn list(
        &self,
        label: &ObjectLabel,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<Resource>, Option<String>)> {
        let page_token = page_token.map(|t| Uuid::parse_str(&t)).transpose()?;
        let mut resource_ids = self
            .id_map
            .get(label)
            .map(|map| {
                map.value()
                    .iter()
                    .filter(|entry| {
                        namespace.is_none_or(|ns| entry.key().prefix_matches(ns))
                            && page_token.is_none_or(|t| &t > entry.value())
                    })
                    .map(|entry| *entry.value())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if resource_ids.is_empty() {
            return Ok((Vec::new(), None));
        }
        resource_ids.sort_unstable();

        let max_page_size = usize::min(max_results.unwrap_or(MAX_PAGE_SIZE), MAX_PAGE_SIZE);
        let mut resources = Vec::new();
        let mut last_id = &Uuid::nil();
        for uuid in resource_ids.iter().rev().take(max_page_size) {
            let resource = self.resources.get(uuid).ok_or(Error::NotFound)?.clone();
            last_id = uuid;
            resources.push(resource);
        }
        let next_page_token = (resources.len() == max_page_size).then(|| last_id.to_string());
        Ok((resources, next_page_token))
    }
}

#[async_trait::async_trait]
impl ResourceStore for InMemoryResourceStore {
    async fn create(&self, resource: Resource) -> Result<(Resource, ResourceRef)> {
        if self
            .get_uuid(resource.resource_label(), &resource.resource_name())
            .is_some()
        {
            return Err(Error::AlreadyExists);
        }
        let uuid = self.new_uuid(resource.resource_label(), &resource.resource_name())?;
        self.resources.insert(uuid, resource.clone());
        Ok((resource, ResourceRef::Uuid(uuid)))
    }

    async fn delete(&self, id: &ResourceIdent) -> Result<()> {
        let uuid = match id.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(id.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        match self.resources.remove(&uuid) {
            Some((_, resource)) => self.remove_uuid(id.label(), &resource.resource_name()),
            None => None,
        };
        Ok(())
    }

    async fn update(
        &self,
        id: &ResourceIdent,
        resource: Resource,
    ) -> Result<(Resource, ResourceRef)> {
        let uuid = match id.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(id.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        // Need to clone to avoid locking the map while holding a reference to the value
        let existing = self
            .resources
            .get(&uuid)
            .ok_or(Error::NotFound)?
            .value()
            .clone();
        if existing.resource_label() != resource.resource_label() {
            self.id_map
                .get(existing.resource_label())
                .and_then(|map| map.value().remove(&existing.resource_name()));
            self.id_map
                .entry(*resource.resource_label())
                .or_default()
                .insert(resource.resource_name().clone(), uuid);
        } else if existing.resource_name() != resource.resource_name() {
            self.id_map
                .get(existing.resource_label())
                .and_then(|map| map.value().remove(&existing.resource_name()));
            self.id_map
                .get(existing.resource_label())
                .and_then(|map| map.value().insert(resource.resource_name(), uuid));
        }
        self.resources.insert(uuid, resource.clone());
        Ok((resource, ResourceRef::Uuid(uuid)))
    }

    async fn add_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
        properties: Option<PropertyMap>,
    ) -> Result<()> {
        let from_uuid = match from.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(from.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        let to_uuid = match to.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(to.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        let map = self.associations.entry(label.clone()).or_default();
        map.insert(from_uuid, (to_uuid, properties.clone()));
        if let Some(inverse) = label.inverse() {
            let inverse_map = self.associations.entry(inverse).or_default();
            inverse_map.insert(to_uuid, (from_uuid, properties.clone()));
        }
        Ok(())
    }

    async fn remove_association(
        &self,
        from: &ResourceIdent,
        to: &ResourceIdent,
        label: &AssociationLabel,
    ) -> Result<()> {
        let from_uuid = match from.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(from.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        let to_uuid = match to.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self.get_uuid(to.label(), name).ok_or(Error::NotFound)?,
            ResourceRef::Undefined => return Err(Error::NotFound),
        };
        let map = self.associations.get(label).ok_or(Error::NotFound)?;
        map.remove(&from_uuid);
        if let Some(inverse) = label.inverse() {
            let inverse_map = self.associations.get(&inverse).ok_or(Error::NotFound)?;
            inverse_map.remove(&to_uuid);
        }
        Ok(())
    }

    async fn list_associations(
        &self,
        resource: &ResourceIdent,
        label: &AssociationLabel,
        target_label: Option<&ResourceIdent>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> Result<(Vec<ResourceIdent>, Option<String>)> {
        let resource_uuid = match resource.as_ref() {
            ResourceRef::Uuid(uuid) => *uuid,
            ResourceRef::Name(name) => self
                .get_uuid(resource.label(), name)
                .ok_or(Error::NotFound)?,
            ResourceRef::Undefined => {
                return Err(Error::invalid_argument("resource must not be undefined"));
            }
        };
        let target_uuid = target_label
            .map(|tl| match tl.as_ref() {
                ResourceRef::Uuid(uuid) => Ok(*uuid),
                ResourceRef::Name(name) => self.get_uuid(tl.label(), name).ok_or(Error::NotFound),
                ResourceRef::Undefined => Err(Error::invalid_argument(
                    "target resource must not be undefined",
                )),
            })
            .transpose()?;
        let page_token = page_token.map(|t| Uuid::parse_str(&t)).transpose()?;
        let mut association_ids = self
            .associations
            .get(label)
            .map(|map| {
                map.value()
                    .get(&resource_uuid)
                    .iter()
                    .filter(|entry| {
                        target_uuid.is_none_or(|uuid| entry.value().0 == uuid)
                            && page_token.is_none_or(|t| &t > entry.key())
                    })
                    .map(|entry| entry.value().0)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if association_ids.is_empty() {
            return Ok((Vec::new(), None));
        }
        association_ids.sort_unstable();

        let max_page_size = usize::min(max_results.unwrap_or(MAX_PAGE_SIZE), MAX_PAGE_SIZE);
        let mut resources = Vec::new();
        let mut last_id = &Uuid::nil();
        for uuid in association_ids.iter().rev().take(max_page_size) {
            let resource = self.resources.get(uuid).ok_or(Error::NotFound)?;
            last_id = uuid;
            resources.push(resource.resource_ident());
        }
        let next_page_token = (resources.len() == max_page_size).then(|| last_id.to_string());
        Ok((resources, next_page_token))
    }
}

#[async_trait::async_trait]
impl SecretManager for InMemoryResourceStore {
    async fn get_secret(&self, secret_name: &str) -> Result<bytes::Bytes> {
        let blob = self
            .secrets
            .get(secret_name)
            .map(|entry| entry.value().clone())
            .ok_or(Error::NotFound)?;
        let plaintext = self.encryptor.open(secret_name, &blob).await?;

        // Lazy KEK rotation: re-wrap under the active KEK if sealed under a retired one (mirrors
        // the Postgres store). Best-effort; never fails the read.
        if let Ok(Some(rewrapped)) = self.encryptor.rewrap(&blob).await {
            self.secrets
                .insert(secret_name.to_string(), bytes::Bytes::from(rewrapped));
        }

        Ok(bytes::Bytes::from(plaintext))
    }

    async fn put_secret(&self, secret_name: &str, secret_value: bytes::Bytes) -> Result<()> {
        let blob = self.encryptor.seal(secret_name, &secret_value).await?;
        self.secrets
            .insert(secret_name.to_string(), bytes::Bytes::from(blob));
        Ok(())
    }

    async fn delete_secret(&self, secret_name: &str) -> Result<()> {
        self.secrets.remove(secret_name).ok_or(Error::NotFound)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unitycatalog_common::models::{Catalog, ObjectLabel};
    use unitycatalog_common::services::encryption::LocalKeyProvider;

    fn test_store() -> InMemoryResourceStore {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        InMemoryResourceStore::new(encryptor)
    }

    #[tokio::test]
    async fn test_get_secret_lazily_rewraps_after_rotation() {
        // Seal a value under v1.
        let v1 = EnvelopeEncryptor::local(LocalKeyProvider::single("v1", vec![0x01; 32]).unwrap());
        let sealed_v1 = v1.seal("cred", b"value").await.unwrap();

        // Build a store whose active KEK is v2 (v1 retired) and plant the v1-sealed blob.
        let rotated = EnvelopeEncryptor::local(
            LocalKeyProvider::new(
                "v2",
                [("v1".into(), vec![0x01; 32]), ("v2".into(), vec![0x02; 32])],
            )
            .unwrap(),
        );
        let store = InMemoryResourceStore::new(rotated);
        store
            .secrets
            .insert("cred".to_string(), bytes::Bytes::from(sealed_v1.clone()));

        // Reading returns the correct plaintext...
        assert_eq!(&store.get_secret("cred").await.unwrap()[..], b"value");
        // ...and the stored blob has been re-wrapped (it differs from the v1 blob, and a
        // v2-only encryptor can now open it).
        let after = store.secrets.get("cred").unwrap().value().clone();
        assert_ne!(&after[..], &sealed_v1[..]);
        let v2_only =
            EnvelopeEncryptor::local(LocalKeyProvider::single("v2", vec![0x02; 32]).unwrap());
        assert_eq!(v2_only.open("cred", &after).await.unwrap(), b"value");
    }

    #[tokio::test]
    async fn test_secret_round_trip() {
        let store = test_store();
        store
            .put_secret("cred", bytes::Bytes::from_static(b"top secret"))
            .await
            .unwrap();
        // Stored blob must be ciphertext, not the plaintext.
        let blob = store.secrets.get("cred").unwrap().value().clone();
        assert!(
            !blob
                .windows(b"top secret".len())
                .any(|w| w == b"top secret")
        );

        let got = store.get_secret("cred").await.unwrap();
        assert_eq!(&got[..], b"top secret");

        // put_secret is an idempotent overwrite.
        store
            .put_secret("cred", bytes::Bytes::from_static(b"rotated"))
            .await
            .unwrap();
        assert_eq!(&store.get_secret("cred").await.unwrap()[..], b"rotated");

        store.delete_secret("cred").await.unwrap();
        assert!(matches!(
            store.get_secret("cred").await.unwrap_err(),
            Error::NotFound
        ));
    }

    #[tokio::test]
    async fn test_create_get_delete() {
        let store = test_store();
        let resource: Resource = Catalog {
            name: "new_catalog".into(),
            ..Default::default()
        }
        .into();
        let (created, reference) = store.create(resource.clone()).await.unwrap();
        assert_eq!(created.resource_name(), resource.resource_name());

        let ident = ObjectLabel::Catalog.to_ident(reference);
        let (retrieved, _) = store.get(&ident).await.unwrap();
        assert_eq!(retrieved, created);

        store.delete(&ident).await.unwrap();
        let result = store.get(&ident).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::NotFound));
    }

    #[tokio::test]
    async fn test_list() {
        let store = test_store();
        let resource: Resource = Catalog {
            name: "new_catalog".into(),
            ..Default::default()
        }
        .into();
        let (created, _) = store.create(resource.clone()).await.unwrap();

        let (resources, next) = store
            .list(&ObjectLabel::Catalog, None, None, None)
            .await
            .unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0], created);
        assert!(next.is_none());

        // add more resources
        let resource: Resource = Catalog {
            name: "new_catalog2".into(),
            ..Default::default()
        }
        .into();
        store.create(resource).await.unwrap();
        let resource: Resource = Catalog {
            name: "new_catalog3".into(),
            ..Default::default()
        }
        .into();
        store.create(resource).await.unwrap();

        let (resources, next) = store
            .list(&ObjectLabel::Catalog, None, Some(2), None)
            .await
            .unwrap();
        assert_eq!(resources.len(), 2);
        assert!(next.is_some());

        let (resources, next) = store
            .list(&ObjectLabel::Catalog, None, Some(2), next)
            .await
            .unwrap();
        assert_eq!(resources.len(), 1);
        assert!(next.is_none());
    }

    #[tokio::test]
    async fn test_provider_round_trip() {
        use unitycatalog_common::models::providers::v1::{Provider, ProviderAuthenticationType};

        let store = test_store();
        let resource: Resource = Provider {
            name: "acme".into(),
            authentication_type: ProviderAuthenticationType::Token as i32,
            comment: Some("inbound share from acme".into()),
            ..Default::default()
        }
        .into();

        // Create exercises the Resource::Provider -> Object conversion.
        let (created, reference) = store.create(resource.clone()).await.unwrap();
        assert_eq!(created.resource_name(), resource.resource_name());

        // Get exercises the Object -> Resource::Provider conversion and the
        // hand-written ObjectLabel::Provider -> ResourceIdent mapping.
        let ident = ObjectLabel::Provider.to_ident(reference);
        let (retrieved, _) = store.get(&ident).await.unwrap();
        assert_eq!(retrieved, created);
        let provider: Provider = retrieved.try_into().unwrap();
        assert_eq!(provider.name, "acme");
        assert_eq!(provider.comment.as_deref(), Some("inbound share from acme"));

        // List by the Provider label.
        let (resources, _) = store
            .list(&ObjectLabel::Provider, None, None, None)
            .await
            .unwrap();
        assert_eq!(resources.len(), 1);

        store.delete(&ident).await.unwrap();
        assert!(matches!(
            store.get(&ident).await.unwrap_err(),
            Error::NotFound
        ));
    }
}
