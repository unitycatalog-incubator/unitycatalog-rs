//! Embedded SQLite storage layer for objects and associations.
//!
//! This is the SQLite counterpart to the Postgres `GraphStore`. The data model
//! is identical (a graph of `objects` and directed `associations` with
//! auto-maintained inverse edges), but the SQL is adapted to the SQLite dialect:
//! UUIDs are stored as BLOBs and generated Rust-side, labels and names are TEXT,
//! timestamps are INTEGER microseconds, and `updated_at` is set explicitly.
//!
//! Rows are read into the local primitive-typed [`ObjectRow`] / [`AssociationRow`]
//! structs and converted to the shared `olai_store` types in Rust. This avoids
//! depending on the Postgres-specific `sqlx::Type` derives on `ObjectLabel`,
//! `AssociationLabel`, and `ResourceName`.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use unitycatalog_common::services::encryption::EnvelopeEncryptor;
use unitycatalog_common::{AssociationLabel, Object, ObjectLabel};
use uuid::Uuid;

use olai_store::EMPTY_RESOURCE_NAME;
use olai_store::name::ResourceName;

use crate::constants::MAX_PAGE_SIZE;
use crate::error::{Error, Result};
use crate::pagination::{PaginateToken, V1PaginateToken};

static MIGRATOR: Migrator = sqlx::migrate!();

/// An embedded, file-based SQLite store for catalog metadata.
#[derive(Clone)]
pub struct SqliteStore {
    pub(crate) pool: SqlitePool,
    pub(crate) encryptor: EnvelopeEncryptor,
}

impl SqliteStore {
    pub fn new(pool: SqlitePool, encryptor: EnvelopeEncryptor) -> Self {
        Self { pool, encryptor }
    }

    /// Open (creating if necessary) a SQLite database at `path`.
    ///
    /// `path` is a filesystem path to the database file; the special value
    /// `:memory:` opens an ephemeral in-memory database (useful for tests).
    /// The database file and any missing schema are created on first use.
    pub async fn connect(path: impl AsRef<str>, encryptor: EnvelopeEncryptor) -> Result<Self> {
        let path = path.as_ref();
        let options = if path == ":memory:" {
            SqliteConnectOptions::from_str("sqlite::memory:").map_err(Error::from)?
        } else {
            SqliteConnectOptions::from_str(&format!("sqlite://{path}"))
                .map_err(Error::from)?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .busy_timeout(std::time::Duration::from_secs(5))
                .foreign_keys(true)
        };
        // A single connection in WAL mode is the simplest correct setup for an
        // embedded single-writer store; an in-memory database must use one
        // connection so every caller sees the same database.
        let pool = SqlitePoolOptions::new()
            .max_connections(if path == ":memory:" { 1 } else { 16 })
            .connect_with(options)
            .await
            .map_err(Error::from)?;
        Ok(Self::new(pool, encryptor))
    }

    pub async fn migrate(&self) -> Result<()> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }
}

// --- row types -------------------------------------------------------------

/// A raw `objects` row with primitive column types.
#[derive(sqlx::FromRow)]
struct ObjectRow {
    id: Vec<u8>,
    label: String,
    name: String,
    properties: Option<String>,
    created_at: i64,
    updated_at: Option<i64>,
}

impl ObjectRow {
    fn into_object(self) -> Result<Object> {
        Ok(Object {
            id: uuid_from_bytes(&self.id)?,
            label: ObjectLabel::from_str(&self.label)
                .map_err(|_| Error::generic(format!("unknown object label: {}", self.label)))?,
            name: ResourceName::from_str(&self.name)
                .map_err(|e| Error::generic(format!("invalid resource name: {e}")))?,
            properties: self.properties.map(|p| parse_json(&p)).transpose()?,
            created_at: micros_to_dt(self.created_at)?,
            updated_at: self.updated_at.map(micros_to_dt).transpose()?,
        })
    }
}

/// A raw `associations` row with primitive column types.
#[derive(sqlx::FromRow)]
struct AssociationRow {
    id: Vec<u8>,
    from_id: Vec<u8>,
    label: String,
    to_id: Vec<u8>,
    to_label: String,
    properties: Option<String>,
    created_at: i64,
    updated_at: Option<i64>,
}

impl AssociationRow {
    fn into_association(self) -> Result<olai_store::Association<ObjectLabel>> {
        Ok(olai_store::Association {
            id: uuid_from_bytes(&self.id)?,
            from_id: uuid_from_bytes(&self.from_id)?,
            label: self.label,
            to_id: uuid_from_bytes(&self.to_id)?,
            to_label: ObjectLabel::from_str(&self.to_label)
                .map_err(|_| Error::generic(format!("unknown object label: {}", self.to_label)))?,
            properties: self.properties.map(|p| parse_json(&p)).transpose()?,
            created_at: micros_to_dt(self.created_at)?,
            updated_at: self.updated_at.map(micros_to_dt).transpose()?,
        })
    }
}

// --- conversion helpers ----------------------------------------------------

fn uuid_from_bytes(bytes: &[u8]) -> Result<Uuid> {
    Uuid::from_slice(bytes).map_err(|e| Error::generic(format!("invalid uuid bytes: {e}")))
}

fn micros_to_dt(micros: i64) -> Result<DateTime<Utc>> {
    DateTime::from_timestamp_micros(micros)
        .ok_or_else(|| Error::generic("invalid timestamp in database"))
}

fn parse_json(s: &str) -> Result<serde_json::Value> {
    serde_json::from_str(s).map_err(|e| Error::generic(format!("invalid json in database: {e}")))
}

fn json_to_string(value: &serde_json::Value) -> Result<String> {
    serde_json::to_string(value)
        .map_err(|e| Error::generic(format!("failed to serialize json: {e}")))
}

/// Encode the parent path (all but the last segment) of a resource name.
///
/// This is stored in the denormalized `namespace` column so that listing the
/// direct children of a namespace is an indexed equality match rather than an
/// array-slice prefix scan.
fn namespace_of(name: &ResourceName) -> String {
    let path = name.path();
    if path.is_empty() {
        return String::new();
    }
    ResourceName::new(path[..path.len() - 1].iter().cloned()).to_string()
}

// --- inherent CRUD ---------------------------------------------------------

impl SqliteStore {
    pub async fn add_object(
        &self,
        label: &ObjectLabel,
        name: &ResourceName,
        properties: Option<serde_json::Value>,
        id: Option<Uuid>,
    ) -> Result<Object> {
        // Callers may pre-allocate an id (e.g. a managed volume embedding the id
        // in its storage path); otherwise generate a time-ordered UUIDv7.
        let id = id.unwrap_or_else(Uuid::now_v7);
        let id_bytes = id.as_bytes().to_vec();
        let label_str = label.to_string();
        let name_str = name.to_string();
        let namespace_str = namespace_of(name);
        let properties_str = properties.as_ref().map(json_to_string).transpose()?;
        let created_at = Utc::now().timestamp_micros();

        let row = sqlx::query_as!(
            ObjectRow,
            r#"
            INSERT INTO objects ( id, label, name, namespace, properties, created_at )
            VALUES ( ?, ?, ?, ?, ?, ? )
            RETURNING
                id AS "id!",
                label AS "label!",
                name AS "name!",
                properties,
                created_at AS "created_at!",
                updated_at
            "#,
            id_bytes,
            label_str,
            name_str,
            namespace_str,
            properties_str,
            created_at,
        )
        .fetch_one(&self.pool)
        .await?;
        row.into_object()
    }

    pub async fn get_object(&self, id: &Uuid) -> Result<Object> {
        let id_bytes = id.as_bytes().to_vec();
        let row = sqlx::query_as!(
            ObjectRow,
            r#"
            SELECT
                id AS "id!",
                label AS "label!",
                name AS "name!",
                properties,
                created_at AS "created_at!",
                updated_at
            FROM objects
            WHERE id = ?
            "#,
            id_bytes,
        )
        .fetch_one(&self.pool)
        .await?;
        row.into_object()
    }

    pub async fn get_object_by_name(
        &self,
        label: &ObjectLabel,
        name: &ResourceName,
    ) -> Result<Object> {
        let label_str = label.to_string();
        let name_str = name.to_string();
        let row = sqlx::query_as!(
            ObjectRow,
            r#"
            SELECT
                id AS "id!",
                label AS "label!",
                name AS "name!",
                properties,
                created_at AS "created_at!",
                updated_at
            FROM objects
            WHERE label = ? AND name = ?
            "#,
            label_str,
            name_str,
        )
        .fetch_one(&self.pool)
        .await?;
        row.into_object()
    }

    pub async fn update_object(
        &self,
        id: &Uuid,
        properties: Option<serde_json::Value>,
    ) -> Result<Object> {
        let id_bytes = id.as_bytes().to_vec();
        let properties_str = properties.as_ref().map(json_to_string).transpose()?;
        let updated_at = Utc::now().timestamp_micros();
        let row = sqlx::query_as!(
            ObjectRow,
            r#"
            UPDATE objects
            SET properties = COALESCE(?, properties),
                updated_at = ?
            WHERE id = ?
            RETURNING
                id AS "id!",
                label AS "label!",
                name AS "name!",
                properties,
                created_at AS "created_at!",
                updated_at
            "#,
            properties_str,
            updated_at,
            id_bytes,
        )
        .fetch_one(&self.pool)
        .await?;
        row.into_object()
    }

    pub async fn delete_object(&self, id: &Uuid) -> Result<()> {
        let id_bytes = id.as_bytes().to_vec();
        let mut txn = self.pool.begin().await?;
        sqlx::query!(
            "DELETE FROM associations WHERE from_id = ? OR to_id = ?",
            id_bytes,
            id_bytes,
        )
        .execute(&mut *txn)
        .await?;
        sqlx::query!("DELETE FROM objects WHERE id = ?", id_bytes)
            .execute(&mut *txn)
            .await?;
        txn.commit().await?;
        Ok(())
    }

    pub async fn list_objects(
        &self,
        label: &ObjectLabel,
        namespace: &ResourceName,
        page_token: Option<&str>,
        max_page_size: Option<usize>,
    ) -> Result<(Vec<Object>, Option<String>)> {
        let max_page_size = usize::min(max_page_size.unwrap_or(MAX_PAGE_SIZE), MAX_PAGE_SIZE);
        let token_id = decode_token(page_token)?;
        let token_bytes = token_id.map(|id| id.as_bytes().to_vec());

        let label_str = label.to_string();
        let namespace_str = namespace.to_string();
        let limit = max_page_size as i64;

        // `namespace = ?` lists the direct children of the namespace. Keyset
        // pagination orders by the time-sortable UUIDv7 id descending; the
        // `? IS NULL` guards make the token optional within one static query.
        let rows = sqlx::query_as!(
            ObjectRow,
            r#"
            SELECT
                id AS "id!",
                label AS "label!",
                name AS "name!",
                properties,
                created_at AS "created_at!",
                updated_at
            FROM objects
            WHERE label = ?
              AND namespace = ?
              AND ( ? IS NULL OR id < ? )
            ORDER BY id DESC
            LIMIT ?
            "#,
            label_str,
            namespace_str,
            token_bytes,
            token_bytes,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        let objects = rows
            .into_iter()
            .map(ObjectRow::into_object)
            .collect::<Result<Vec<_>>>()?;
        let next = next_token(&objects, max_page_size, |o| (o.created_at, o.id));
        Ok((objects, next))
    }

    pub async fn add_association(
        &self,
        from_id: &Uuid,
        label: &AssociationLabel,
        to_id: &Uuid,
        properties: Option<serde_json::Value>,
    ) -> Result<()> {
        let from_bytes = from_id.as_bytes().to_vec();
        let to_bytes = to_id.as_bytes().to_vec();

        let mut txn = self.pool.begin().await?;

        // Look up the labels of both endpoints (and confirm they exist).
        let from_label = object_label(&from_bytes, &mut txn)
            .await?
            .ok_or_else(|| Error::entity_not_found("from_id"))?;
        let to_label = object_label(&to_bytes, &mut txn)
            .await?
            .ok_or_else(|| Error::entity_not_found("to_id"))?;

        let properties_str = properties.as_ref().map(json_to_string).transpose()?;

        insert_association(
            &from_bytes,
            label.as_ref(),
            &to_bytes,
            &to_label,
            properties_str.as_deref(),
            &mut txn,
        )
        .await?;

        // Mirror the edge so the graph is navigable in both directions.
        if let Some(inverse) = label.inverse() {
            insert_association(
                &to_bytes,
                inverse.as_ref(),
                &from_bytes,
                &from_label,
                properties_str.as_deref(),
                &mut txn,
            )
            .await?;
        }

        txn.commit().await?;
        Ok(())
    }

    pub async fn delete_association(
        &self,
        from_id: &Uuid,
        label: &AssociationLabel,
        to_id: &Uuid,
    ) -> Result<()> {
        let from_bytes = from_id.as_bytes().to_vec();
        let to_bytes = to_id.as_bytes().to_vec();
        let label_str = label.to_string();

        let mut txn = self.pool.begin().await?;
        sqlx::query!(
            "DELETE FROM associations WHERE from_id = ? AND label = ? AND to_id = ?",
            from_bytes,
            label_str,
            to_bytes,
        )
        .execute(&mut *txn)
        .await?;
        if let Some(inverse) = label.inverse() {
            let inverse_str = inverse.to_string();
            sqlx::query!(
                "DELETE FROM associations WHERE from_id = ? AND label = ? AND to_id = ?",
                to_bytes,
                inverse_str,
                from_bytes,
            )
            .execute(&mut *txn)
            .await?;
        }
        txn.commit().await?;
        Ok(())
    }

    pub async fn list_associations(
        &self,
        from_id: &Uuid,
        label: &AssociationLabel,
        target_label: Option<&ObjectLabel>,
        page_token: Option<&str>,
        max_page_size: Option<usize>,
    ) -> Result<(Vec<olai_store::Association<ObjectLabel>>, Option<String>)> {
        let max_page_size = usize::min(max_page_size.unwrap_or(MAX_PAGE_SIZE), MAX_PAGE_SIZE);
        let token_id = decode_token(page_token)?;
        let token_bytes = token_id.map(|id| id.as_bytes().to_vec());

        let from_bytes = from_id.as_bytes().to_vec();
        let label_str = label.to_string();
        let target_label_str = target_label.map(|l| l.to_string());
        let limit = max_page_size as i64;

        let rows = sqlx::query_as!(
            AssociationRow,
            r#"
            SELECT
                id AS "id!",
                from_id AS "from_id!",
                label AS "label!",
                to_id AS "to_id!",
                to_label AS "to_label!",
                properties,
                created_at AS "created_at!",
                updated_at
            FROM associations
            WHERE from_id = ?
              AND label = ?
              AND ( ? IS NULL OR to_label = ? )
              AND ( ? IS NULL OR id < ? )
            ORDER BY id DESC
            LIMIT ?
            "#,
            from_bytes,
            label_str,
            target_label_str,
            target_label_str,
            token_bytes,
            token_bytes,
            limit,
        )
        .fetch_all(&self.pool)
        .await?;

        let associations = rows
            .into_iter()
            .map(AssociationRow::into_association)
            .collect::<Result<Vec<_>>>()?;
        let next = next_token(&associations, max_page_size, |a| (a.created_at, a.id));
        Ok((associations, next))
    }
}

// --- free helpers ----------------------------------------------------------

fn decode_token(page_token: Option<&str>) -> Result<Option<Uuid>> {
    let token = page_token
        .map(PaginateToken::<Uuid>::try_from)
        .transpose()?;
    Ok(token.map(|PaginateToken::V1(V1PaginateToken { id, .. })| id))
}

/// Build the next-page token from the last item of a full page.
fn next_token<T>(
    items: &[T],
    max_page_size: usize,
    key: impl Fn(&T) -> (DateTime<Utc>, Uuid),
) -> Option<String> {
    if items.len() != max_page_size {
        return None;
    }
    items.last().map(|item| {
        let (created_at, id) = key(item);
        PaginateToken::V1(V1PaginateToken { created_at, id }).to_string()
    })
}

async fn object_label(
    id_bytes: &[u8],
    txn: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<Option<String>> {
    let label = sqlx::query_scalar!("SELECT label FROM objects WHERE id = ?", id_bytes)
        .fetch_optional(&mut **txn)
        .await?;
    Ok(label)
}

async fn insert_association(
    from_bytes: &[u8],
    label: &str,
    to_bytes: &[u8],
    to_label: &str,
    properties: Option<&str>,
    txn: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<()> {
    let id = Uuid::now_v7().as_bytes().to_vec();
    let created_at = Utc::now().timestamp_micros();
    sqlx::query!(
        r#"
        INSERT INTO associations ( id, from_id, label, to_id, to_label, properties, created_at )
        VALUES ( ?, ?, ?, ?, ?, ?, ? )
        "#,
        id,
        from_bytes,
        label,
        to_bytes,
        to_label,
        properties,
        created_at,
    )
    .execute(&mut **txn)
    .await?;
    Ok(())
}

// --- ObjectStore<ObjectLabel> ----------------------------------------------

#[async_trait::async_trait]
impl olai_store::ObjectStoreReader<ObjectLabel> for SqliteStore {
    async fn get(&self, id: &Uuid) -> olai_store::Result<Object> {
        Ok(self.get_object(id).await?)
    }

    async fn get_by_name(
        &self,
        label: ObjectLabel,
        name: &ResourceName,
    ) -> olai_store::Result<Object> {
        Ok(self.get_object_by_name(&label, name).await?)
    }

    async fn list(
        &self,
        label: ObjectLabel,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> olai_store::Result<(Vec<Object>, Option<String>)> {
        let namespace = namespace.unwrap_or(&EMPTY_RESOURCE_NAME);
        Ok(self
            .list_objects(&label, namespace, page_token.as_deref(), max_results)
            .await?)
    }
}

#[async_trait::async_trait]
impl olai_store::ObjectStore<ObjectLabel> for SqliteStore {
    async fn create(
        &self,
        label: ObjectLabel,
        name: &ResourceName,
        properties: Option<serde_json::Value>,
        id: Option<Uuid>,
    ) -> olai_store::Result<Object> {
        Ok(self.add_object(&label, name, properties, id).await?)
    }

    async fn update(
        &self,
        id: &Uuid,
        properties: Option<serde_json::Value>,
    ) -> olai_store::Result<Object> {
        Ok(self.update_object(id, properties).await?)
    }

    async fn delete(&self, id: &Uuid) -> olai_store::Result<()> {
        Ok(self.delete_object(id).await?)
    }
}

// --- AssociationStore<ObjectLabel> -----------------------------------------

#[async_trait::async_trait]
impl olai_store::AssociationStoreReader<ObjectLabel> for SqliteStore {
    async fn list(
        &self,
        from_id: Uuid,
        label: &str,
        target_label: Option<ObjectLabel>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> olai_store::Result<(Vec<olai_store::Association<ObjectLabel>>, Option<String>)> {
        let assoc_label: AssociationLabel = label.parse().map_err(|_| {
            olai_store::Error::InvalidArgument(format!("Unknown association label: {label}"))
        })?;
        Ok(self
            .list_associations(
                &from_id,
                &assoc_label,
                target_label.as_ref(),
                page_token.as_deref(),
                max_results,
            )
            .await?)
    }
}

#[async_trait::async_trait]
impl olai_store::AssociationStore<ObjectLabel> for SqliteStore {
    async fn add(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        label: &str,
        properties: Option<serde_json::Value>,
    ) -> olai_store::Result<()> {
        let assoc_label: AssociationLabel = label.parse().map_err(|_| {
            olai_store::Error::InvalidArgument(format!("Unknown association label: {label}"))
        })?;
        self.add_association(&from_id, &assoc_label, &to_id, properties)
            .await?;
        Ok(())
    }

    async fn remove(&self, from_id: Uuid, to_id: Uuid, label: &str) -> olai_store::Result<()> {
        let assoc_label: AssociationLabel = label.parse().map_err(|_| {
            olai_store::Error::InvalidArgument(format!("Unknown association label: {label}"))
        })?;
        self.delete_association(&from_id, &assoc_label, &to_id)
            .await?;
        Ok(())
    }
}
