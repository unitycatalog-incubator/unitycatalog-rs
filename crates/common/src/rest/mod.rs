#[cfg(feature = "axum")]
pub use auth::*;
#[cfg(feature = "axum")]
pub use routers::*;

#[cfg(feature = "axum")]
mod routers {
    pub use super::catalogs::get_router as get_catalog_router;
    pub use super::credentials::get_router as get_credentials_router;
    pub use super::external_locations::get_router as get_external_locations_router;
    pub use super::recipients::get_router as get_recipients_router;
    pub use super::schemas::get_router as get_schemas_router;
    pub use super::shares::get_router as get_shares_router;
    pub use super::tables::get_router as get_tables_router;
    pub use crate::sharing::get_router as get_sharing_router;
}

#[cfg(feature = "axum")]
mod auth;
#[cfg(feature = "rest-client")]
pub mod client;
#[cfg(any(all(test, feature = "axum"), feature = "integration"))]
pub mod integration;

#[cfg(feature = "axum")]
mod catalogs {
    use crate::api::catalogs::*;
    use axum::routing::{Router, delete, get, patch, post};

    /// Create a new [Router] for the UC Catalogs API.
    pub fn get_router<T: CatalogHandler + Clone>(handler: T) -> Router {
        Router::new()
            .route("/catalogs", post(create_catalog::<T>))
            .route("/catalogs", get(list_catalogs::<T>))
            .route("/catalogs/{name}", get(get_catalog::<T>))
            .route("/catalogs/{name}", patch(update_catalog::<T>))
            .route("/catalogs/{name}", delete(delete_catalog::<T>))
            .with_state(handler)
    }
}

#[cfg(feature = "axum")]
mod schemas {
    use crate::api::schemas::*;
    use axum::routing::{Router, delete, get, patch, post};

    /// Create a new [Router] for the UC Schemas API.
    pub fn get_router<T: SchemasHandler + Clone>(handler: T) -> Router {
        Router::new()
            .route("/schemas", post(create_schema::<T>))
            .route("/schemas", get(list_schemas::<T>))
            .route("/schemas/{name}", get(get_schema::<T>))
            .route("/schemas/{name}", patch(update_schema::<T>))
            .route("/schemas/{name}", delete(delete_schema::<T>))
            .with_state(handler)
    }
}

#[cfg(feature = "axum")]
mod recipients {
    use crate::api::recipients::*;
    use axum::routing::{Router, delete, get, patch, post};

    /// Create a new [Router] for the UC Recipients API.
    pub fn get_router<T: RecipientsHandler + Clone>(handler: T) -> Router {
        Router::new()
            .route("/recipients", post(create_recipient::<T>))
            .route("/recipients", get(list_recipients::<T>))
            .route("/recipients/{name}", get(get_recipient::<T>))
            .route("/recipients/{name}", patch(update_recipient::<T>))
            .route("/recipients/{name}", delete(delete_recipient::<T>))
            .with_state(handler)
    }
}

#[cfg(feature = "axum")]
mod credentials {
    use crate::api::credentials::*;
    use axum::routing::{Router, delete, get, post};

    /// Create a new [Router] for the UC Credentials API.
    pub fn get_router<T: CredentialsHandler + Clone>(handler: T) -> Router {
        Router::new()
            .route("/credentials", get(list_credentials::<T>))
            .route("/credentials", post(create_credential::<T>))
            .route("/credentials/{name}", get(get_credential::<T>))
            //.route("/credentials/{name}", patch(update_credential::<T>))
            .route("/credentials/{name}", delete(delete_credential::<T>))
            .with_state(handler)
    }
}

#[cfg(feature = "axum")]
mod external_locations {
    use crate::api::external_locations::*;
    use axum::routing::{Router, delete, get, patch, post};

    /// Create a new [Router] for the UC External Locations API.
    pub fn get_router<T: ExternalLocationsHandler + Clone>(handler: T) -> Router {
        Router::new()
            .route("/external-locations", post(create_external_location::<T>))
            .route("/external-locations", get(list_external_locations::<T>))
            .route(
                "/external-locations/{name}",
                get(get_external_location::<T>),
            )
            .route(
                "/external-locations/{name}",
                patch(update_external_location::<T>),
            )
            .route(
                "/external-locations/{name}",
                delete(delete_external_location::<T>),
            )
            .with_state(handler)
    }
}

#[cfg(feature = "axum")]
mod shares {
    use crate::api::shares::*;
    use axum::routing::{Router, delete, get, patch, post};

    /// Create a new [Router] for the UC Shares API.
    pub fn get_router<T: SharesHandler + Clone>(handler: T) -> Router {
        Router::new()
            .route("/shares", get(list_shares::<T>))
            .route("/shares", post(create_share::<T>))
            .route("/shares/{name}", get(get_share::<T>))
            .route("/shares/{name}", patch(update_share::<T>))
            .route("/shares/{name}", delete(delete_share::<T>))
            .with_state(handler)
    }
}

#[cfg(feature = "axum")]
mod tables {
    use crate::api::tables::*;
    use axum::routing::{Router, delete, get, post};

    /// Create a new [Router] for the UC Tables API.
    pub fn get_router<T: TablesHandler + Clone>(handler: T) -> Router {
        Router::new()
            .route("/tables", post(create_table::<T>))
            .route("/tables", get(list_tables::<T>))
            .route("/tables/{name}", get(get_table::<T>))
            // .route("/tables/{name}", patch(update_table::<T>))
            .route("/tables/{name}", delete(delete_table::<T>))
            .with_state(handler)
    }
}

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
