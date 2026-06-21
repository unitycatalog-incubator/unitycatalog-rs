use itertools::Itertools;
use unitycatalog_common::models::ResourceName;
use unitycatalog_common::models::catalogs::v1::Catalog;
use unitycatalog_common::models::schemas::v1::Schema;
use unitycatalog_common::models::staging_tables::v1::*;
use unitycatalog_common::models::{ObjectLabel, ResourceIdent};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::staging_tables::StagingTableHandler;
use crate::policy::{Permission, Policy, Principal};
use crate::services::ProvidesLocalStoragePolicy;
use crate::services::location::StorageLocationUrl;
use crate::store::ResourceStore;
use crate::{Error, Result};

/// Managed-storage prefix appended to a catalog/schema storage root. Mirrors the
/// Unity Catalog reference implementation's `__unitystorage` segment. A
/// catalog/schema location that already carries the prefix is used as-is.
const MANAGED_STORAGE_PREFIX: &str = "__unitystorage";

impl SecuredAction for CreateStagingTableRequest {
    fn resource(&self) -> ResourceIdent {
        // Creating a staging table requires CREATE on the parent schema.
        ResourceIdent::schema(ResourceName::new([
            self.catalog_name.as_str(),
            self.schema_name.as_str(),
        ]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext> + ProvidesLocalStoragePolicy>
    StagingTableHandler<RequestContext> for T
{
    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_staging_table(
        &self,
        request: CreateStagingTableRequest,
        context: RequestContext,
    ) -> Result<StagingTable> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;

        // A managed table cannot share a name with an existing table in the schema.
        let table_ident = ResourceIdent::table(ResourceName::new([
            request.catalog_name.as_str(),
            request.schema_name.as_str(),
            request.name.as_str(),
        ]));
        if self.get(&table_ident).await.is_ok() {
            return Err(Error::AlreadyExists);
        }

        // Allocate the immutable id and location. The id is the future managed
        // table's id; the store persists the row under this same id.
        let id = uuid::Uuid::new_v4();
        let root =
            resolve_managed_storage_root(self, &request.catalog_name, &request.schema_name).await?;
        let staging_location = format!("{}/tables/{}", root.trim_end_matches('/'), id);

        let created_by = match context.recipient() {
            Principal::User(name) => Some(name.clone()),
            Principal::Anonymous => None,
        };

        let staging_table = StagingTable {
            id: id.hyphenated().to_string(),
            name: request.name,
            catalog_name: request.catalog_name,
            schema_name: request.schema_name,
            staging_location,
            created_by,
            stage_committed: false,
            created_at: None,
        };

        Ok(self.create(staging_table.into()).await?.0.try_into()?)
    }
}

/// Resolve the managed storage root for a table created in `catalog.schema`.
///
/// Resolution order mirrors the Unity Catalog reference implementation: the
/// schema's `storage_location` if set, otherwise the catalog's `storage_root`.
/// If a root carries no managed-storage prefix yet, `__unitystorage` is
/// appended. A catalog/schema with no configured root cannot host managed
/// tables, so this returns `invalid_argument`.
pub(crate) async fn resolve_managed_storage_root(
    handler: &(impl ResourceStore + ProvidesLocalStoragePolicy + ?Sized),
    catalog_name: &str,
    schema_name: &str,
) -> Result<String> {
    // Validate a resolved root against the local-storage allowlist before
    // handing out a managed sub-path. Defense-in-depth: even a root that
    // predates the policy (or a cloud root, which passes through) is checked at
    // use time. Cloud schemes are unaffected.
    let checked = |root: String| -> Result<String> {
        handler
            .local_storage_policy()
            .check(&StorageLocationUrl::parse(&root)?)?;
        Ok(with_managed_prefix(&root))
    };

    let schema_ident = ResourceIdent::schema(ResourceName::new([catalog_name, schema_name]));
    let schema: Schema = handler.get(&schema_ident).await?.0.try_into()?;
    if let Some(loc) = schema.storage_location.filter(|s| !s.is_empty()) {
        return checked(loc);
    }

    let catalog_ident = ResourceIdent::catalog(ResourceName::new([catalog_name]));
    let catalog: Catalog = handler.get(&catalog_ident).await?.0.try_into()?;
    if let Some(root) = catalog.storage_root.filter(|s| !s.is_empty()) {
        return checked(root);
    }

    // Neither schema nor catalog defines a root. When the server allows exactly
    // one local root, use it as the implicit managed root so a purely-local dev
    // server hosts managed tables without any storage_root configured. The
    // returned value is a *root*; the caller appends the standard
    // `__unitystorage/tables/{uuid}` convention (see `with_managed_prefix` and
    // the caller in `create_staging_table`), so the on-disk layout matches every
    // other managed table. More than one allowed root is ambiguous → keep the
    // original error.
    let allowed = handler.local_storage_policy().allowed_roots();
    if let [sole_root] = allowed {
        let root = url::Url::from_directory_path(sole_root)
            .map_err(|_| {
                Error::invalid_argument(format!(
                    "allowed local storage root '{}' is not an absolute path",
                    sole_root.display()
                ))
            })?
            .to_string();
        return checked(root);
    }

    Err(Error::invalid_argument(format!(
        "neither schema '{catalog_name}.{schema_name}' nor catalog '{catalog_name}' has a managed \
         storage root configured; cannot create a managed table"
    )))
}

/// Find the staging table reserved at `staging_location`, if any.
///
/// Staging rows are keyed by their (immutable) location; `CreateStagingTable`
/// guarantees uniqueness via a fresh UUID per location. Returns `NotFound` if no
/// staging table was allocated there.
///
/// TODO(perf): this lists all staging tables; once the store supports property
/// filters, query by `staging_location` directly (mirrors the existing TODOs on
/// `list_external_locations` / `list_table_volume_locations`).
pub(crate) async fn find_staging_table_by_location(
    handler: &(impl ResourceStore + ?Sized),
    staging_location: &str,
) -> Result<StagingTable> {
    let (resources, _) = handler
        .list(&ObjectLabel::StagingTable, None, None, None)
        .await?;
    resources
        .into_iter()
        .map(StagingTable::try_from)
        .filter_ok(|st| st.staging_location == staging_location)
        .next()
        .ok_or(Error::NotFound)?
        .map_err(Error::from)
}

/// Append the managed-storage prefix to `root` unless it already ends with it.
fn with_managed_prefix(root: &str) -> String {
    let trimmed = root.trim_end_matches('/');
    if trimmed.ends_with(MANAGED_STORAGE_PREFIX) {
        trimmed.to_string()
    } else {
        format!("{trimmed}/{MANAGED_STORAGE_PREFIX}")
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::catalogs::v1::CreateCatalogRequest;
    use unitycatalog_common::models::schemas::v1::CreateSchemaRequest;
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::api::{CatalogHandler, SchemaHandler};
    use crate::memory::InMemoryResourceStore;
    use crate::policy::ConstantPolicy;
    use crate::services::ServerHandler;

    fn handler() -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        ServerHandler::try_new_tokio(policy, store.clone(), store).unwrap()
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: Principal::anonymous(),
        }
    }

    async fn make_catalog(
        h: &ServerHandler<RequestContext>,
        name: &str,
        storage_root: Option<&str>,
    ) {
        h.create_catalog(
            CreateCatalogRequest {
                name: name.to_string(),
                storage_root: storage_root.map(str::to_string),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    async fn make_schema(
        h: &ServerHandler<RequestContext>,
        catalog: &str,
        name: &str,
        storage_location: Option<&str>,
    ) {
        h.create_schema(
            CreateSchemaRequest {
                name: name.to_string(),
                catalog_name: catalog.to_string(),
                storage_location: storage_location.map(str::to_string),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
    }

    fn create_req(catalog: &str, schema: &str, name: &str) -> CreateStagingTableRequest {
        CreateStagingTableRequest {
            name: name.to_string(),
            catalog_name: catalog.to_string(),
            schema_name: schema.to_string(),
        }
    }

    #[test]
    fn managed_prefix_is_not_doubled() {
        assert_eq!(
            with_managed_prefix("s3://b/root"),
            "s3://b/root/__unitystorage"
        );
        assert_eq!(
            with_managed_prefix("s3://b/root/"),
            "s3://b/root/__unitystorage"
        );
        assert_eq!(
            with_managed_prefix("s3://b/root/__unitystorage"),
            "s3://b/root/__unitystorage"
        );
    }

    #[tokio::test]
    async fn staging_location_derives_from_catalog_root() {
        let h = handler();
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        make_schema(&h, "cat", "sch", None).await;

        let st = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await
            .unwrap();

        assert!(!st.stage_committed);
        // {root}/__unitystorage/tables/{uuid}
        let prefix = "s3://bucket/cat/__unitystorage/tables/";
        assert!(
            st.staging_location.starts_with(prefix),
            "location {} should start with {prefix}",
            st.staging_location
        );
        // The trailing segment is the generated id.
        assert_eq!(
            st.staging_location,
            format!("{prefix}{}", st.id),
            "location should end with the staging id"
        );
    }

    #[tokio::test]
    async fn schema_storage_location_takes_precedence_over_catalog() {
        let h = handler();
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        make_schema(&h, "cat", "sch", Some("s3://other/sch")).await;

        let st = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await
            .unwrap();

        assert!(
            st.staging_location
                .starts_with("s3://other/sch/__unitystorage/tables/"),
            "got {}",
            st.staging_location
        );
    }

    #[tokio::test]
    async fn missing_storage_root_is_rejected() {
        let h = handler();
        make_catalog(&h, "cat", None).await;
        make_schema(&h, "cat", "sch", None).await;

        let res = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await;
        assert!(matches!(res, Err(Error::InvalidArgument(_))), "{res:?}");
    }

    #[tokio::test]
    async fn find_by_location_round_trips() {
        let h = handler();
        make_catalog(&h, "cat", Some("s3://bucket/cat")).await;
        make_schema(&h, "cat", "sch", None).await;

        let st = h
            .create_staging_table(create_req("cat", "sch", "tbl"), ctx())
            .await
            .unwrap();

        let found = find_staging_table_by_location(&h, &st.staging_location)
            .await
            .unwrap();
        assert_eq!(found.id, st.id);
        assert_eq!(found.name, "tbl");

        let missing = find_staging_table_by_location(&h, "s3://bucket/cat/tables/nope").await;
        assert!(matches!(missing, Err(Error::NotFound)), "{missing:?}");
    }
}
