use std::sync::Arc;
use std::time::Instant;

use chrono::{DateTime, Utc};
use cloud_client::{TemporaryToken, TokenCache};
use object_store::aws::AwsCredential;
use object_store::azure::AzureCredential;
use object_store::gcp::GcpCredential;
use object_store::{CredentialProvider, Result};
use unitycatalog_client::{PathOperation, TableOperation, TemporaryCredentialClient};
use unitycatalog_common::models::temporary_credentials::v1::{
    TemporaryCredential, temporary_credential::Credentials,
};

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SecurableRef {
    Table(uuid::Uuid, TableOperation),
    Path(url::Url, PathOperation),
}

pub struct UCCredentialProvider<T> {
    client: TemporaryCredentialClient,
    cache: TokenCache<Arc<T>>,
    securable: SecurableRef,
}

pub async fn new_azure(
    client: TemporaryCredentialClient,
    cred: &TemporaryCredential,
    securable: SecurableRef,
) -> Result<UCCredentialProvider<AzureCredential>> {
    let cache = TokenCache::<Arc<AzureCredential>>::default();
    // NB: we just do this to initialize the cache with the credential we already have.
    let _ = cache.get_or_insert_with(|| async { as_azure(cred) });
    Ok(UCCredentialProvider::<AzureCredential> {
        client,
        cache,
        securable,
    })
}

pub async fn new_aws(
    client: TemporaryCredentialClient,
    cred: &TemporaryCredential,
    securable: SecurableRef,
) -> Result<UCCredentialProvider<AwsCredential>> {
    let cache = TokenCache::<Arc<AwsCredential>>::default();
    // NB: we just do this to initialize the cache with the credential we already have.
    let _ = cache.get_or_insert_with(|| async { as_aws(cred) });
    Ok(UCCredentialProvider::<AwsCredential> {
        client,
        cache,
        securable,
    })
}

pub async fn new_gcp(
    client: TemporaryCredentialClient,
    cred: &TemporaryCredential,
    securable: SecurableRef,
) -> Result<UCCredentialProvider<GcpCredential>> {
    let cache = TokenCache::<Arc<GcpCredential>>::default();
    // NB: we just do this to initialize the cache with the credential we already have.
    let _ = cache.get_or_insert_with(|| async { as_gcp(cred) });
    Ok(UCCredentialProvider::<GcpCredential> {
        client,
        cache,
        securable,
    })
}

impl<T> UCCredentialProvider<T> {
    async fn get_credential_inner(&self) -> Result<TemporaryCredential> {
        match &self.securable {
            SecurableRef::Table(table, op) => Ok(self
                .client
                .temporary_table_credential(*table, *op)
                .await
                .map_err(Error::from)?
                .0),
            SecurableRef::Path(path, op) => Ok(self
                .client
                .temporary_path_credential(path.clone(), *op, false)
                .await
                .map_err(Error::from)?
                .0),
        }
    }
}

impl<T> std::fmt::Debug for UCCredentialProvider<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UCCredentialProvider")
            .field("securable", &self.securable)
            .finish()
    }
}

#[async_trait::async_trait]
impl CredentialProvider for UCCredentialProvider<AzureCredential> {
    type Credential = AzureCredential;

    async fn get_credential(&self) -> Result<Arc<AzureCredential>> {
        self.cache
            .get_or_insert_with(|| async { as_azure(&self.get_credential_inner().await?) })
            .await
    }
}

#[async_trait::async_trait]
impl CredentialProvider for UCCredentialProvider<AwsCredential> {
    type Credential = AwsCredential;

    async fn get_credential(&self) -> Result<Arc<AwsCredential>> {
        self.cache
            .get_or_insert_with(|| async { as_aws(&self.get_credential_inner().await?) })
            .await
    }
}

#[async_trait::async_trait]
impl CredentialProvider for UCCredentialProvider<GcpCredential> {
    type Credential = GcpCredential;

    async fn get_credential(&self) -> Result<Arc<GcpCredential>> {
        self.cache
            .get_or_insert_with(|| async { as_gcp(&self.get_credential_inner().await?) })
            .await
    }
}

pub(super) fn as_azure(cred: &TemporaryCredential) -> Result<TemporaryToken<Arc<AzureCredential>>> {
    use Credentials::*;

    let az_cred = match cred
        .credentials
        .as_ref()
        .ok_or(crate::Error::NoCredential)?
    {
        AzureAad(token) => AzureCredential::BearerToken(token.aad_token.clone()),
        AzureUserDelegationSas(sas) => {
            // split sas query string into pairs
            let pairs = sas.sas_token.split('&');
            let mut map = Vec::new();
            for pair in pairs {
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    map.push((key.to_string(), value.to_string()));
                }
            }
            AzureCredential::SASToken(map)
        }
        _ => return Err(crate::Error::credential_mismatch("Expected Azure credential.").into()),
    };

    Ok(TemporaryToken {
        token: Arc::new(az_cred),
        expiry: get_expiry(&cred)?,
    })
}

pub(super) fn as_aws(cred: &TemporaryCredential) -> Result<TemporaryToken<Arc<AwsCredential>>> {
    use Credentials::*;

    let aws_cred = match cred
        .credentials
        .as_ref()
        .ok_or(crate::Error::NoCredential)?
    {
        // TODO: the return type also contains `access_point` field.
        // but we currently cannot use it in the current apis.
        AwsTempCredentials(aws) => AwsCredential {
            key_id: aws.access_key_id.clone(),
            secret_key: aws.secret_access_key.clone(),
            token: Some(aws.session_token.clone()),
        },
        R2TempCredentials(r2) => AwsCredential {
            key_id: r2.access_key_id.clone(),
            secret_key: r2.secret_access_key.clone(),
            token: Some(r2.session_token.clone()),
        },
        _ => return Err(crate::Error::credential_mismatch("Expected AWS credential.").into()),
    };

    Ok(TemporaryToken {
        token: Arc::new(aws_cred),
        expiry: get_expiry(&cred)?,
    })
}

pub(super) fn as_gcp(cred: &TemporaryCredential) -> Result<TemporaryToken<Arc<GcpCredential>>> {
    use Credentials::*;

    let gcp_cred = match cred
        .credentials
        .as_ref()
        .ok_or(crate::Error::NoCredential)?
    {
        GcpOauthToken(gcp) => GcpCredential {
            bearer: gcp.oauth_token.clone(),
        },
        _ => return Err(crate::Error::credential_mismatch("Expected GCS credential.").into()),
    };

    Ok(TemporaryToken {
        token: Arc::new(gcp_cred),
        expiry: get_expiry(&cred)?,
    })
}

fn get_expiry(cred: &TemporaryCredential) -> Result<Option<Instant>> {
    let expiry = DateTime::from_timestamp_millis(cred.expiration_time)
        .ok_or(Error::credential_mismatch("Invalid expiration time"))?;
    let ttl = (expiry - Utc::now()).to_std().unwrap_or_default();

    Ok(Some(Instant::now() + ttl))
}
