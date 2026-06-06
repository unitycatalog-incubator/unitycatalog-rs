use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::functions::v1::*;

use crate::Result;
pub use crate::codegen::functions::FunctionClient;
pub(super) use crate::codegen::functions::client::FunctionServiceClient;

impl FunctionServiceClient {
    /// Return a paginated stream of functions within a catalog+schema.
    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<Function>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let schema_name = schema_name.into();
        super::utils::stream_paginated(
            (catalog_name, schema_name, max_results),
            move |(catalog_name, schema_name, max_results), page_token| async move {
                let request = ListFunctionsRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name: schema_name.clone(),
                    max_results,
                    page_token,
                    ..Default::default()
                };
                let res = self.list_functions(&request).await?;
                Ok((
                    res.functions,
                    (catalog_name, schema_name, max_results),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }
}
