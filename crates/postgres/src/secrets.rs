use bytes::Bytes;
use unitycatalog_common::Result;
use unitycatalog_common::services::secrets::SecretManager;

use crate::GraphStore;

#[async_trait::async_trait]
impl SecretManager for GraphStore {
    async fn get_secret(&self, secret_name: &str) -> Result<Bytes> {
        let mut conn = self.pool.acquire().await.map_err(crate::Error::from)?;
        let value: Option<Vec<u8>> = sqlx::query_scalar!(
            r#"
            SELECT value FROM secrets
            WHERE name = $1
            "#,
            secret_name,
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(crate::Error::from)?;
        let blob = value.ok_or_else(|| crate::Error::entity_not_found(secret_name))?;
        let plaintext = self.encryptor.open(secret_name, &blob).await?;
        Ok(Bytes::from(plaintext))
    }

    async fn put_secret(&self, secret_name: &str, secret_value: Bytes) -> Result<()> {
        let blob = self.encryptor.seal(secret_name, &secret_value).await?;
        let mut txn = self.pool.begin().await.map_err(crate::Error::from)?;
        sqlx::query!(
            r#"
            INSERT INTO secrets(name, value)
            VALUES ($1, $2)
            ON CONFLICT (name) DO UPDATE SET value = EXCLUDED.value
            "#,
            secret_name,
            &blob,
        )
        .execute(&mut *txn)
        .await
        .map_err(crate::Error::from)?;
        txn.commit().await.map_err(crate::Error::from)?;
        Ok(())
    }

    async fn delete_secret(&self, secret_name: &str) -> Result<()> {
        let mut txn = self.pool.begin().await.map_err(crate::Error::from)?;
        let result = sqlx::query!(
            r#"
            DELETE FROM secrets
            WHERE name = $1
            "#,
            secret_name,
        )
        .execute(&mut *txn)
        .await
        .map_err(crate::Error::from)?;
        if result.rows_affected() == 0 {
            txn.rollback().await.map_err(crate::Error::from)?;
            return Err(crate::Error::entity_not_found(secret_name).into());
        }
        txn.commit().await.map_err(crate::Error::from)?;
        Ok(())
    }
}
