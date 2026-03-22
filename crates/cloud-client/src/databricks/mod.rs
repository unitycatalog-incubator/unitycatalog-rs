//! Databricks Unified Auth implementation.
//!
//! Supports PAT, OAuth M2M (service principal client_credentials), notebook context auth,
//! Azure MSI fallback, and GCP SA token exchange — following the Databricks Unified Auth spec.
//!
//! # Auth resolution order (in [`DatabricksBuilder::build`])
//!
//! 1. Explicit `with_credentials(provider)` override.
//! 2. PAT / static token (`DATABRICKS_TOKEN` / `with_token()`).
//!    Also covers: notebook context auth (≥ 13.3 LTS injects `DATABRICKS_TOKEN`
//!    automatically) and self-hosted UC static tokens.
//! 3. OAuth M2M — `DATABRICKS_CLIENT_ID` + `DATABRICKS_CLIENT_SECRET` →
//!    POST `{host}/oidc/v1/token` with `grant_type=client_credentials`.
//! 4. Azure MSI fallback — if `DATABRICKS_AZURE_RESOURCE_ID` is set, uses
//!    Azure IMDS to obtain an Azure AD token for the Databricks App ID
//!    (`2ff814a6-3304-4ab8-85cb-cd0e6f879c1d`) and uses it directly as the
//!    Databricks bearer token.
//! 5. GCP SA token exchange — if `GOOGLE_APPLICATION_CREDENTIALS` is set,
//!    exchanges a GCP service-account JWT for a Databricks OIDC token via
//!    `{host}/oidc/v1/token` with
//!    `grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer`.

mod builder;
pub(crate) mod credential;

pub use builder::{DatabricksBuilder, DatabricksConfig, DatabricksConfigKey};
pub use credential::DatabricksCredential;
