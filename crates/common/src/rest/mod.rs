#[cfg(feature = "axum")]
pub use auth::*;
#[cfg(feature = "axum")]
pub use routers::*;

#[cfg(feature = "axum")]
mod routers {
    pub use crate::codegen::catalogs::create_router as get_catalog_router;
    pub use crate::codegen::credentials::create_router as get_credentials_router;
    pub use crate::codegen::external_locations::create_router as get_external_locations_router;
    pub use crate::codegen::recipients::create_router as get_recipients_router;
    pub use crate::codegen::schemas::create_router as get_schemas_router;
    pub use crate::codegen::shares::create_router as get_shares_router;
    pub use crate::codegen::tables::create_router as get_tables_router;
    pub use crate::sharing::get_router as get_sharing_router;
}

#[cfg(feature = "axum")]
mod auth;
#[cfg(feature = "rest-client")]
pub mod client;
#[cfg(any(all(test, feature = "axum"), feature = "integration"))]
pub mod integration;

#[cfg(all(test, feature = "axum"))]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::memory::InMemoryResourceStore;
    use crate::rest::auth::{AnonymousAuthenticator, AuthenticationLayer};
    use crate::services::policy::{ConstantPolicy, Policy, ProvidesPolicy};
    use crate::services::secrets::{ProvidesSecretManager, SecretManager};
    use crate::{ProvidesResourceStore, ResourceStore};

    #[derive(Clone)]
    struct Handler {
        store: InMemoryResourceStore,
        policy: Arc<dyn Policy>,
    }

    impl Default for Handler {
        fn default() -> Self {
            Self {
                store: InMemoryResourceStore::new(),
                policy: Arc::new(ConstantPolicy::default()),
            }
        }
    }

    impl ProvidesResourceStore for Handler {
        fn store(&self) -> &dyn ResourceStore {
            &self.store
        }
    }

    impl ProvidesPolicy for Handler {
        fn policy(&self) -> &Arc<dyn Policy> {
            &self.policy
        }
    }

    impl ProvidesSecretManager for Handler {
        fn secret_manager(&self) -> &dyn SecretManager {
            &self.store
        }
    }

    #[tokio::test]
    async fn test_catalog_router() {
        let handler = Handler::default();
        let app = get_catalog_router(handler.clone())
            .merge(get_schemas_router(handler))
            .layer(AuthenticationLayer::new(AnonymousAuthenticator));
        super::integration::test_catalog_router(app).await;
    }

    #[tokio::test]
    async fn test_credentials_router() {
        let handler = Handler::default();
        let app = get_credentials_router(handler.clone())
            .merge(get_external_locations_router(handler))
            .layer(AuthenticationLayer::new(AnonymousAuthenticator));
        super::integration::test_credentials_router(app).await;
    }
}
