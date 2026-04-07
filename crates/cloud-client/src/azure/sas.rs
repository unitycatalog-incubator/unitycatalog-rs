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

/// SAS signed version — must be at least 2020-02-10 to support `sdd=` directory depth.
const SAS_VERSION: &str = "2020-12-06";

/// Signed resource type: "c" = container (prefix restriction via `sdd=` on ADLS Gen2).
const SAS_SIGNED_RESOURCE: &str = "c";

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

// ── Directory depth helper ────────────────────────────────────────────────────

/// Compute the signed directory depth for a blob prefix path.
///
/// Azure ADLS Gen2 SAS tokens scope access to a directory prefix via `sdd=N`
/// where N is the number of `/`-separated, non-empty path segments in the prefix.
///
/// Examples:
/// - `""` → `None` (container-level, no `sdd`)
/// - `"data"` → `Some(1)`
/// - `"data/2024"` → `Some(2)`
/// - `"data/2024/"` → `Some(2)` (trailing slash ignored)
fn signed_directory_depth(prefix: &str) -> Option<usize> {
    let depth = prefix.split('/').filter(|s| !s.is_empty()).count();
    if depth == 0 { None } else { Some(depth) }
}

// ── SAS construction ─────────────────────────────────────────────────────────

/// Build a SAS token query string signed with a User Delegation Key.
///
/// When `prefix` is non-empty the SAS is scoped to that ADLS Gen2 directory
/// path via `sdd=` (requires a Hierarchical Namespace / ADLS Gen2 account).
/// On flat-namespace Blob Storage the `sdd=` parameter is ignored by the
/// service and the token remains container-wide.
///
/// Returns the SAS parameters as a query string (without leading `?`).
pub(crate) fn build_user_delegation_sas(
    account: &str,
    container: &str,
    prefix: &str,
    key: &UserDelegationKey,
    expiry: DateTime<Utc>,
    permissions: &str,
) -> Result<String> {
    let start = Utc::now();
    let start_str = start.to_rfc3339_opts(SecondsFormat::Secs, true);
    let expiry_str = expiry.to_rfc3339_opts(SecondsFormat::Secs, true);
    let depth = signed_directory_depth(prefix);

    // Canonicalized resource includes the prefix path for ADLS Gen2 directory SAS.
    let canonicalized_resource = if depth.is_some() && !prefix.is_empty() {
        let clean = prefix.trim_matches('/');
        format!("/blob/{account}/{container}/{clean}")
    } else {
        format!("/blob/{account}/{container}")
    };

    // signedDirectoryDepth field — empty string when not scoping to a directory.
    let sdd_field = depth.map(|d| d.to_string()).unwrap_or_default();

    // String-to-sign for User Delegation SAS (API version 2020-12-06).
    // https://learn.microsoft.com/en-us/rest/api/storageservices/create-user-delegation-sas
    // Field order (each separated by \n):
    //   signedPermissions, signedStart, signedExpiry, canonicalizedResource,
    //   signedKeyObjectId, signedKeyTenantId, signedKeyStart, signedKeyExpiry,
    //   signedKeyService, signedKeyVersion,
    //   signedAuthorizedUserObjectId (empty), signedUnauthorizedUserObjectId (empty),
    //   signedCorrelationId (empty), signedIP (empty), signedProtocol,
    //   signedVersion, signedResource, signedSnapshotTime (empty),
    //   signedEncryptionScope (empty), rscc (empty), rscd (empty), rsce (empty),
    //   rscl (empty), rsct (empty), signedDirectoryDepth
    let string_to_sign = format!(
        "{permissions}\n{start}\n{expiry}\n{resource}\n{oid}\n{tid}\n{skt}\n{ske}\n{sks}\n{skv}\n\n\n\nhttps\n{version}\n{sr}\n\n\n\n\n\n\n\n{sdd}",
        permissions = permissions,
        start = start_str,
        expiry = expiry_str,
        resource = canonicalized_resource,
        oid = key.signed_oid,
        tid = key.signed_tid,
        skt = key.signed_start,
        ske = key.signed_expiry,
        sks = key.signed_service,
        skv = key.signed_version,
        version = SAS_VERSION,
        sr = SAS_SIGNED_RESOURCE,
        sdd = sdd_field,
    );

    let key_bytes = BASE64
        .decode(&key.value)
        .map_err(|e| crate::Error::Generic { source: e.into() })?;
    let signature = BASE64.encode(hmac_sha256(&key_bytes, string_to_sign.as_bytes()).as_ref());

    let mut sas = format!(
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

    if let Some(d) = depth {
        sas.push_str(&format!("&sdd={d}"));
    }

    Ok(sas)
}

/// Build a SAS token signed with a storage account key.
///
/// When `prefix` is non-empty the SAS includes `sdd=` for ADLS Gen2
/// directory scoping. On flat-namespace accounts the parameter is ignored.
///
/// Uses the Service SAS signing algorithm (API version 2020-12-06).
/// <https://learn.microsoft.com/en-us/rest/api/storageservices/create-service-sas>
pub(crate) fn build_storage_key_sas(
    account: &str,
    container: &str,
    prefix: &str,
    account_key_b64: &str,
    expiry: DateTime<Utc>,
    permissions: &str,
) -> Result<String> {
    let start = Utc::now();
    let start_str = start.to_rfc3339_opts(SecondsFormat::Secs, true);
    let expiry_str = expiry.to_rfc3339_opts(SecondsFormat::Secs, true);
    let depth = signed_directory_depth(prefix);

    let canonicalized_resource = if depth.is_some() && !prefix.is_empty() {
        let clean = prefix.trim_matches('/');
        format!("/blob/{account}/{container}/{clean}")
    } else {
        format!("/blob/{account}/{container}")
    };

    let sdd_field = depth.map(|d| d.to_string()).unwrap_or_default();

    // String-to-sign for Service SAS (container resource, API version 2020-12-06).
    // https://learn.microsoft.com/en-us/rest/api/storageservices/create-service-sas
    // Field order (each separated by \n):
    //   signedPermissions, signedStart, signedExpiry, canonicalizedResource,
    //   signedIdentifier (empty), signedIP (empty), signedProtocol,
    //   signedVersion, signedResource, signedSnapshotTime (empty),
    //   signedEncryptionScope (empty), rscc (empty), rscd (empty), rsce (empty),
    //   rscl (empty), rsct (empty), signedDirectoryDepth
    let string_to_sign = format!(
        "{permissions}\n{start}\n{expiry}\n{resource}\n\nhttps\n{version}\n{sr}\n\n\n\n\n\n\n\n{sdd}",
        permissions = permissions,
        start = start_str,
        expiry = expiry_str,
        resource = canonicalized_resource,
        version = SAS_VERSION,
        sr = SAS_SIGNED_RESOURCE,
        sdd = sdd_field,
    );

    let key_bytes = BASE64
        .decode(account_key_b64)
        .map_err(|e| crate::Error::Generic { source: e.into() })?;
    let signature = BASE64.encode(hmac_sha256(&key_bytes, string_to_sign.as_bytes()).as_ref());

    let mut sas = format!(
        "sv={version}&se={expiry}&sp={permissions}&spr=https&sr={resource}&sig={sig}",
        version = SAS_VERSION,
        expiry = url_encode(&expiry_str),
        permissions = permissions,
        resource = SAS_SIGNED_RESOURCE,
        sig = url_encode(&signature),
    );

    if let Some(d) = depth {
        sas.push_str(&format!("&sdd={d}"));
    }

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

/// Generate a User Delegation SAS token scoped to an Azure Blob Storage container
/// and optionally a directory prefix within it (ADLS Gen2 `sdd=` scoping).
///
/// Uses the provided AAD bearer token to fetch a User Delegation Key, then signs
/// a SAS. When `prefix` is non-empty the `sdd=` parameter restricts access to
/// that directory path on ADLS Gen2 (Hierarchical Namespace) accounts; on
/// flat-namespace Blob Storage the parameter is silently ignored by the service.
pub async fn generate_user_delegation_sas(
    account: &str,
    container: &str,
    prefix: &str,
    bearer_token: &str,
    read_only: bool,
    ttl_secs: u64,
) -> Result<String> {
    let expiry = sas_expiry(ttl_secs);
    let start = Utc::now();
    let key = fetch_user_delegation_key(account, bearer_token, start, expiry).await?;
    let permissions = if read_only { SAS_READ } else { SAS_READ_WRITE };
    build_user_delegation_sas(account, container, prefix, &key, expiry, permissions)
}

/// Generate a service SAS token scoped to an Azure Blob Storage container and
/// optionally a directory prefix (ADLS Gen2 `sdd=` scoping) using a storage account key.
///
/// This does not require an AAD token — useful for local Azurite testing.
pub fn generate_storage_key_sas(
    account: &str,
    container: &str,
    prefix: &str,
    account_key_b64: &str,
    read_only: bool,
    ttl_secs: u64,
) -> Result<String> {
    let expiry = sas_expiry(ttl_secs);
    let permissions = if read_only { SAS_READ } else { SAS_READ_WRITE };
    build_storage_key_sas(
        account,
        container,
        prefix,
        account_key_b64,
        expiry,
        permissions,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed_directory_depth_empty() {
        assert_eq!(signed_directory_depth(""), None);
        assert_eq!(signed_directory_depth("/"), None);
    }

    #[test]
    fn test_signed_directory_depth_segments() {
        assert_eq!(signed_directory_depth("data"), Some(1));
        assert_eq!(signed_directory_depth("data/2024"), Some(2));
        assert_eq!(signed_directory_depth("data/2024/"), Some(2));
        assert_eq!(signed_directory_depth("/data/2024/events"), Some(3));
    }

    #[test]
    fn test_sas_permissions_read() {
        assert_eq!(SAS_READ, "rl");
    }

    #[test]
    fn test_sas_permissions_read_write() {
        assert_eq!(SAS_READ_WRITE, "racwdl");
    }

    #[test]
    fn test_storage_key_sas_no_prefix_no_sdd() {
        let azurite_key = "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";
        let sas = generate_storage_key_sas("devstoreaccount1", "test", "", azurite_key, true, 3600)
            .expect("SAS generation failed");
        assert!(sas.contains("sv="), "missing sv");
        assert!(sas.contains("se="), "missing se");
        assert!(sas.contains("sp=rl"), "expected read permissions");
        assert!(sas.contains("sig="), "missing sig");
        assert!(
            !sas.contains("sdd="),
            "should not have sdd for empty prefix"
        );
    }

    #[test]
    fn test_storage_key_sas_with_prefix_has_sdd() {
        let azurite_key = "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";
        let sas = generate_storage_key_sas(
            "devstoreaccount1",
            "test",
            "data/events",
            azurite_key,
            true,
            3600,
        )
        .expect("SAS generation failed");
        assert!(sas.contains("sdd=2"), "expected sdd=2 for data/events");
    }

    #[test]
    fn test_storage_key_sas_read_write_permissions() {
        let azurite_key = "Eby8vdM02xNOcqFlqUwJPLlmEtlCDXJ1OUzFT50uSRZ6IFsuFq2UVErCz4I6tq/K1SZFPTOtr/KBHBeksoGMGw==";
        let sas =
            generate_storage_key_sas("devstoreaccount1", "test", "", azurite_key, false, 3600)
                .expect("SAS generation failed");
        assert!(sas.contains("sp=racwdl"), "expected read-write permissions");
    }
}
