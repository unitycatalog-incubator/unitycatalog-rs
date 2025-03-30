// @generated
// This file is @generated by prost-build.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExternalLocationInfo {
    /// Name of the external location.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// Path URL of the external location.
    #[prost(string, tag="2")]
    pub url: ::prost::alloc::string::String,
    /// Name of the storage credential used with this location.
    #[prost(string, tag="3")]
    pub credential_name: ::prost::alloc::string::String,
    /// Indicates whether the external location is read-only.
    #[prost(bool, tag="4")]
    pub read_only: bool,
    /// User-provided free-form text description.
    #[prost(string, optional, tag="5")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
    /// The owner of the external location.
    #[prost(string, optional, tag="6")]
    pub owner: ::core::option::Option<::prost::alloc::string::String>,
    // metastore id
    // string metastore_id = 7;

    /// Unique ID of the location's storage credential.
    #[prost(string, tag="8")]
    pub credential_id: ::prost::alloc::string::String,
    /// Time at which this catalog was created, in epoch milliseconds.
    #[prost(int64, optional, tag="9")]
    pub created_at: ::core::option::Option<i64>,
    /// Username of catalog creator.
    #[prost(string, optional, tag="10")]
    pub created_by: ::core::option::Option<::prost::alloc::string::String>,
    /// Time at which this catalog was last updated, in epoch milliseconds.
    #[prost(int64, optional, tag="11")]
    pub updated_at: ::core::option::Option<i64>,
    /// Username of user who last modified catalog.
    #[prost(string, optional, tag="12")]
    pub updated_by: ::core::option::Option<::prost::alloc::string::String>,
    /// Indicates whether the principal is limited to retrieving metadata
    /// for the associated object through the BROWSE privilege when include_browse is enabled in the request.
    #[prost(bool, optional, tag="13")]
    pub browse_only: ::core::option::Option<bool>,
    #[prost(string, optional, tag="100")]
    pub external_location_id: ::core::option::Option<::prost::alloc::string::String>,
}
/// List external locations
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListExternalLocationsRequest {
    /// The maximum number of results per page that should be returned.
    #[prost(int32, optional, tag="2")]
    pub max_results: ::core::option::Option<i32>,
    /// Opaque pagination token to go to next page based on previous query.
    #[prost(string, optional, tag="3")]
    pub page_token: ::core::option::Option<::prost::alloc::string::String>,
    /// Whether to include schemas in the response for which the principal can only access selective metadata for
    #[prost(bool, optional, tag="4")]
    pub include_browse: ::core::option::Option<bool>,
}
/// List external locations response.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListExternalLocationsResponse {
    /// The external locations returned.
    #[prost(message, repeated, tag="1")]
    pub external_locations: ::prost::alloc::vec::Vec<ExternalLocationInfo>,
    /// The next_page_token value to include in the next List request.
    #[prost(string, optional, tag="2")]
    pub next_page_token: ::core::option::Option<::prost::alloc::string::String>,
}
/// Create a new external location
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateExternalLocationRequest {
    /// Name of external location.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// Path URL of the external location.
    #[prost(string, tag="2")]
    pub url: ::prost::alloc::string::String,
    /// Name of the storage credential used with this location.
    #[prost(string, tag="3")]
    pub credential_name: ::prost::alloc::string::String,
    /// Indicates whether the external location is read-only.
    #[prost(bool, optional, tag="4")]
    pub read_only: ::core::option::Option<bool>,
    /// User-provided free-form text description.
    #[prost(string, optional, tag="5")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
    /// Skips validation of the storage credential associated with the external location.
    #[prost(bool, optional, tag="6")]
    pub skip_validation: ::core::option::Option<bool>,
}
/// Get an external location
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetExternalLocationRequest {
    /// Name of external location.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
/// Update an external location
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateExternalLocationRequest {
    /// Name of external location.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// Path URL of the external location.
    #[prost(string, optional, tag="2")]
    pub url: ::core::option::Option<::prost::alloc::string::String>,
    /// Name of the storage credential used with this location.
    #[prost(string, optional, tag="3")]
    pub credential_name: ::core::option::Option<::prost::alloc::string::String>,
    /// Indicates whether the external location is read-only.
    #[prost(bool, optional, tag="4")]
    pub read_only: ::core::option::Option<bool>,
    /// owner of the external location.
    #[prost(string, optional, tag="5")]
    pub owner: ::core::option::Option<::prost::alloc::string::String>,
    /// User-provided free-form text description.
    #[prost(string, optional, tag="6")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
    /// new name of the external location.
    #[prost(string, optional, tag="7")]
    pub new_name: ::core::option::Option<::prost::alloc::string::String>,
    /// force update of the external location.
    #[prost(bool, optional, tag="8")]
    pub force: ::core::option::Option<bool>,
    /// Skips validation of the storage credential associated with the external location.
    #[prost(bool, optional, tag="9")]
    pub skip_validation: ::core::option::Option<bool>,
}
/// Delete an external location
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteExternalLocationRequest {
    /// Name of external location.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// Force deletion even if the external location is not empty.
    #[prost(bool, optional, tag="2")]
    pub force: ::core::option::Option<bool>,
}
include!("unitycatalog.external_locations.v1.serde.rs");
// @@protoc_insertion_point(module)