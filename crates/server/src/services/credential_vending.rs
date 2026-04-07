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
    AwsTemporaryCredentials, AzureAad, TemporaryCredential, temporary_credential::Credentials,
};

use crate::{Error, Result};

/// Default credential TTL when the cloud provider does not supply an expiry.
const DEFAULT_TTL_SECS: u64 = 3600;

/// Convert an optional `Instant` expiry to epoch milliseconds.
///
/// Uses `SystemTime::now() + (expiry - Instant::now())` so the wall-clock epoch
/// is computed from the relative expiry duration, which is what the cloud SDKs
/// return (duration-since-now, not absolute time).
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

fn azure_token_to_temporary_credential(
    url: &str,
    token: TemporaryToken<Arc<AzureCredential>>,
) -> TemporaryCredential {
    let expiration_time = expiry_to_epoch_millis(token.expiry);
    let aad_token = match token.token.as_ref() {
        AzureCredential::BearerToken(t) => t.clone(),
    };
    TemporaryCredential {
        expiration_time,
        url: url.to_owned(),
        credentials: Some(Credentials::AzureAad(AzureAad { aad_token })),
    }
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

/// Generate a temporary credential for the given `Credential` and storage `url`.
///
/// Dispatches to the appropriate cloud provider based on which credential field
/// is populated:
/// - `AzureServicePrincipal` → Azure AD bearer token via OAuth2 client-secret or workload identity
/// - `AzureManagedIdentity`  → Azure AD bearer token via IMDS
/// - `AzureStorageKey`       → error (storage keys cannot be exchanged for short-lived credentials)
/// - `AwsIamRoleConfig`      → AWS STS `AssumeRole` temporary credentials
/// - GCP                     → not yet implemented
pub(crate) async fn vend_credential(
    credential: &Credential,
    url: &str,
) -> Result<TemporaryCredential> {
    if let Some(sp) = &credential.azure_service_principal {
        return vend_azure_service_principal(sp, url).await;
    }
    if let Some(msi) = &credential.azure_managed_identity {
        return vend_azure_managed_identity(msi, url).await;
    }
    if credential.azure_storage_key.is_some() {
        return Err(Error::invalid_argument(
            "Azure storage key credentials cannot vend temporary tokens; \
             use a service principal or managed identity instead.",
        ));
    }
    if let Some(role) = &credential.aws_iam_role_config {
        return vend_aws_iam_role(role, url).await;
    }
    Err(Error::invalid_argument(
        "No supported credential type found on this credential object.",
    ))
}

async fn vend_azure_service_principal(
    sp: &unitycatalog_common::models::credentials::v1::AzureServicePrincipal,
    url: &str,
) -> Result<TemporaryCredential> {
    let token = match &sp.credential {
        Some(AzureSpCredential::ClientSecret(secret)) => {
            cloud_client::azure::fetch_client_secret_token(
                &sp.directory_id,
                sp.application_id.clone(),
                secret.clone(),
                None,
            )
            .await?
        }
        Some(AzureSpCredential::FederatedTokenFile(token_file)) => {
            cloud_client::azure::fetch_workload_identity_token(
                &sp.directory_id,
                sp.application_id.clone(),
                token_file.clone(),
                None,
            )
            .await?
        }
        None => {
            return Err(Error::invalid_argument(
                "Azure service principal credential is missing client_secret or federated_token_file.",
            ));
        }
    };
    Ok(azure_token_to_temporary_credential(url, token))
}

async fn vend_azure_managed_identity(
    msi: &unitycatalog_common::models::credentials::v1::AzureManagedIdentity,
    url: &str,
) -> Result<TemporaryCredential> {
    let (client_id, object_id, msi_res_id) = match &msi.identifier {
        Some(AzureMiIdentifier::ApplicationId(id)) => (Some(id.clone()), None, None),
        Some(AzureMiIdentifier::ObjectId(id)) => (None, Some(id.clone()), None),
        Some(AzureMiIdentifier::MsiResourceId(id)) => (None, None, Some(id.clone())),
        None => (None, None, None),
    };
    let token =
        cloud_client::azure::fetch_managed_identity_token(client_id, object_id, msi_res_id).await?;
    Ok(azure_token_to_temporary_credential(url, token))
}

async fn vend_aws_iam_role(
    role: &unitycatalog_common::models::credentials::v1::AwsIamRoleConfig,
    url: &str,
) -> Result<TemporaryCredential> {
    let region = role.region.as_deref().unwrap_or("us-east-1");
    let token = cloud_client::aws::assume_role(&role.role_arn, region, None).await?;
    Ok(aws_token_to_temporary_credential(url, token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_expiry_to_epoch_millis_with_expiry() {
        let future_expiry = Instant::now() + Duration::from_secs(3600);
        let millis = expiry_to_epoch_millis(Some(future_expiry));
        let now_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        // Should be roughly now + 3600s, within a 5s margin
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
    fn test_azure_token_to_temporary_credential() {
        use std::sync::Arc;
        let token = TemporaryToken {
            token: Arc::new(AzureCredential::BearerToken("test-token".to_string())),
            expiry: Some(Instant::now() + Duration::from_secs(3600)),
        };
        let cred =
            azure_token_to_temporary_credential("https://storage.example.com/container", token);
        assert_eq!(cred.url, "https://storage.example.com/container");
        assert!(cred.expiration_time > 0);
        match cred.credentials {
            Some(Credentials::AzureAad(aad)) => assert_eq!(aad.aad_token, "test-token"),
            _ => panic!("expected AzureAad credential"),
        }
    }

    #[test]
    fn test_aws_token_to_temporary_credential() {
        use std::sync::Arc;
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
}
