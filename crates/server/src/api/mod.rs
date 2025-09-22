pub use catalogs::CatalogHandler;
pub use credentials::CredentialHandler;
pub use external_locations::ExternalLocationHandler;
pub use recipients::RecipientHandler;
pub use schemas::SchemaHandler;
pub use shares::ShareHandler;
pub use tables::TableHandler;
pub use temporary_credentials::TemporaryCredentialHandler;
pub use volumes::VolumeHandler;

use crate::policy::{Permission, Principal};
use unitycatalog_common::models::ResourceIdent;

pub mod catalogs;
pub mod credentials;
pub mod external_locations;
pub mod recipients;
pub mod schemas;
pub mod shares;
pub mod sharing;
pub mod tables;
pub mod temporary_credentials;
pub mod volumes;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub recipient: Principal,
}

impl RequestContext {
    pub fn recipient(&self) -> &Principal {
        &self.recipient
    }
}

impl AsRef<Principal> for RequestContext {
    fn as_ref(&self) -> &Principal {
        &self.recipient
    }
}

pub trait SecuredAction: Send + Sync {
    /// The resource that the action is performed on.
    fn resource(&self) -> ResourceIdent;

    /// The permission required to perform the action.
    fn permission(&self) -> &'static Permission;
}
