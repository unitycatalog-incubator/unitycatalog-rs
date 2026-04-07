use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cloud_client::TemporaryToken;
use cloud_client::aws::AwsCredential;
use cloud_client::azure::AzureCredential;
use unitycatalog_common::models::credentials::v1::{
    Credential, azure_managed_identity::Identifier as AzureMiIdentifier,
    azure_service_principal::Credential as AzureSpCredential,
};
use unitycatalog_common::models::temporary_credentials::v1::{
    AwsTemporaryCredentials, AzureUserDelegationSas, TemporaryCredential,
    temporary_credential::Credentials,
};

use crate::services::location::StorageLocationUrl;
use crate::{Error, Result};

/// Default credential TTL when the cloud provider does not supply an expiry.
const DEFAULT_TTL_SECS: u64 = 3600;

/// The operation requested by the caller — used to determine the minimum permissions
/// that should be encoded in the vended credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VendOperation {
    /// Read-only access (s3:GetObject, SAS sp=rl, …).
    Read,
    /// Read and write access (s3:PutObject + s3:DeleteObject, SAS sp=racwdl, …).
    ReadWrite,
}

/// Convert an optional `Instant` expiry to epoch milliseconds.
fn expiry_to_epoch_millis(expiry: Option<Instant>) -> i64 {
    let ttl = match expiry {
        Some(exp) => exp
            .checked_duration_since(Instant::now())
            .unwrap_or_default(),
        None => Duration::from_secs(DEFAULT_TTL_SECS),
    };
    let wall = SystemTime::now() + ttl;
    wall.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn aws_token_to_temporary_credential(
    url: &str,
    token: TemporaryToken<Arc<AwsCredential>>,
) -> TemporaryCredential {
    let expiration_time = expiry_to_epoch_millis(token.expiry);
    let cred = token.token.as_ref();
    TemporaryCredential {
        expiration_time,
        url: url.to_owned(),
        credentials: Some(Credentials::AwsTempCredentials(AwsTemporaryCredentials {
            access_key_id: cred.key_id.clone(),
            secret_access_key: cred.secret_key.clone(),
            session_token: cred.token.clone().unwrap_or_default(),
            access_point: String::new(),
        })),
    }
}

fn azure_sas_to_temporary_credential(url: &str, sas_token: String) -> TemporaryCredential {
    // Compute expiry from the `se=` parameter if present; fall back to 1 h.
    let expiration_time =
        parse_sas_expiry(&sas_token).unwrap_or_else(|| expiry_to_epoch_millis(None));
    TemporaryCredential {
        expiration_time,
        url: url.to_owned(),
        credentials: Some(Credentials::AzureUserDelegationSas(
            AzureUserDelegationSas { sas_token },
        )),
    }
}

/// Parse the `se=` (signed-expiry) field from a SAS query string and return epoch millis.
fn parse_sas_expiry(sas: &str) -> Option<i64> {
    for part in sas.split('&') {
        if let Some(encoded) = part.strip_prefix("se=") {
            let decoded = percent_encoding::percent_decode_str(encoded)
                .decode_utf8()
                .ok()?;
            let dt = chrono::DateTime::parse_from_rfc3339(&decoded).ok()?;
            return Some(dt.timestamp_millis());
        }
    }
    None
}

/// Build an AWS inline session policy scoped to `bucket` / `prefix` for the given operation.
///
/// The returned JSON string can be passed as the `Policy` parameter to `STS:AssumeRole`.
/// It is intersected with the role's own policy, so it can only restrict, never expand.
fn build_s3_session_policy(bucket: &str, prefix: &str, operation: VendOperation) -> String {
    let object_arn = if prefix.is_empty() {
        format!("arn:aws:s3:::{bucket}/*")
    } else {
        format!("arn:aws:s3:::{bucket}/{prefix}/*")
    };
    let bucket_arn = format!("arn:aws:s3:::{bucket}");

    let actions = match operation {
        VendOperation::Read => {
            r#"["s3:GetObject","s3:GetObjectVersion","s3:ListBucket","s3:GetBucketLocation"]"#
        }
        VendOperation::ReadWrite => {
            r#"["s3:GetObject","s3:GetObjectVersion","s3:PutObject","s3:DeleteObject","s3:ListBucket","s3:GetBucketLocation"]"#
        }
    };

    format!(
        r#"{{"Version":"2012-10-17","Statement":[{{"Effect":"Allow","Action":{actions},"Resource":["{object_arn}","{bucket_arn}"]}}]}}"#
    )
}

/// Generate a temporary credential for the given `Credential` and storage `url`,
/// downscoped to the requested `operation`.
///
/// Dispatches to the appropriate cloud provider based on which credential field
/// is populated:
/// - `AzureServicePrincipal` → fetch AAD bearer token, then exchange for a User Delegation SAS
/// - `AzureManagedIdentity`  → fetch AAD bearer token via IMDS, then User Delegation SAS
/// - `AzureStorageKey`       → service SAS signed with the account key
/// - `AwsIamRoleConfig`      → AWS STS `AssumeRole` with inline session policy
/// - GCP                     → not yet implemented
pub(crate) async fn vend_credential(
    credential: &Credential,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    if let Some(sp) = &credential.azure_service_principal {
        return vend_azure_service_principal(sp, url, operation).await;
    }
    if let Some(msi) = &credential.azure_managed_identity {
        return vend_azure_managed_identity(msi, url, operation).await;
    }
    if let Some(key) = &credential.azure_storage_key {
        return vend_azure_storage_key(key, url, operation).await;
    }
    if let Some(role) = &credential.aws_iam_role_config {
        return vend_aws_iam_role(role, url, operation).await;
    }
    Err(Error::invalid_argument(
        "No supported credential type found on this credential object.",
    ))
}

async fn vend_azure_service_principal(
    sp: &unitycatalog_common::models::credentials::v1::AzureServicePrincipal,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let bearer_token = match &sp.credential {
        Some(AzureSpCredential::ClientSecret(secret)) => {
            let token = cloud_client::azure::fetch_client_secret_token(
                &sp.directory_id,
                sp.application_id.clone(),
                secret.clone(),
                None,
            )
            .await?;
            extract_bearer_token(token.token.as_ref())?
        }
        Some(AzureSpCredential::FederatedTokenFile(token_file)) => {
            let token = cloud_client::azure::fetch_workload_identity_token(
                &sp.directory_id,
                sp.application_id.clone(),
                token_file.clone(),
                None,
            )
            .await?;
            extract_bearer_token(token.token.as_ref())?
        }
        None => {
            return Err(Error::invalid_argument(
                "Azure service principal credential is missing client_secret or federated_token_file.",
            ));
        }
    };
    vend_azure_sas_from_bearer(url, &bearer_token, operation).await
}

async fn vend_azure_managed_identity(
    msi: &unitycatalog_common::models::credentials::v1::AzureManagedIdentity,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let (client_id, object_id, msi_res_id) = match &msi.identifier {
        Some(AzureMiIdentifier::ApplicationId(id)) => (Some(id.clone()), None, None),
        Some(AzureMiIdentifier::ObjectId(id)) => (None, Some(id.clone()), None),
        Some(AzureMiIdentifier::MsiResourceId(id)) => (None, None, Some(id.clone())),
        None => (None, None, None),
    };
    let token =
        cloud_client::azure::fetch_managed_identity_token(client_id, object_id, msi_res_id).await?;
    let bearer_token = extract_bearer_token(token.token.as_ref())?;
    vend_azure_sas_from_bearer(url, &bearer_token, operation).await
}

async fn vend_azure_storage_key(
    key: &unitycatalog_common::models::credentials::v1::AzureStorageKey,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let storage_url = StorageLocationUrl::parse(url)?;
    let account = storage_url
        .azure_account()
        .or_else(|| {
            // For Azurite the account is encoded in the path; fall back to the key's account field.
            Some(key.account_name.clone())
        })
        .ok_or_else(|| {
            Error::invalid_argument("Cannot determine Azure storage account from URL")
        })?;
    let (container, prefix) = storage_url.bucket_and_prefix()?;
    let read_only = operation == VendOperation::Read;
    let sas_token = cloud_client::azure::generate_storage_key_sas(
        &account,
        &container,
        &prefix,
        &key.account_key,
        read_only,
        DEFAULT_TTL_SECS,
    )?;
    Ok(azure_sas_to_temporary_credential(url, sas_token))
}

/// Given an AAD bearer token, fetch a User Delegation Key and build a scoped SAS.
async fn vend_azure_sas_from_bearer(
    url: &str,
    bearer_token: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let storage_url = StorageLocationUrl::parse(url)?;
    let account = storage_url.azure_account().ok_or_else(|| {
        Error::invalid_argument("Cannot determine Azure storage account from URL")
    })?;
    let (container, prefix) = storage_url.bucket_and_prefix()?;
    let read_only = operation == VendOperation::Read;
    let sas_token = cloud_client::azure::generate_user_delegation_sas(
        &account,
        &container,
        &prefix,
        bearer_token,
        read_only,
        DEFAULT_TTL_SECS,
    )
    .await?;
    Ok(azure_sas_to_temporary_credential(url, sas_token))
}

fn extract_bearer_token(credential: &AzureCredential) -> Result<String> {
    match credential {
        AzureCredential::BearerToken(t) => Ok(t.clone()),
    }
}

async fn vend_aws_iam_role(
    role: &unitycatalog_common::models::credentials::v1::AwsIamRoleConfig,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let region = role.region.as_deref().unwrap_or("us-east-1");
    let storage_url = StorageLocationUrl::parse(url)?;
    let (bucket, prefix) = storage_url.bucket_and_prefix()?;
    let policy = build_s3_session_policy(&bucket, &prefix, operation);
    let token = cloud_client::aws::assume_role(&role.role_arn, region, None, Some(policy)).await?;
    Ok(aws_token_to_temporary_credential(url, token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_expiry_to_epoch_millis_with_expiry() {
        let future_expiry = Instant::now() + Duration::from_secs(3600);
        let millis = expiry_to_epoch_millis(Some(future_expiry));
        let now_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert!(millis > now_millis + 3590_000, "expiry too soon: {millis}");
        assert!(millis < now_millis + 3610_000, "expiry too far: {millis}");
    }

    #[test]
    fn test_expiry_to_epoch_millis_none_defaults_to_one_hour() {
        let millis = expiry_to_epoch_millis(None);
        let now_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert!(millis > now_millis + 3590_000, "expiry too soon: {millis}");
        assert!(millis < now_millis + 3610_000, "expiry too far: {millis}");
    }

    #[test]
    fn test_aws_token_to_temporary_credential() {
        let token = TemporaryToken {
            token: Arc::new(AwsCredential {
                key_id: "AKIAIOSFODNN7EXAMPLE".to_string(), // gitleaks:allow
                secret_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(), // gitleaks:allow
                token: Some("session-token".to_string()),
            }),
            expiry: Some(Instant::now() + Duration::from_secs(3600)),
        };
        let cred = aws_token_to_temporary_credential("s3://my-bucket/path", token);
        assert_eq!(cred.url, "s3://my-bucket/path");
        assert!(cred.expiration_time > 0);
        match cred.credentials {
            Some(Credentials::AwsTempCredentials(aws)) => {
                assert_eq!(aws.access_key_id, "AKIAIOSFODNN7EXAMPLE"); // gitleaks:allow
                assert_eq!(aws.session_token, "session-token");
            }
            _ => panic!("expected AwsTempCredentials credential"),
        }
    }

    #[test]
    fn test_s3_session_policy_read_only() {
        let policy = build_s3_session_policy("my-bucket", "some/prefix", VendOperation::Read);
        assert!(policy.contains("s3:GetObject"), "missing GetObject");
        assert!(policy.contains("s3:ListBucket"), "missing ListBucket");
        assert!(
            !policy.contains("s3:PutObject"),
            "should not allow PutObject for read"
        );
        assert!(
            !policy.contains("s3:DeleteObject"),
            "should not allow DeleteObject for read"
        );
        assert!(
            policy.contains("arn:aws:s3:::my-bucket/some/prefix/*"),
            "missing object ARN"
        );
        assert!(
            policy.contains("arn:aws:s3:::my-bucket\""),
            "missing bucket ARN"
        );
    }

    #[test]
    fn test_s3_session_policy_read_write() {
        let policy = build_s3_session_policy("my-bucket", "data/", VendOperation::ReadWrite);
        assert!(policy.contains("s3:PutObject"), "missing PutObject");
        assert!(policy.contains("s3:DeleteObject"), "missing DeleteObject");
        assert!(
            policy.contains("arn:aws:s3:::my-bucket/data//*"),
            "missing object ARN"
        );
    }

    #[test]
    fn test_s3_session_policy_empty_prefix() {
        let policy = build_s3_session_policy("my-bucket", "", VendOperation::Read);
        assert!(
            policy.contains("arn:aws:s3:::my-bucket/*"),
            "missing wildcard ARN"
        );
    }

    #[test]
    fn test_parse_sas_expiry() {
        let sas = "sv=2020-12-06&se=2024-01-01T00%3A00%3A00Z&sp=rl&sig=abc";
        let millis = parse_sas_expiry(sas);
        assert!(millis.is_some(), "expected to parse expiry");
        assert!(millis.unwrap() > 0);
    }
}
