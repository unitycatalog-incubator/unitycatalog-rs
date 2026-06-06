pub use codegen::UnityCatalogClient;
pub use codegen::catalogs::CatalogClient;
pub use codegen::credentials::CredentialClient;
pub use codegen::external_locations::ExternalLocationClient;
pub use codegen::functions::FunctionClient;
pub use codegen::providers::ProviderClient;
pub use codegen::recipients::RecipientClient;
pub use codegen::schemas::SchemaClient;
pub use codegen::shares::ShareClient;
pub use codegen::tables::TableClient;
pub use codegen::tag_policies::TagPolicyClient;
pub use codegen::volumes::VolumeClient;
pub use error::*;
pub use temporary_credentials::*;

pub mod codegen;
pub mod error;
mod temporary_credentials;

impl UnityCatalogClient {
    /// Ergonomic accessor for the temporary credential vending client.
    ///
    /// Wraps the generated low-level client with the hand-written name → UUID resolving helpers
    /// (`temporary_table_credential`, `temporary_volume_credential`, `temporary_path_credential`).
    pub fn temporary_credentials(&self) -> TemporaryCredentialClient {
        TemporaryCredentialClient::new(self.temporary_credentials_client())
    }
}
