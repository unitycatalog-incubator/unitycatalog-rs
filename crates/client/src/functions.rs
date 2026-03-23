use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use unitycatalog_common::models::functions::v1::*;

use crate::Result;
use crate::codegen::functions::builders::{
    CreateFunctionBuilder, DeleteFunctionBuilder, GetFunctionBuilder, UpdateFunctionBuilder,
};
pub(super) use crate::codegen::functions::client::FunctionClient as FunctionClientBase;

impl FunctionClientBase {
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

/// Ergonomic client for a specific function (catalog.schema.function).
#[derive(Clone)]
pub struct FunctionClient {
    catalog_name: String,
    schema_name: String,
    name: String,
    client: FunctionClientBase,
}

impl FunctionClient {
    pub fn new(
        catalog_name: impl ToString,
        schema_name: impl ToString,
        name: impl ToString,
        client: FunctionClientBase,
    ) -> Self {
        Self {
            catalog_name: catalog_name.to_string(),
            schema_name: schema_name.to_string(),
            name: name.to_string(),
            client,
        }
    }

    /// Construct a `FunctionClient` from a three-level fully-qualified name
    /// (`catalog_name.schema_name.function_name`).
    pub fn new_from_full_name(full_name: impl ToString, client: FunctionClientBase) -> Self {
        let full_name = full_name.to_string();
        let parts: Vec<&str> = full_name.split('.').collect();
        if parts.len() != 3 {
            panic!("Invalid function full name format. Expected: catalog.schema.function");
        }
        Self {
            catalog_name: parts[0].to_string(),
            schema_name: parts[1].to_string(),
            name: parts[2].to_string(),
            client,
        }
    }

    /// Return the fully-qualified name in `catalog.schema.function` form.
    pub fn full_name(&self) -> String {
        format!("{}.{}.{}", self.catalog_name, self.schema_name, self.name)
    }

    /// Create a function using the builder pattern.
    pub fn create(
        &self,
        data_type: impl Into<String>,
        full_data_type: impl Into<String>,
        parameter_style: ParameterStyle,
        is_deterministic: bool,
        sql_data_access: SqlDataAccess,
        is_null_call: bool,
        security_type: SecurityType,
        routine_body: RoutineBody,
    ) -> CreateFunctionBuilder {
        CreateFunctionBuilder::new(
            self.client.clone(),
            &self.name,
            &self.catalog_name,
            &self.schema_name,
            data_type,
            full_data_type,
            parameter_style,
            is_deterministic,
            sql_data_access,
            is_null_call,
            security_type,
            routine_body,
        )
    }

    /// Get this function using the builder pattern.
    pub fn get(&self) -> GetFunctionBuilder {
        GetFunctionBuilder::new(self.client.clone(), self.full_name())
    }

    /// Update this function using the builder pattern.
    pub fn update(&self) -> UpdateFunctionBuilder {
        UpdateFunctionBuilder::new(self.client.clone(), self.full_name())
    }

    /// Delete this function using the builder pattern.
    pub fn delete(&self) -> DeleteFunctionBuilder {
        DeleteFunctionBuilder::new(self.client.clone(), self.full_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_client_construction() {
        let client = FunctionClientBase::new(
            cloud_client::CloudClient::new_unauthenticated(),
            url::Url::parse("http://localhost:8080/").unwrap(),
        );

        let function = FunctionClient::new("test_catalog", "test_schema", "test_fn", client);

        assert_eq!(function.catalog_name, "test_catalog");
        assert_eq!(function.schema_name, "test_schema");
        assert_eq!(function.name, "test_fn");
        assert_eq!(function.full_name(), "test_catalog.test_schema.test_fn");
    }

    #[test]
    fn test_function_client_from_full_name() {
        let client = FunctionClientBase::new(
            cloud_client::CloudClient::new_unauthenticated(),
            url::Url::parse("http://localhost:8080/").unwrap(),
        );

        let function = FunctionClient::new_from_full_name("catalog.schema.fn_name", client);

        assert_eq!(function.catalog_name, "catalog");
        assert_eq!(function.schema_name, "schema");
        assert_eq!(function.name, "fn_name");
        assert_eq!(function.full_name(), "catalog.schema.fn_name");
    }

    #[test]
    #[should_panic(expected = "Invalid function full name format")]
    fn test_function_client_from_invalid_full_name() {
        let client = FunctionClientBase::new(
            cloud_client::CloudClient::new_unauthenticated(),
            url::Url::parse("http://localhost:8080/").unwrap(),
        );

        FunctionClient::new_from_full_name("invalid.name", client);
    }
}
