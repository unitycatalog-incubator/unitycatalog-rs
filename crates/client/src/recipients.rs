use std::collections::HashMap;

use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::recipients::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub(super) use crate::codegen::recipients::RecipientClient as RecipientClientBase;

impl RecipientClientBase {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<RecipientInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = ListRecipientsRequest {
                max_results,
                page_token,
            };
            let res = self.list_recipients(&request).await?;
            Ok((res.recipients, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct RecipientClient {
    name: String,
    client: RecipientClientBase,
}

impl RecipientClient {
    pub fn new(name: impl ToString, client: RecipientClientBase) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    pub(super) async fn create(
        &self,
        authentication_type: AuthenticationType,
        comment: Option<impl ToString>,
    ) -> Result<RecipientInfo> {
        let request = CreateRecipientRequest {
            name: self.name.clone(),
            authentication_type: authentication_type.into(),
            comment: comment.map(|c| c.to_string()),
            ..Default::default()
        };
        self.client.create_recipient(&request).await
    }

    pub async fn get(&self) -> Result<RecipientInfo> {
        let request = GetRecipientRequest {
            name: self.name.clone(),
        };
        self.client.get_recipient(&request).await
    }

    pub async fn update(
        &self,
        new_name: Option<impl ToString>,
        comment: Option<impl ToString>,
        owner: Option<impl ToString>,
        properties: impl Into<Option<HashMap<String, String>>>,
        expiration_time: Option<i64>,
    ) -> Result<RecipientInfo> {
        let request = UpdateRecipientRequest {
            name: self.name.clone(),
            new_name: new_name.map(|s| s.to_string()),
            comment: comment.map(|s| s.to_string()),
            owner: owner.map(|s| s.to_string()),
            properties: properties.into().unwrap_or_default(),
            expiration_time,
        };
        self.client.update_recipient(&request).await
    }

    pub async fn delete(&self) -> Result<()> {
        let request = DeleteRecipientRequest {
            name: self.name.clone(),
        };
        self.client.delete_recipient(&request).await
    }
}
