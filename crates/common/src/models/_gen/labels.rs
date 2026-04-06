// @generated — do not edit by hand.
/// All resource types managed by Unity Catalog.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Debug, PartialEq)]
pub enum Resource {
    Catalog(super::catalogs::v1::Catalog),
    Column(super::tables::v1::Column),
    Credential(super::credentials::v1::Credential),
    ExternalLocation(super::external_locations::v1::ExternalLocation),
    Function(super::functions::v1::Function),
    Recipient(super::recipients::v1::Recipient),
    Schema(super::schemas::v1::Schema),
    Share(super::shares::v1::Share),
    Table(super::tables::v1::Table),
    Volume(super::volumes::v1::Volume),
}
/// Discriminant label for each resource type.
#[derive(
    ::strum::AsRefStr,
    ::strum::Display,
    ::strum::EnumIter,
    ::strum::EnumString,
    ::serde::Serialize,
    ::serde::Deserialize,
    Hash,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "sqlx", derive(::sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "object_label", rename_all = "snake_case")
)]
pub enum ObjectLabel {
    Catalog,
    Column,
    Credential,
    ExternalLocation,
    Function,
    Recipient,
    Schema,
    Share,
    Table,
    Volume,
}
