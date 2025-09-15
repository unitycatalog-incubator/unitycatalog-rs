use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};
use unitycatalog_sharing_client::models::sharing::v1::*;
pub use unitycatalog_sharing_client::models::*;

pub use self::handler::*;
use crate::api::SecuredAction;
use crate::policy::Permission;

mod handler;

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
impl SecuredAction for ListSchemasRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.share.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for ListAllTablesRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::share(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for ListTablesRequest {
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
