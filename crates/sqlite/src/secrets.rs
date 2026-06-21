use bytes::Bytes;
use chrono::Utc;
use unitycatalog_common::Result;
use unitycatalog_common::services::secrets::SecretManager;
use uuid::Uuid;

use crate::SqliteStore;

#[async_trait::async_trait]
impl SecretManager for SqliteStore {
    async fn get_secret(&self, secret_name: &str) -> Result<Bytes> {
        let value: Option<Vec<u8>> =
            sqlx::query_scalar!("SELECT value FROM secrets WHERE name = ?", secret_name,)
                .fetch_optional(&self.pool)
                .await
                .map_err(crate::Error::from)?;
        let blob = value.ok_or_else(|| crate::Error::entity_not_found(secret_name))?;
        let plaintext = self.encryptor.open(secret_name, &blob).await?;

        // Lazy KEK rotation: if this secret was sealed under a retired KEK, re-wrap its data key
        // under the active KEK and write it back. Best-effort — a write failure must not fail the
        // read, and the value ciphertext is untouched so the result is identical either way.
        if let Ok(Some(rewrapped)) = self.encryptor.rewrap(&blob).await {
            let _ = sqlx::query!(
                "UPDATE secrets SET value = ? WHERE name = ?",
                rewrapped,
                secret_name,
            )
            .execute(&self.pool)
            .await;
        }

        Ok(Bytes::from(plaintext))
    }

    async fn put_secret(&self, secret_name: &str, secret_value: Bytes) -> Result<()> {
        let blob = self.encryptor.seal(secret_name, &secret_value).await?;
        let id = Uuid::now_v7().as_bytes().to_vec();
        let created_at = Utc::now().timestamp_micros();
        sqlx::query!(
            r#"
            INSERT INTO secrets ( id, name, value, created_at )
            VALUES ( ?, ?, ?, ? )
            ON CONFLICT (name) DO UPDATE SET value = excluded.value, updated_at = ?
            "#,
            id,
            secret_name,
            blob,
            created_at,
            created_at,
        )
        .execute(&self.pool)
        .await
        .map_err(crate::Error::from)?;
        Ok(())
    }

    async fn delete_secret(&self, secret_name: &str) -> Result<()> {
        let result = sqlx::query!("DELETE FROM secrets WHERE name = ?", secret_name)
            .execute(&self.pool)
            .await
            .map_err(crate::Error::from)?;
        if result.rows_affected() == 0 {
            return Err(crate::Error::entity_not_found(secret_name).into());
        }
        Ok(())
    }
}
