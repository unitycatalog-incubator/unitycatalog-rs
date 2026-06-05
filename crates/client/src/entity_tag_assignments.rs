use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::tags::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub(super) use crate::codegen::entity_tag_assignments::EntityTagAssignmentClient as EntityTagAssignmentClientBase;
use crate::codegen::entity_tag_assignments::{
    CreateEntityTagAssignmentBuilder, DeleteEntityTagAssignmentBuilder,
    GetEntityTagAssignmentBuilder, UpdateEntityTagAssignmentBuilder,
};

impl EntityTagAssignmentClientBase {
    /// Stream the tag assignments for an entity.
    ///
    /// `entity_type` is one of catalogs, schemas, tables, columns, volumes; `entity_name` is the
    /// fully qualified entity name.
    pub fn list(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<EntityTagAssignment>> {
        // Thread (entity_type, entity_name, remaining) through the pagination state so the
        // closure stays `Copy` (stream_paginated requires it) instead of capturing Strings.
        let state = (entity_type.into(), entity_name.into(), max_results.into());
        stream_paginated(
            state,
            move |(entity_type, entity_name, mut max_results), page_token| async move {
                let request = ListEntityTagAssignmentsRequest {
                    entity_type: entity_type.clone(),
                    entity_name: entity_name.clone(),
                    max_results,
                    page_token,
                };
                let res = self.list_entity_tag_assignments(&request).await?;
                if let Some(ref mut remaining) = max_results {
                    *remaining -= res.tag_assignments.len() as i32;
                    if *remaining <= 0 {
                        max_results = Some(0);
                    }
                }
                Ok((
                    res.tag_assignments,
                    (entity_type, entity_name, max_results),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    /// Assign a tag to an entity (builder for the create RPC).
    pub fn assign(&self) -> CreateEntityTagAssignmentBuilder {
        CreateEntityTagAssignmentBuilder::new(self.clone())
    }

    /// Get a single tag assignment by entity and tag key.
    pub fn get(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
    ) -> GetEntityTagAssignmentBuilder {
        GetEntityTagAssignmentBuilder::new(self.clone(), entity_type, entity_name, tag_key)
    }

    /// Update a tag assignment by entity and tag key.
    pub fn update(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
    ) -> UpdateEntityTagAssignmentBuilder {
        UpdateEntityTagAssignmentBuilder::new(self.clone(), entity_type, entity_name, tag_key)
    }

    /// Delete a tag assignment by entity and tag key.
    pub fn delete(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
    ) -> DeleteEntityTagAssignmentBuilder {
        DeleteEntityTagAssignmentBuilder::new(self.clone(), entity_type, entity_name, tag_key)
    }
}
