use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::shares::v1::*;

use super::utils::stream_paginated;
use crate::Result;
pub(super) use crate::codegen::shares::ShareClient as ShareClientBase;

impl ShareClientBase {
    pub fn list(&self, max_results: impl Into<Option<i32>>) -> BoxStream<'_, Result<ShareInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = ListSharesRequest {
                max_results,
                page_token,
            };
            let res = self.list_shares(&request).await?;
            Ok((res.shares, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}

#[derive(Clone)]
pub struct ShareClient {
    name: String,
    client: ShareClientBase,
}

impl ShareClient {
    pub fn new(name: impl ToString, client: ShareClientBase) -> Self {
        Self {
            name: name.to_string(),
            client,
        }
    }

    pub(super) async fn create(&self, comment: Option<impl ToString>) -> Result<ShareInfo> {
        let request = CreateShareRequest {
            name: self.name.clone(),
            comment: comment.map(|c| c.to_string()),
        };
        self.client.create_share(&request).await
    }

    pub async fn get(&self, include_shared_data: impl Into<Option<bool>>) -> Result<ShareInfo> {
        let request = GetShareRequest {
            name: self.name.clone(),
            include_shared_data: include_shared_data.into(),
        };
        self.client.get_share(&request).await
    }

    pub async fn delete(&self) -> Result<()> {
        let request = DeleteShareRequest {
            name: self.name.clone(),
        };
        self.client.delete_share(&request).await
    }

    pub async fn update(
        &self,
        new_name: Option<impl ToString>,
        updates: Vec<DataObjectUpdate>,
        comment: Option<impl ToString>,
        owner: Option<impl ToString>,
    ) -> Result<ShareInfo> {
        let request = UpdateShareRequest {
            name: self.name.clone(),
            new_name: new_name
                .map(|s| s.to_string())
                .and_then(|s| (!s.is_empty()).then_some(s)),
            comment: comment.map(|c| c.to_string()),
            owner: owner.map(|o| o.to_string()),
            updates,
        };
        self.client.update_share(&request).await
    }
}
