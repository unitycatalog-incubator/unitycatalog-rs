use bytes::Bytes;
use unitycatalog_common::Result;
use unitycatalog_common::services::SecretManager;
use uuid::Uuid;

use crate::GraphStore;

const DUMMY: &str = "dummy";

#[async_trait::async_trait]
impl SecretManager for GraphStore {
    async fn get_secret(&self, secret_name: &str) -> Result<(Uuid, Bytes)> {
        let mut conn = self.pool.acquire().await.map_err(crate::Error::from)?;
        #[derive(sqlx::FromRow)]
        struct Secret {
            id: Uuid,
            value: Option<String>,
        }
        let res = sqlx::query_as!(
            Secret,
            r#"
            SELECT id, pgp_sym_decrypt(value, $1) as value FROM secrets
            WHERE name = $2
            ORDER BY id DESC
            LIMIT 1
            "#,
            DUMMY,
            secret_name,
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(crate::Error::from)?;
        if res.value.is_none() {
            return Err(crate::Error::entity_not_found(secret_name).into());
        }
        Ok((res.id, bytes::Bytes::from(res.value.unwrap())))
    }

    async fn get_secret_version(&self, secret_name: &str, version: Uuid) -> Result<Bytes> {
        let mut conn = self.pool.acquire().await.map_err(crate::Error::from)?;
        let value: Option<String> = sqlx::query_scalar!(
            r#"
            SELECT pgp_sym_decrypt(value, $2) FROM secrets
            WHERE name = $1 AND id = $3
            "#,
            secret_name,
            DUMMY,
            version,
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(crate::Error::from)?;
        if let Some(value) = value {
            return Ok(bytes::Bytes::from(value));
        }
        Err(crate::Error::entity_not_found(secret_name).into())
    }

    async fn create_secret(&self, secret_name: &str, secret_value: Bytes) -> Result<Uuid> {
        let mut txn = self.pool.begin().await.map_err(crate::Error::from)?;
        let value = std::str::from_utf8(&secret_value).unwrap();
        let query_result = sqlx::query_scalar!(
            r#"
            INSERT INTO secrets(name, value)
            VALUES ($1, pgp_sym_encrypt($2, $3, 'cipher-algo=aes256'))
            RETURNING id
            "#,
            secret_name,
            value,
            DUMMY,
        )
        .fetch_one(&mut *txn)
        .await;
        match query_result {
            Ok(id) => {
                txn.commit().await.map_err(crate::Error::from)?;
                Ok(id)
            }
            Err(e) => {
                txn.rollback().await.map_err(crate::Error::from)?;
                tracing::error!("create_secret: {} -> {:?}", secret_name, e);
                Err(crate::Error::from(e).into())
            }
        }
    }

    async fn update_secret(&self, secret_name: &str, secret_value: Bytes) -> Result<Uuid> {
        // NB: we create a new version of the secret on each create operation.
        // get request should be used to get the latest version.
        self.create_secret(secret_name, secret_value).await
    }

    async fn delete_secret(&self, secret_name: &str) -> Result<()> {
        let mut txn = self.pool.begin().await.map_err(crate::Error::from)?;
        let _ = sqlx::query!(
            r#"
            DELETE FROM secrets
            WHERE name = $1
            "#,
            secret_name,
        )
        .execute(&mut *txn)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }
}
