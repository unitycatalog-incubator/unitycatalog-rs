use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use chrono::{DateTime, SecondsFormat, Utc};
use reqwest::{Method, header::AUTHORIZATION};
use serde::Deserialize;

use crate::retry::RetryExt;
use crate::service::{HttpService, make_service};
use crate::util::hmac_sha256;
use crate::{ClientOptions, Result, RetryConfig};

/// SAS signed version — must be at least 2018-11-09 to support user delegation SAS.
const SAS_VERSION: &str = "2020-12-06";

/// Signed resource type: "b" = blob (covers individual blobs under the prefix).
const SAS_SIGNED_RESOURCE: &str = "c"; // container-level so prefix matching works

// ── User Delegation Key ──────────────────────────────────────────────────────

/// Response from the Azure Storage `Get User Delegation Key` endpoint.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct UserDelegationKey {
    pub signed_oid: String,
    pub signed_tid: String,
    pub signed_start: String,
    pub signed_expiry: String,
    pub signed_service: String,
    pub signed_version: String,
    pub value: String,
}

/// Fetch a User Delegation Key from Azure Storage using an AAD bearer token.
///
/// The key is valid for the given duration and can be used to sign SAS tokens
/// scoped to the storage account.
pub(crate) async fn fetch_user_delegation_key(
    account: &str,
    bearer_token: &str,
    start: DateTime<Utc>,
    expiry: DateTime<Utc>,
) -> Result<UserDelegationKey> {
    let url =
        format!("https://{account}.blob.core.windows.net/?restype=service&comp=userdelegationkey");
    let body = format!(
        "<KeyInfo><Start>{}</Start><Expiry>{}</Expiry></KeyInfo>",
        start.to_rfc3339_opts(SecondsFormat::Secs, true),
        expiry.to_rfc3339_opts(SecondsFormat::Secs, true),
    );

    let client = ClientOptions::default().client()?;
    let service: Arc<dyn HttpService> = make_service(client.clone(), None);
    let retry = RetryConfig::default();

    let text = client
        .request(Method::POST, &url)
        .header(AUTHORIZATION, format!("Bearer {bearer_token}"))
        .header("x-ms-version", SAS_VERSION)
        .header(reqwest::header::CONTENT_TYPE, "application/xml")
        .body(body)
        .retryable(&retry, service)
        .idempotent(true)
        .send()
        .await
        .map_err(|e| crate::Error::Generic {
            source: Box::new(e),
        })?
        .text()
        .await
        .map_err(|e| crate::Error::Generic {
            source: Box::new(e),
        })?;

    quick_xml::de::from_str::<UserDelegationKey>(&text).map_err(|e| crate::Error::Generic {
        source: e.to_string().into(),
    })
}

// ── SAS construction ─────────────────────────────────────────────────────────

/// Build a container-level SAS token query string signed with a User Delegation Key.
///
/// Returns the SAS parameters as a query string (without leading `?`).
pub(crate) fn build_user_delegation_sas(
    account: &str,
    container: &str,
    key: &UserDelegationKey,
    expiry: DateTime<Utc>,
    permissions: &str,
) -> Result<String> {
    let start = Utc::now();
    let start_str = start.to_rfc3339_opts(SecondsFormat::Secs, true);
    let expiry_str = expiry.to_rfc3339_opts(SecondsFormat::Secs, true);

    // String-to-sign for User Delegation SAS (container resource).
    // https://learn.microsoft.com/en-us/rest/api/storageservices/create-user-delegation-sas#version-2020-12-06-and-later
    let string_to_sign = format!(
        "{permissions}\n{start_str}\n{expiry_str}\n/blob/{account}/{container}\n{signed_oid}\n{signed_tid}\n{signed_start}\n{signed_expiry}\n{signed_service}\n{signed_version}\n\nhttps\n{version}\n{resource}\n\n\n\n\n\n",
        permissions = permissions,
        start_str = start_str,
        expiry_str = expiry_str,
        account = account,
        container = container,
        signed_oid = key.signed_oid,
        signed_tid = key.signed_tid,
        signed_start = key.signed_start,
        signed_expiry = key.signed_expiry,
        signed_service = key.signed_service,
        signed_version = key.signed_version,
        version = SAS_VERSION,
        resource = SAS_SIGNED_RESOURCE,
    );

    let key_bytes = BASE64
        .decode(&key.value)
        .map_err(|e| crate::Error::Generic { source: e.into() })?;
    let signature = BASE64.encode(hmac_sha256(&key_bytes, string_to_sign.as_bytes()).as_ref());

    let sas = format!(
        "sv={version}&se={expiry}&sp={permissions}&spr=https&sr={resource}\
         &skoid={skoid}&sktid={sktid}&skt={skt}&ske={ske}&sks={sks}&skv={skv}\
         &sig={sig}",
        version = SAS_VERSION,
        expiry = url_encode(&expiry_str),
        permissions = permissions,
        resource = SAS_SIGNED_RESOURCE,
        skoid = key.signed_oid,
        sktid = key.signed_tid,
        skt = url_encode(&key.signed_start),
        ske = url_encode(&key.signed_expiry),
        sks = key.signed_service,
        skv = key.signed_version,
        sig = url_encode(&signature),
    );
    Ok(sas)
}

/// Build a container-level SAS token signed with a storage account key.
///
/// Uses the Shared Key Lite signing algorithm.
/// <https://learn.microsoft.com/en-us/rest/api/storageservices/create-service-sas>
pub(crate) fn build_storage_key_sas(
    account: &str,
    container: &str,
    account_key_b64: &str,
    expiry: DateTime<Utc>,
    permissions: &str,
) -> Result<String> {
    let start = Utc::now();
    let start_str = start.to_rfc3339_opts(SecondsFormat::Secs, true);
    let expiry_str = expiry.to_rfc3339_opts(SecondsFormat::Secs, true);
    let canonicalized_resource = format!("/blob/{account}/{container}");

    // String-to-sign for service SAS (container), API version 2020-12-06.
    // https://learn.microsoft.com/en-us/rest/api/storageservices/create-service-sas#version-2020-12-06-and-later
    let string_to_sign = format!(
        "{permissions}\n{start_str}\n{expiry_str}\n{resource}\n\nhttps\n{version}\n{resource_type}\n\n\n\n\n\n",
        permissions = permissions,
        start_str = start_str,
        expiry_str = expiry_str,
        resource = canonicalized_resource,
        version = SAS_VERSION,
        resource_type = SAS_SIGNED_RESOURCE,
    );

    let key_bytes = BASE64
        .decode(account_key_b64)
        .map_err(|e| crate::Error::Generic { source: e.into() })?;
    let signature = BASE64.encode(hmac_sha256(&key_bytes, string_to_sign.as_bytes()).as_ref());

    let sas = format!(
        "sv={version}&se={expiry}&sp={permissions}&spr=https&sr={resource}&sig={sig}",
        version = SAS_VERSION,
        expiry = url_encode(&expiry_str),
        permissions = permissions,
        resource = SAS_SIGNED_RESOURCE,
        sig = url_encode(&signature),
    );
    Ok(sas)
}

/// Percent-encode a value for use in a SAS query string.
fn url_encode(s: &str) -> String {
    percent_encoding::utf8_percent_encode(s, percent_encoding::NON_ALPHANUMERIC).to_string()
}

/// SAS permission string for read-only access (read + list).
pub(crate) const SAS_READ: &str = "rl";

/// SAS permission string for read-write access (read, add, create, write, delete, list).
pub(crate) const SAS_READ_WRITE: &str = "racwdl";

/// Default SAS TTL in seconds (1 hour).
const DEFAULT_TTL_SECS: u64 = 3600;

/// Compute an expiry `DateTime<Utc>` that is `ttl_secs` from now.
pub(crate) fn sas_expiry(ttl_secs: u64) -> DateTime<Utc> {
    let now = SystemTime::now();
    let expiry = now + Duration::from_secs(ttl_secs);
    let secs = expiry
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    DateTime::from_timestamp(secs as i64, 0).unwrap_or_else(Utc::now)
}

// ── Public entry points used by credential_vending ──────────────────────────

/// Generate a User Delegation SAS token for an Azure Blob Storage container.
///
/// Uses the provided AAD bearer token to fetch a User Delegation Key, then
/// signs a container-scoped SAS with the given permissions.
pub async fn generate_user_delegation_sas(
    account: &str,
    container: &str,
    bearer_token: &str,
    read_only: bool,
    ttl_secs: u64,
) -> Result<String> {
    let expiry = sas_expiry(ttl_secs);
    let start = Utc::now();
    let key = fetch_user_delegation_key(account, bearer_token, start, expiry).await?;
    let permissions = if read_only { SAS_READ } else { SAS_READ_WRITE };
    build_user_delegation_sas(account, container, &key, expiry, permissions)
}

/// Generate a service SAS token for an Azure Blob Storage container using a storage account key.
///
/// This does not require an AAD token — the account key is used directly to sign the SAS.
/// Useful for local Azurite testing where managed identity or service principals are unavailable.
pub fn generate_storage_key_sas(
    account: &str,
    container: &str,
    account_key_b64: &str,
    read_only: bool,
    ttl_secs: u64,
) -> Result<String> {
    let expiry = sas_expiry(ttl_secs);
    let permissions = if read_only { SAS_READ } else { SAS_READ_WRITE };
    build_storage_key_sas(account, container, account_key_b64, expiry, permissions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sas_permissions_read() {
        assert_eq!(SAS_READ, "rl");
    }

    #[test]
    fn test_sas_permissions_read_write() {
        assert_eq!(SAS_READ_WRITE, "racwdl");
    }

    #[test]
    fn test_storage_key_sas_contains_required_fields() {
        // Use the well-known Azurite storage account key (public test credential).
        let azurite_key = "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";
        let sas = generate_storage_key_sas("devstoreaccount1", "test", azurite_key, true, 3600)
            .expect("SAS generation failed");
        assert!(sas.contains("sv="), "missing sv");
        assert!(sas.contains("se="), "missing se");
        assert!(sas.contains("sp=rl"), "expected read permissions");
        assert!(sas.contains("sig="), "missing sig");
    }

    #[test]
    fn test_storage_key_sas_read_write_permissions() {
        let azurite_key = "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";
        let sas = generate_storage_key_sas("devstoreaccount1", "test", azurite_key, false, 3600)
            .expect("SAS generation failed");
        assert!(sas.contains("sp=racwdl"), "expected read-write permissions");
    }
}
