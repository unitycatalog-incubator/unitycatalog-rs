use crate::api::{RequestContext, SecuredAction, ShareHandler};
use crate::resources::{ResourceIdent, ResourceName, ResourceRef};
use crate::services::policy::Permission;

pub use self::models::*;
pub use super::codegen::{SharingClient, SharingHandler};
pub use handler::SharingQueryHandler;

mod handler;
mod models;

impl SecuredAction for GetShareRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}
impl SecuredAction for ListSharesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}
impl SecuredAction for ListSharingSchemasRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for ListShareTablesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for ListSchemaTablesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for QueryTableRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetTableVersionRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetTableMetadataRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}
