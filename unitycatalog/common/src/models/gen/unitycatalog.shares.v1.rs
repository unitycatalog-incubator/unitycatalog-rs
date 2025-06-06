// @generated
// This file is @generated by prost-build.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataObject {
    /// A fully qualified name that uniquely identifies a data object.
    ///
    /// For example, a table's fully qualified name is in the format of <catalog>.<schema>.<table>,
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// Type of the data object.
    #[prost(enumeration="DataObjectType", tag="2")]
    pub data_object_type: i32,
    /// The time when this data object is added to the share, in epoch milliseconds.
    #[prost(int64, optional, tag="3")]
    pub added_at: ::core::option::Option<i64>,
    /// Username of the sharer.
    #[prost(string, optional, tag="4")]
    pub added_by: ::core::option::Option<::prost::alloc::string::String>,
    /// A user-provided comment when adding the data object to the share.
    #[prost(string, optional, tag="5")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
    /// A user-provided new name for the data object within the share.
    ///
    /// If this new name is not provided, the object's original name will be used as the shared_as name.
    /// The shared_as name must be unique within a share.
    /// For tables, the new name must follow the format of <schema>.<table>.
    #[prost(string, optional, tag="6")]
    pub shared_as: ::core::option::Option<::prost::alloc::string::String>,
    /// Array of partitions for the shared data.
    #[prost(string, repeated, tag="7")]
    pub partitions: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Whether to enable cdf or indicate if cdf is enabled on the shared object.
    #[prost(bool, optional, tag="8")]
    pub enable_cdf: ::core::option::Option<bool>,
    /// Whether to enable or disable sharing of data history. If not specified, the default is DISABLED.
    #[prost(enumeration="HistoryStatus", optional, tag="9")]
    pub history_data_sharing_status: ::core::option::Option<i32>,
    /// The start version associated with the object.
    ///
    /// This allows data providers to control the lowest object version that is accessible by clients.
    /// If specified, clients can query snapshots or changes for versions >= start_version.
    /// If not specified, clients can only query starting from the version of the object at the time it was added to the share.
    ///
    /// NOTE: The start_version should be <= the current version of the object.
    #[prost(int64, optional, tag="10")]
    pub start_version: ::core::option::Option<i64>,
}
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShareInfo {
    /// Unique ID of the recipient.
    #[prost(string, optional, tag="100")]
    pub id: ::core::option::Option<::prost::alloc::string::String>,
    /// Name of the share.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// Username of current owner of share.
    #[prost(string, optional, tag="2")]
    pub owner: ::core::option::Option<::prost::alloc::string::String>,
    /// User-provided free-form text description.
    #[prost(string, optional, tag="3")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
    // Storage root URL for the share.
    // optional string storage_root = 4;

    /// A list of shared data objects within the share.
    #[prost(message, repeated, tag="5")]
    pub data_objects: ::prost::alloc::vec::Vec<DataObject>,
    /// Time at which this share was created, in epoch milliseconds.
    #[prost(int64, optional, tag="6")]
    pub created_at: ::core::option::Option<i64>,
    /// Username of the creator of the share.
    #[prost(string, optional, tag="7")]
    pub created_by: ::core::option::Option<::prost::alloc::string::String>,
    /// Time at which this share was updated, in epoch milliseconds.
    #[prost(int64, optional, tag="8")]
    pub updated_at: ::core::option::Option<i64>,
    /// Username of share updater.
    #[prost(string, optional, tag="9")]
    pub updated_by: ::core::option::Option<::prost::alloc::string::String>,
}
#[cfg_attr(feature = "python", ::pyo3::pyclass)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DataObjectType {
    /// Unknown data object type.
    Unspecified = 0,
    Table = 1,
    Schema = 2,
}
impl DataObjectType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DataObjectType::Unspecified => "DATA_OBJECT_TYPE_UNSPECIFIED",
            DataObjectType::Table => "TABLE",
            DataObjectType::Schema => "SCHEMA",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "DATA_OBJECT_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "TABLE" => Some(Self::Table),
            "SCHEMA" => Some(Self::Schema),
            _ => None,
        }
    }
}
#[cfg_attr(feature = "python", ::pyo3::pyclass)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum HistoryStatus {
    /// Data history sharing is disabled.
    Disabled = 0,
    /// Data history sharing is enabled.
    Enabled = 1,
}
impl HistoryStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            HistoryStatus::Disabled => "DISABLED",
            HistoryStatus::Enabled => "ENABLED",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "DISABLED" => Some(Self::Disabled),
            "ENABLED" => Some(Self::Enabled),
            _ => None,
        }
    }
}
/// Request to list shares.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListSharesRequest {
    /// The maximum number of results per page that should be returned.
    #[prost(int32, optional, tag="1")]
    pub max_results: ::core::option::Option<i32>,
    /// Opaque pagination token to go to next page based on previous query.
    #[prost(string, optional, tag="2")]
    pub page_token: ::core::option::Option<::prost::alloc::string::String>,
}
/// Response to list shares.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListSharesResponse {
    /// List of shares.
    #[prost(message, repeated, tag="1")]
    pub shares: ::prost::alloc::vec::Vec<ShareInfo>,
    /// Opaque pagination token to go to next page based on previous query.
    #[prost(string, optional, tag="2")]
    pub next_page_token: ::core::option::Option<::prost::alloc::string::String>,
}
/// Creates a new share for data objects.
///
/// Data objects can be added after creation with update.
/// The caller must be a metastore admin or have the CREATE_SHARE privilege on the metastore.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateShareRequest {
    /// Name of the share.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// User-provided free-form text description.
    #[prost(string, optional, tag="2")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
}
/// Get a share by name.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetShareRequest {
    /// Name of the share.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// Query for data to include in the share.
    #[prost(bool, optional, tag="2")]
    pub include_shared_data: ::core::option::Option<bool>,
}
/// Data object update.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DataObjectUpdate {
    /// Name of the share.
    #[prost(enumeration="Action", tag="1")]
    pub action: i32,
    /// User-provided free-form text description.
    #[prost(message, optional, tag="2")]
    pub data_object: ::core::option::Option<DataObject>,
}
/// Update a share.
///
/// The caller must be a metastore admin or have the UPDATE_SHARE privilege on the metastore.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateShareRequest {
    /// Name of the share.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// Array of shared data object updates.
    #[prost(message, repeated, tag="2")]
    pub updates: ::prost::alloc::vec::Vec<DataObjectUpdate>,
    /// A new name for the share.
    #[prost(string, optional, tag="3")]
    pub new_name: ::core::option::Option<::prost::alloc::string::String>,
    /// Owner of the share.
    #[prost(string, optional, tag="4")]
    pub owner: ::core::option::Option<::prost::alloc::string::String>,
    /// User-provided free-form text description.
    #[prost(string, optional, tag="5")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
}
/// Delete a share.
///
/// The caller must be a metastore admin or have the DELETE_SHARE privilege on the metastore.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteShareRequest {
    /// Name of the share.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
#[cfg_attr(feature = "python", ::pyo3::pyclass)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Action {
    /// Unspecified action.
    Unspecified = 0,
    Add = 1,
    Remove = 2,
    Update = 3,
}
impl Action {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Action::Unspecified => "ACTION_UNSPECIFIED",
            Action::Add => "ADD",
            Action::Remove => "REMOVE",
            Action::Update => "UPDATE",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ACTION_UNSPECIFIED" => Some(Self::Unspecified),
            "ADD" => Some(Self::Add),
            "REMOVE" => Some(Self::Remove),
            "UPDATE" => Some(Self::Update),
            _ => None,
        }
    }
}
include!("unitycatalog.shares.v1.serde.rs");
// @@protoc_insertion_point(module)