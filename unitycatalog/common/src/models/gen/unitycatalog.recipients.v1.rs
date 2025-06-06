// @generated
// This file is @generated by prost-build.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RecipientToken {
    /// Unique ID of the recipient token.
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    /// Time at which this recipient token was created, in epoch milliseconds.
    #[prost(int64, tag="2")]
    pub created_at: i64,
    /// Username of recipient token creator.
    #[prost(string, tag="3")]
    pub created_by: ::prost::alloc::string::String,
    /// Full activation URL to retrieve the access token. It will be empty if the token is already retrieved.
    #[prost(string, tag="4")]
    pub activation_url: ::prost::alloc::string::String,
    /// Expiration timestamp of the token in epoch milliseconds.
    #[prost(int64, tag="5")]
    pub expiration_time: i64,
    /// Time at which this recipient token was updated, in epoch milliseconds.
    #[prost(int64, tag="6")]
    pub updated_at: i64,
    /// Username of recipient token updater.
    #[prost(string, tag="7")]
    pub updated_by: ::prost::alloc::string::String,
}
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RecipientInfo {
    /// Unique ID of the recipient.
    #[prost(string, optional, tag="100")]
    pub id: ::core::option::Option<::prost::alloc::string::String>,
    /// The name of the recipient.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// The delta sharing authentication type.
    #[prost(enumeration="AuthenticationType", tag="2")]
    pub authentication_type: i32,
    /// Username of the recipient owner.
    #[prost(string, tag="3")]
    pub owner: ::prost::alloc::string::String,
    /// Description about the recipient.
    #[prost(string, optional, tag="4")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
    /// A map of key-value properties attached to the securable.
    #[prost(message, optional, tag="6")]
    pub properties: ::core::option::Option<super::super::super::google::protobuf::Struct>,
    /// Time at which this share was created, in epoch milliseconds.
    #[prost(int64, optional, tag="7")]
    pub created_at: ::core::option::Option<i64>,
    /// Username of the creator of the share.
    #[prost(string, optional, tag="8")]
    pub created_by: ::core::option::Option<::prost::alloc::string::String>,
    /// This field is only present when the authentication_type is TOKEN.
    #[prost(message, repeated, tag="9")]
    pub tokens: ::prost::alloc::vec::Vec<RecipientToken>,
    /// Time at which this share was updated, in epoch milliseconds.
    #[prost(int64, optional, tag="10")]
    pub updated_at: ::core::option::Option<i64>,
    /// Username of share updater.
    #[prost(string, optional, tag="11")]
    pub updated_by: ::core::option::Option<::prost::alloc::string::String>,
}
#[cfg_attr(feature = "python", ::pyo3::pyclass)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum AuthenticationType {
    /// No authentication is required.
    Unspecified = 0,
    /// Basic authentication is required.
    Token = 1,
    /// OAuth2 authentication is required.
    OauthClientCredentials = 2,
}
impl AuthenticationType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            AuthenticationType::Unspecified => "AUTHENTICATION_TYPE_UNSPECIFIED",
            AuthenticationType::Token => "TOKEN",
            AuthenticationType::OauthClientCredentials => "OAUTH_CLIENT_CREDENTIALS",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "AUTHENTICATION_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "TOKEN" => Some(Self::Token),
            "OAUTH_CLIENT_CREDENTIALS" => Some(Self::OauthClientCredentials),
            _ => None,
        }
    }
}
/// Request to list recipients.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListRecipientsRequest {
    /// The maximum number of results per page that should be returned.
    #[prost(int32, optional, tag="1")]
    pub max_results: ::core::option::Option<i32>,
    /// Opaque pagination token to go to next page based on previous query.
    #[prost(string, optional, tag="2")]
    pub page_token: ::core::option::Option<::prost::alloc::string::String>,
}
/// Response to list recipients.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListRecipientsResponse {
    /// List of recipients.
    #[prost(message, repeated, tag="1")]
    pub recipients: ::prost::alloc::vec::Vec<RecipientInfo>,
    /// Opaque pagination token to go to next page based on previous query.
    #[prost(string, optional, tag="2")]
    pub next_page_token: ::core::option::Option<::prost::alloc::string::String>,
}
/// Creates a new recipient
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateRecipientRequest {
    /// Name of the recipient.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// The delta sharing authentication type.
    #[prost(enumeration="AuthenticationType", tag="2")]
    pub authentication_type: i32,
    /// Username of the recipient owner.
    #[prost(string, tag="3")]
    pub owner: ::prost::alloc::string::String,
    /// Description about the recipient.
    #[prost(string, optional, tag="4")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
    /// Recipient properties as map of string key-value pairs.
    ///
    /// When provided in update request, the specified properties will override the existing properties.
    /// To add and remove properties, one would need to perform a read-modify-write.
    #[prost(message, optional, tag="5")]
    pub properties: ::core::option::Option<super::super::super::google::protobuf::Struct>,
    /// Expiration timestamp of the token, in epoch milliseconds.
    #[prost(int64, optional, tag="6")]
    pub expiration_time: ::core::option::Option<i64>,
}
/// Get a recipient by name.
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetRecipientRequest {
    /// Name of the recipient.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
/// Update a recipient
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateRecipientRequest {
    /// Name of the recipient.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    /// New name for the recipient
    #[prost(string, optional, tag="2")]
    pub new_name: ::core::option::Option<::prost::alloc::string::String>,
    /// Username of the recipient owner.
    #[prost(string, optional, tag="3")]
    pub owner: ::core::option::Option<::prost::alloc::string::String>,
    /// Description about the recipient.
    #[prost(string, optional, tag="4")]
    pub comment: ::core::option::Option<::prost::alloc::string::String>,
    /// Recipient properties as map of string key-value pairs.
    ///
    /// When provided in update request, the specified properties will override the existing properties.
    /// To add and remove properties, one would need to perform a read-modify-write.
    #[prost(message, optional, tag="5")]
    pub properties: ::core::option::Option<super::super::super::google::protobuf::Struct>,
    /// Expiration timestamp of the token, in epoch milliseconds.
    #[prost(int64, optional, tag="6")]
    pub expiration_time: ::core::option::Option<i64>,
}
/// Delete a recipient
#[cfg_attr(feature = "python", ::pyo3::pyclass(get_all, set_all))]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteRecipientRequest {
    /// Name of the recipient.
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
}
include!("unitycatalog.recipients.v1.serde.rs");
// @@protoc_insertion_point(module)