//! Storage layer for managing objects and associations.
//!
//! Lossely based on the data model applied in Meta's [TAO].
//!
//! [TAO]: https://www.usenix.org/system/files/conference/atc13/atc13-bronson.pdf

use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use unitycatalog_common::{ResourceIdent, ResourceRef};
use uuid::Uuid;

use super::{Association, AssociationLabel, Object, ObjectLabel};
use crate::constants::MAX_PAGE_SIZE;
use crate::pagination::V1PaginateToken;
use crate::resources::IdentRefs as _;
use crate::{error::Result, pagination::PaginateToken};

static MIGRATOR: Migrator = sqlx::migrate!();

use sqlx_pg as pg_impl;

#[derive(Clone)]
pub struct Store {
    pub(crate) pool: PgPool,
}

impl Store {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect(url: impl AsRef<str>) -> Result<Self> {
        let options: PgConnectOptions = url.as_ref().parse()?;
        let pool_options = PgPoolOptions::new().max_connections(96);
        let pool = pool_options.connect_with(options).await?;
        Ok(Self::new(pool))
    }

    pub async fn migrate(&self) -> Result<()> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }

    /// Convert a resource reference to a UUID.
    ///
    /// If the reference is a name, the corresponding object is fetched from the store.
    /// to get the UUID. The object is returned as well in case it is needed later
    /// to avoid an additional fetch.
    ///
    /// # Parameters
    /// - `reference`: The reference to convert.
    ///
    /// # Returns
    /// The UUID of the reference and the object if the reference is a name.
    ///
    /// # Errors
    /// In case of an undefined reference, an error is returned.
    pub async fn ident_to_uuid(&self, reference: &ResourceIdent) -> Result<(Uuid, Option<Object>)> {
        let (label, ident) = reference.ident();
        match ident {
            ResourceRef::Uuid(id) => Ok((*id, None)),
            ResourceRef::Name(name) => {
                let object = self.get_object_by_name(label, name).await?;
                Ok((object.id, Some(object)))
            }
            ResourceRef::Undefined => Err(crate::Error::entity_not_found("undefined")),
        }
    }

    /// Add an object to the store.
    ///
    /// # Parameters
    /// - `label`: The label of the object.
    /// - `name`: The namespaced name of the object.
    /// - `name`: The name of the object.
    /// - `properties`: The properties of the object.
    ///
    /// # Returns
    /// The object that was added to the store.
    ///
    /// # Errors
    /// - [AlreadyExists](crate::Error::AlreadyExists): If an object with the
    ///   same name already exists in the namespace
    pub async fn add_object(
        &self,
        label: &ObjectLabel,
        name: &[String],
        properties: Option<serde_json::Value>,
    ) -> Result<Object> {
        let mut txn = self.pool.begin().await?;
        let obj = pg_impl::add_object(label, name, properties, &mut txn).await?;
        txn.commit().await?;
        Ok(obj)
    }

    /// Get an object from the store.
    ///
    /// # Parameters
    /// - `id`: The globally unique identifier of the object.
    ///
    /// # Returns
    /// The object with the given identifier.
    ///
    /// # Errors
    /// - [EntityNotFound](crate::Error::EntityNotFound): If the object does not exist.
    pub async fn get_object(&self, id: &Uuid) -> Result<Object> {
        let mut conn = self.pool.acquire().await?;
        let obj = pg_impl::get_object(id, &mut conn).await?;
        Ok(obj)
    }

    /// Get an object from the store by name.
    ///
    /// The name of the object is unique within the namespace.
    ///
    /// # Parameters
    /// - `label`: The label of the object.
    /// - `namespace`: The namespace of the object.
    /// - `name`: The name of the object.
    ///
    /// # Returns
    /// The object with the given name.
    ///
    /// # Errors
    /// - [EntityNotFound](crate::Error::EntityNotFound): If the object does not exist.
    pub async fn get_object_by_name(&self, label: &ObjectLabel, name: &[String]) -> Result<Object> {
        let mut conn = self.pool.acquire().await?;
        let obj = pg_impl::get_object_by_name(label, name, &mut conn).await?;
        Ok(obj)
    }

    /// Update an object in the store.
    ///
    /// # Parameters
    /// - `id`: The globally unique identifier of the object.
    /// - `properties`: The properties of the object.
    ///
    /// # Returns
    /// The updated object.
    ///
    /// # Errors
    /// - [EntityNotFound](crate::Error::EntityNotFound): If the object does not exist.
    pub async fn update_object(
        &self,
        id: &Uuid,
        new_label: impl Into<Option<&ObjectLabel>>,
        new_name: impl Into<Option<&[String]>>,
        properties: impl Into<Option<serde_json::Value>>,
    ) -> Result<Object> {
        let mut txn = self.pool.begin().await?;
        let obj = pg_impl::update_object(id, new_label, new_name, properties, &mut txn).await?;
        txn.commit().await?;
        Ok(obj)
    }

    /// Delete an object from the store.
    ///
    /// # Parameters
    /// - `id`: The globally unique identifier of the object.
    pub async fn delete_object(&self, id: &Uuid) -> Result<()> {
        let mut txn = self.pool.begin().await?;
        pg_impl::delete_object(id, &mut txn).await?;
        Ok(txn.commit().await?)
    }

    /// List objects from the store.
    ///
    /// Returns a list of objects in the namespace. The list is paginated.
    ///
    /// # Parameters
    /// - `label`: The label of the objects.
    /// - `namespace`: The namespace of the objects.
    /// - `page_token`: The page token.
    /// - `max_page_size`: The maximum page size.
    ///
    /// # Returns
    /// A tuple containing the objects in the namespace and an optional next page token.
    pub async fn list_objects(
        &self,
        label: &ObjectLabel,
        namespace: &[String],
        page_token: Option<&str>,
        max_page_size: Option<usize>,
    ) -> Result<(Vec<Object>, Option<String>)> {
        let max_page_size = usize::min(max_page_size.unwrap_or(MAX_PAGE_SIZE), MAX_PAGE_SIZE);
        let token = page_token
            .map(PaginateToken::<Uuid>::try_from)
            .transpose()?;
        let (_token_ts, token_id) = token
            .as_ref()
            .map(|PaginateToken::V1(V1PaginateToken { created_at, id })| (created_at, id))
            .unzip();
        let mut conn = self.pool.acquire().await?;
        pg_impl::list_objects(label, namespace, max_page_size, token_id, &mut conn).await
    }

    /// Add an association to the store.
    ///
    /// Associations are directed edges between objects.
    /// If an inverse association exists, it is automatically created.
    ///
    /// # Parameters
    /// - `from_id`: The identifier of the source object.
    /// - `label`: The label of the association.
    /// - `to_id`: The identifier of the target object.
    /// - `properties`: The properties of the association.
    ///
    /// # Returns
    /// The association that was added to the store.
    ///
    /// # Errors
    /// - [EntityNotFound](crate::Error::EntityNotFound): If the source or target object does not exist.
    /// - [AlreadyExists](crate::Error::AlreadyExists): If the association already exists.
    pub async fn add_association(
        &self,
        from_id: &Uuid,
        label: &AssociationLabel,
        to_id: &Uuid,
        properties: impl Into<Option<serde_json::Value>>,
    ) -> Result<Association> {
        let mut txn = self.pool.begin().await?;
        let association =
            pg_impl::add_association(from_id, label, to_id, properties, &mut txn).await?;
        txn.commit().await?;
        Ok(association)
    }

    /// Delete an association from the store.
    ///
    /// If an inverse association exists, it is automatically deleted.
    pub async fn delete_association(
        &self,
        from_id: &Uuid,
        label: &AssociationLabel,
        to_id: &Uuid,
    ) -> Result<()> {
        let mut txn = self.pool.begin().await?;
        pg_impl::delete_association(from_id, label, to_id, &mut txn).await?;
        txn.commit().await?;
        Ok(())
    }

    /// List associations of a specific type from an object to a set of objects.
    ///
    /// # Parameters
    /// - `from_id`: The identifier of the source object.
    /// - `label`: The label of the association.
    /// - `to_ids`: The identifiers of the target objects.
    ///
    /// # Returns
    /// The associations from the source object to the target objects.
    pub async fn get_associations(
        &self,
        from_id: &Uuid,
        label: &AssociationLabel,
        to_ids: &[Uuid],
        page_token: Option<&str>,
        max_page_size: Option<usize>,
    ) -> Result<(Vec<Association>, Option<String>)> {
        let max_page_size = usize::min(max_page_size.unwrap_or(MAX_PAGE_SIZE), MAX_PAGE_SIZE);
        let token = page_token
            .map(PaginateToken::<Uuid>::try_from)
            .transpose()?;
        let (_token_ts, token_id) = token
            .as_ref()
            .map(|PaginateToken::V1(V1PaginateToken { created_at, id })| (created_at, id))
            .unzip();
        let mut conn = self.pool.acquire().await?;
        pg_impl::get_associations(from_id, label, to_ids, max_page_size, token_id, &mut conn).await
    }

    /// List associations of a specific type from an object to all objects.
    pub async fn list_associations(
        &self,
        from_id: &Uuid,
        label: &AssociationLabel,
        target_label: Option<&ObjectLabel>,
        page_token: Option<&str>,
        max_page_size: Option<usize>,
    ) -> Result<(Vec<Association>, Option<String>)> {
        let max_page_size = usize::min(max_page_size.unwrap_or(MAX_PAGE_SIZE), MAX_PAGE_SIZE);
        let token = page_token
            .map(PaginateToken::<Uuid>::try_from)
            .transpose()?;
        let (_token_ts, token_id) = token
            .as_ref()
            .map(
                |PaginateToken::V1(V1PaginateToken { created_at, id }): &PaginateToken<Uuid>| {
                    (created_at, id)
                },
            )
            .unzip();
        let mut conn = self.pool.acquire().await?;
        pg_impl::list_associations(
            from_id,
            label,
            target_label,
            max_page_size,
            token_id,
            &mut conn,
        )
        .await
    }
}

mod sqlx_pg {
    use super::*;

    pub(super) async fn list_associations(
        from_id: &Uuid,
        label: &AssociationLabel,
        target_label: Option<&ObjectLabel>,
        max_page_size: usize,
        token_id: Option<&Uuid>,
        conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> std::result::Result<(Vec<Association>, Option<String>), crate::Error> {
        let assocs = sqlx::query_as!(
            Association,
            r#"
                SELECT
                    id,
                    from_id,
                    label AS "label: AssociationLabel",
                    to_id,
                    properties,
                    created_at,
                    updated_at,
                    to_label as "to_label: ObjectLabel"
                FROM associations
                WHERE from_id = $1
                  AND label = $2
                  AND ( to_label = $3 OR $3 IS NULL )
                  -- Pagination
                  AND ( id < $4 OR $4 IS NULL )
                ORDER BY id DESC
                LIMIT $5
                "#,
            from_id,
            label as &AssociationLabel,
            target_label as Option<&ObjectLabel>,
            token_id,
            max_page_size as i64
        )
        .fetch_all(&mut **conn)
        .await?;

        let next = (assocs.len() == max_page_size)
            .then(|| {
                assocs.last().map(|a| {
                    PaginateToken::V1(V1PaginateToken {
                        created_at: a.created_at,
                        id: a.id,
                    })
                    .to_string()
                })
            })
            .flatten();

        Ok((assocs, next))
    }

    pub(super) async fn add_object(
        label: &ObjectLabel,
        name: &[String],
        properties: Option<serde_json::Value>,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Object, crate::Error> {
        let obj = sqlx::query_as!(
            Object,
            r#"
                INSERT INTO objects ( label, name, properties )
                VALUES ( $1, $2, $3 )
                RETURNING
                    id,
                    label AS "label: ObjectLabel",
                    name,
                    properties,
                    created_at,
                    updated_at
                "#,
            label as &ObjectLabel,
            name,
            properties
        )
        .fetch_one(&mut **txn)
        .await?;
        Ok(obj)
    }

    pub(super) async fn get_object(
        id: &Uuid,
        conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Object, crate::Error> {
        let obj = sqlx::query_as!(
            Object,
            r#"
                SELECT
                    id,
                    label AS "label: ObjectLabel",
                    name,
                    properties,
                    created_at,
                    updated_at
                FROM objects
                WHERE id = $1
                "#,
            id
        )
        .fetch_one(&mut **conn)
        .await?;
        Ok(obj)
    }

    pub(super) async fn get_object_by_name(
        label: &ObjectLabel,
        name: &[String],
        conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> Result<Object, crate::Error> {
        let obj = sqlx::query_as!(
            Object,
            r#"
                SELECT
                    id,
                    label AS "label: ObjectLabel",
                    name,
                    properties,
                    created_at,
                    updated_at
                FROM objects
                WHERE label = $1
                  AND name = $2
                "#,
            label as &ObjectLabel,
            name
        )
        .fetch_one(&mut **conn)
        .await?;
        Ok(obj)
    }

    pub(super) async fn update_object(
        id: &Uuid,
        new_label: impl Into<Option<&ObjectLabel>>,
        new_name: impl Into<Option<&[String]>>,
        properties: impl Into<Option<serde_json::Value>>,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Object, crate::Error> {
        let obj = sqlx::query_as!(
            Object,
            r#"
                UPDATE objects
                SET
                    label = COALESCE($2, label),
                    name = COALESCE($3, name),
                    properties = COALESCE($4, properties)
                WHERE id = $1
                RETURNING
                    id,
                    label AS "label: ObjectLabel",
                    name,
                    properties,
                    created_at,
                    updated_at
                "#,
            id,
            new_label.into() as Option<&ObjectLabel>,
            new_name.into(),
            properties.into()
        )
        .fetch_one(&mut **txn)
        .await?;
        Ok(obj)
    }

    pub(super) async fn delete_object(
        id: &Uuid,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), crate::Error> {
        sqlx::query!(
            r#"
                DELETE FROM associations
                WHERE from_id = $1 OR to_id = $1
                "#,
            id
        )
        .execute(&mut **txn)
        .await?;
        sqlx::query!(
            r#"
                DELETE FROM objects
                WHERE id = $1
                "#,
            id
        )
        .execute(&mut **txn)
        .await?;
        Ok(())
    }

    pub(super) async fn list_objects(
        label: &ObjectLabel,
        namespace: &[String],
        max_page_size: usize,
        token_id: Option<&Uuid>,
        conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> std::result::Result<(Vec<Object>, Option<String>), crate::Error> {
        let objects = sqlx::query_as!(
            Object,
            r#"
                SELECT
                    id,
                    label AS "label: ObjectLabel",
                    name,
                    properties,
                    created_at,
                    updated_at
                FROM objects
                WHERE label = $1
                    AND ( $2 = 0 OR name[1:$2] = $3)
                    AND ( id < $4 OR $4 IS NULL )
                ORDER BY id DESC
                LIMIT $5
                "#,
            label as &ObjectLabel,
            namespace.len() as i32,
            namespace,
            token_id,
            max_page_size as i64
        )
        .fetch_all(&mut **conn)
        .await?;
        dbg!("list - done");

        let next = (objects.len() == max_page_size)
            .then(|| {
                objects.last().map(|o| {
                    PaginateToken::V1(V1PaginateToken {
                        created_at: o.created_at,
                        id: o.id,
                    })
                    .to_string()
                })
            })
            .flatten();

        Ok((objects, next))
    }

    pub(super) async fn get_associations(
        from_id: &Uuid,
        label: &AssociationLabel,
        to_ids: &[Uuid],
        max_page_size: usize,
        token_id: Option<&Uuid>,
        conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    ) -> std::result::Result<(Vec<Association>, Option<String>), crate::Error> {
        let assocs = sqlx::query_as!(
            Association,
            r#"
                SELECT
                    id,
                    from_id,
                    label AS "label: AssociationLabel",
                    to_id,
                    properties,
                    created_at,
                    updated_at,
                    to_label as "to_label: ObjectLabel"
                FROM associations
                WHERE from_id = $1
                  AND label = $2
                  AND to_id = ANY($3)
                  -- Pagination
                  AND ( id < $4 OR $4 IS NULL )
                ORDER BY id DESC
                LIMIT $5
                "#,
            from_id,
            label as &AssociationLabel,
            &to_ids,
            token_id,
            max_page_size as i64
        )
        .fetch_all(&mut **conn)
        .await?;

        let next = (assocs.len() == max_page_size)
            .then(|| {
                assocs.last().map(|a| {
                    PaginateToken::V1(V1PaginateToken {
                        created_at: a.created_at,
                        id: a.id,
                    })
                    .to_string()
                })
            })
            .flatten();

        Ok((assocs, next))
    }

    pub async fn add_association(
        from_id: &Uuid,
        label: &AssociationLabel,
        to_id: &Uuid,
        properties: impl Into<Option<serde_json::Value>>,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<Association> {
        let properties = properties.into();
        let to_label = sqlx::query!(
            r#"
            SELECT id, label AS "label: ObjectLabel"
            FROM objects
            WHERE id = $1 OR id = $2
            "#,
            from_id,
            to_id
        )
        .fetch_all(&mut **txn)
        .await?;

        let id_map = to_label
            .into_iter()
            .map(|o| (o.id, o.label))
            .collect::<std::collections::HashMap<_, _>>();
        let to_label = id_map
            .get(to_id)
            .ok_or(crate::Error::entity_not_found("to_id"))?;
        let from_label = id_map
            .get(from_id)
            .ok_or(crate::Error::entity_not_found("from_id"))?;

        // Add the association.
        let association = sqlx::query_as!(
            Association,
            r#"
            INSERT INTO associations ( from_id, label, to_id, to_label, properties )
            VALUES ( $1, $2, $3, $4, $5 )
            RETURNING
                id,
                from_id,
                label AS "label: AssociationLabel",
                to_id,
                to_label as "to_label: ObjectLabel",
                properties,
                created_at,
                updated_at
            "#,
            from_id,
            label as &AssociationLabel,
            to_id,
            to_label as &ObjectLabel,
            properties.clone()
        )
        .fetch_one(&mut **txn)
        .await?;

        // Add the inverse association.
        if let Some(inverse_label) = label.inverse() {
            sqlx::query!(
                r#"
                INSERT INTO associations ( from_id, label, to_id, to_label, properties )
                VALUES ( $1, $2, $3, $4, $5 )
                "#,
                to_id,
                inverse_label as AssociationLabel,
                from_id,
                from_label as &ObjectLabel,
                properties
            )
            .execute(&mut **txn)
            .await?;
        }

        Ok(association)
    }

    pub(super) async fn delete_association(
        from_id: &Uuid,
        label: &AssociationLabel,
        to_id: &Uuid,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ) -> Result<(), crate::error::Error> {
        sqlx::query!(
            r#"
            DELETE FROM associations
            WHERE from_id = $1 AND label = $2 AND to_id = $3
            "#,
            from_id,
            label as &AssociationLabel,
            to_id
        )
        .execute(&mut **txn)
        .await?;
        if let Some(inverse_label) = label.inverse() {
            sqlx::query!(
                r#"
                DELETE FROM associations
                WHERE from_id = $1 AND label = $2 AND to_id = $3
                "#,
                to_id,
                inverse_label as AssociationLabel,
                from_id
            )
            .execute(&mut **txn)
            .await?;
        };
        Ok(())
    }
}

mod tokio_pg {
    use super::*;

    use tokio_postgres::Transaction;

    fn row_to_obj(row: tokio_postgres::Row) -> Result<Object> {
        Ok(Object {
            id: row.try_get("id")?,
            label: row.try_get("label")?,
            name: row.try_get("name")?,
            properties: row.try_get("properties")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }

    fn row_to_association(row: tokio_postgres::Row) -> Result<Association> {
        Ok(Association {
            id: row.try_get("id")?,
            from_id: row.try_get("from_id")?,
            label: row.try_get("label")?,
            to_id: row.try_get("to_id")?,
            properties: row.try_get("properties")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            to_label: row.try_get("to_label")?,
        })
    }

    pub(super) async fn add_object(
        label: &ObjectLabel,
        name: &[String],
        properties: Option<serde_json::Value>,
        txn: &mut Transaction<'_>,
    ) -> Result<Object, crate::Error> {
        let row = txn
            .query_one(
                "INSERT INTO objects (label, name, properties)
             VALUES ($1, $2, $3)
             RETURNING id, label, name, properties, created_at, updated_at",
                &[&label, &name, &properties],
            )
            .await?;
        row_to_obj(row)
    }

    pub(super) async fn get_object(
        id: &Uuid,
        conn: &tokio_postgres::Client,
    ) -> Result<Object, crate::Error> {
        let row = conn
            .query_one(
                "SELECT
                    id,
                    label,
                    name,
                    properties,
                    created_at,
                    updated_at
                FROM objects
                WHERE id = $1",
                &[&id],
            )
            .await?;
        row_to_obj(row)
    }

    pub(super) async fn get_object_by_name(
        label: &ObjectLabel,
        name: &[String],
        conn: &tokio_postgres::Client,
    ) -> Result<Object, crate::Error> {
        let row = conn
            .query_one(
                "SELECT
                    id,
                    label,
                    name,
                    properties,
                    created_at,
                    updated_at
                FROM objects
                WHERE label = $1
                  AND name = $2",
                &[&label, &name],
            )
            .await?;
        row_to_obj(row)
    }

    pub(super) async fn update_object(
        id: &Uuid,
        new_label: impl Into<Option<&ObjectLabel>>,
        new_name: impl Into<Option<&[String]>>,
        properties: impl Into<Option<serde_json::Value>>,
        txn: &mut Transaction<'_>,
    ) -> Result<Object, crate::Error> {
        let new_label = new_label.into();
        let new_name = new_name.into();
        let properties = properties.into();

        let row = txn
            .query_one(
                "UPDATE objects
                SET
                    label = COALESCE($2, label),
                    name = COALESCE($3, name),
                    properties = COALESCE($4, properties)
                WHERE id = $1
                RETURNING
                    id,
                    label,
                    name,
                    properties,
                    created_at,
                    updated_at",
                &[&id, &new_label, &new_name, &properties],
            )
            .await?;

        row_to_obj(row)
    }

    pub(super) async fn delete_object(
        id: &Uuid,
        txn: &mut Transaction<'_>,
    ) -> Result<(), crate::Error> {
        // First delete all associations involving this object
        txn.execute(
            "DELETE FROM associations
             WHERE from_id = $1 OR to_id = $1",
            &[&id],
        )
        .await?;

        // Then delete the object itself
        txn.execute(
            "DELETE FROM objects
             WHERE id = $1",
            &[&id],
        )
        .await?;

        Ok(())
    }

    pub(super) async fn list_objects(
        label: &ObjectLabel,
        namespace: &[String],
        max_page_size: usize,
        token_id: Option<&Uuid>,
        conn: &tokio_postgres::Client,
    ) -> std::result::Result<(Vec<Object>, Option<String>), crate::Error> {
        // Construct the query with parameterized values
        let query = "
            SELECT
                id,
                label,
                name,
                properties,
                created_at,
                updated_at
            FROM objects
            WHERE label = $1
                AND ( $2 = 0 OR name[1:$2] = $3)
                AND ( id < $4 OR $4 IS NULL )
            ORDER BY id DESC
            LIMIT $5
        ";

        // Convert namespace length to i32 for PostgreSQL
        let ns_len = namespace.len() as i32;

        // Execute the query
        let rows = conn
            .query(
                query,
                &[
                    &label,
                    &ns_len,
                    &namespace,
                    &token_id,
                    &(max_page_size as i64),
                ],
            )
            .await?;

        // Convert rows to objects
        let mut objects = Vec::with_capacity(rows.len());
        for row in rows {
            objects.push(row_to_obj(row)?);
        }

        // Generate pagination token if necessary
        let next = (objects.len() == max_page_size)
            .then(|| {
                objects.last().map(|o| {
                    PaginateToken::V1(V1PaginateToken {
                        created_at: o.created_at,
                        id: o.id,
                    })
                    .to_string()
                })
            })
            .flatten();

        Ok((objects, next))
    }

    pub(super) async fn get_associations(
        from_id: &Uuid,
        label: &AssociationLabel,
        to_ids: &[Uuid],
        max_page_size: usize,
        token_id: Option<&Uuid>,
        conn: &tokio_postgres::Client,
    ) -> std::result::Result<(Vec<Association>, Option<String>), crate::Error> {
        // Construct the query
        let query = "
            SELECT
                id,
                from_id,
                label,
                to_id,
                properties,
                created_at,
                updated_at,
                to_label
            FROM associations
            WHERE from_id = $1
              AND label = $2
              AND to_id = ANY($3)
              -- Pagination
              AND ( id < $4 OR $4 IS NULL )
            ORDER BY id DESC
            LIMIT $5
        ";

        // Execute the query
        let rows = conn
            .query(
                query,
                &[
                    &from_id,
                    &label,
                    &to_ids,
                    &token_id,
                    &(max_page_size as i64),
                ],
            )
            .await?;

        // Convert rows to associations
        let mut assocs = Vec::with_capacity(rows.len());
        for row in rows {
            assocs.push(row_to_association(row)?);
        }

        // Generate pagination token if necessary
        let next = (assocs.len() == max_page_size)
            .then(|| {
                assocs.last().map(|a| {
                    PaginateToken::V1(V1PaginateToken {
                        created_at: a.created_at,
                        id: a.id,
                    })
                    .to_string()
                })
            })
            .flatten();

        Ok((assocs, next))
    }

    pub(super) async fn add_association(
        from_id: &Uuid,
        label: &AssociationLabel,
        to_id: &Uuid,
        properties: impl Into<Option<serde_json::Value>>,
        txn: &mut Transaction<'_>,
    ) -> Result<Association> {
        let properties = properties.into();

        // Fetch the object labels for both the source and target objects
        let rows = txn
            .query(
                "SELECT id, label
                FROM objects
                WHERE id = $1 OR id = $2",
                &[&from_id, &to_id],
            )
            .await?;

        // Create a map of object IDs to their labels
        let mut id_map = std::collections::HashMap::new();
        for row in rows {
            let id: Uuid = row.try_get("id")?;
            let label: ObjectLabel = row.try_get("label")?;
            id_map.insert(id, label);
        }

        // Get the labels for the source and target objects
        let to_label = id_map
            .get(to_id)
            .ok_or(crate::Error::entity_not_found("to_id"))?;
        let from_label = id_map
            .get(from_id)
            .ok_or(crate::Error::entity_not_found("from_id"))?;

        // Add the association
        let row = txn
            .query_one(
                "INSERT INTO associations (from_id, label, to_id, to_label, properties)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING
                    id,
                    from_id,
                    label,
                    to_id,
                    to_label,
                    properties,
                    created_at,
                    updated_at",
                &[&from_id, &label, &to_id, &to_label, &properties],
            )
            .await?;

        // Add the inverse association if it exists
        if let Some(inverse_label) = label.inverse() {
            txn.execute(
                "INSERT INTO associations (from_id, label, to_id, to_label, properties)
                VALUES ($1, $2, $3, $4, $5)",
                &[&to_id, &inverse_label, &from_id, &from_label, &properties],
            )
            .await?;
        }

        row_to_association(row)
    }

    pub(super) async fn delete_association(
        from_id: &Uuid,
        label: &AssociationLabel,
        to_id: &Uuid,
        txn: &mut Transaction<'_>,
    ) -> Result<(), crate::error::Error> {
        // Delete the association
        txn.execute(
            "DELETE FROM associations
            WHERE from_id = $1 AND label = $2 AND to_id = $3",
            &[&from_id, &label, &to_id],
        )
        .await?;

        // Delete inverse association if it exists
        if let Some(inverse_label) = label.inverse() {
            txn.execute(
                "DELETE FROM associations
                WHERE from_id = $1 AND label = $2 AND to_id = $3",
                &[&to_id, &inverse_label, &from_id],
            )
            .await?;
        }

        Ok(())
    }
}
