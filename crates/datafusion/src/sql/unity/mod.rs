use std::{
    fmt,
    sync::{Arc, LazyLock},
};

use datafusion::arrow::{
    array::{RecordBatch, StringArray},
    datatypes::{DataType, Field, Schema},
};
use datafusion::{
    common::{DFSchema, DFSchemaRef, internal_err},
    error::Result,
    logical_expr::{LogicalPlan, UserDefinedLogicalNodeCore},
    prelude::Expr,
};
use serde::Serialize;
use unitycatalog_client::UnityCatalogClient;

pub use self::catalogs::*;
pub use self::exec::*;
pub use self::functions::*;
pub use self::schemas::*;
pub use self::tables::*;

mod catalogs;
mod exec;
mod functions;
mod schemas;
mod tables;

/// A Unity Catalog DDL statement that can be executed against a live Unity
/// Catalog instance, returning a single result [`RecordBatch`].
#[async_trait::async_trait]
pub trait ExecutableUnityCatalogStatement {
    /// The Arrow schema of the [`RecordBatch`] returned by [`Self::execute`].
    fn return_schema(&self) -> &DFSchemaRef;

    /// Execute the statement against `client` and return its result batch.
    async fn execute(&self, client: UnityCatalogClient) -> Result<RecordBatch>;
}

pub(crate) static CREATE_UC_RETURN_SCHEMA: LazyLock<DFSchemaRef> = LazyLock::new(|| {
    let arrow_schema = Schema::new(vec![
        Field::new("securable_name", DataType::Utf8, false),
        Field::new("securable_type", DataType::Utf8, false),
        Field::new("securable_object", DataType::Utf8, false),
    ]);
    DFSchemaRef::new(DFSchema::try_from(arrow_schema).unwrap())
});

pub(crate) static DROP_UC_RETURN_SCHEMA: LazyLock<DFSchemaRef> = LazyLock::new(|| {
    let arrow_schema = Schema::new(vec![
        Field::new("securable_name", DataType::Utf8, false),
        Field::new("securable_type", DataType::Utf8, false),
        Field::new("status", DataType::Utf8, false),
    ]);
    DFSchemaRef::new(DFSchema::try_from(arrow_schema).unwrap())
});

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum UnityCatalogStatement {
    CreateCatalog(CreateCatalogStatement),
    DropCatalog(DropCatalogStatement),
    CreateSchema(CreateSchemaStatement),
    DropSchema(DropSchemaStatement),
    CreateManagedTable(CreateManagedTableStatement),
    CreateFunction(CreateFunctionStatement),
    DropFunction(DropFunctionStatement),
}

impl From<CreateCatalogStatement> for UnityCatalogStatement {
    fn from(value: CreateCatalogStatement) -> Self {
        UnityCatalogStatement::CreateCatalog(value)
    }
}

impl From<DropCatalogStatement> for UnityCatalogStatement {
    fn from(value: DropCatalogStatement) -> Self {
        UnityCatalogStatement::DropCatalog(value)
    }
}

impl From<CreateSchemaStatement> for UnityCatalogStatement {
    fn from(value: CreateSchemaStatement) -> Self {
        UnityCatalogStatement::CreateSchema(value)
    }
}

impl From<DropSchemaStatement> for UnityCatalogStatement {
    fn from(value: DropSchemaStatement) -> Self {
        UnityCatalogStatement::DropSchema(value)
    }
}

impl From<CreateManagedTableStatement> for UnityCatalogStatement {
    fn from(value: CreateManagedTableStatement) -> Self {
        UnityCatalogStatement::CreateManagedTable(value)
    }
}

impl From<CreateFunctionStatement> for UnityCatalogStatement {
    fn from(value: CreateFunctionStatement) -> Self {
        UnityCatalogStatement::CreateFunction(value)
    }
}

impl From<DropFunctionStatement> for UnityCatalogStatement {
    fn from(value: DropFunctionStatement) -> Self {
        UnityCatalogStatement::DropFunction(value)
    }
}

impl UnityCatalogStatement {
    pub fn command_name(&self) -> &str {
        use UnityCatalogStatement::*;

        match &self {
            CreateCatalog(_) => "CreateCatalog",
            DropCatalog(_) => "DropCatalog",
            CreateSchema(_) => "CreateSchema",
            DropSchema(_) => "DropSchema",
            CreateManagedTable(_) => "CreateManagedTable",
            CreateFunction(_) => "CreateFunction",
            DropFunction(_) => "DropFunction",
        }
    }

    pub fn fmt_for_explain_params(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use UnityCatalogStatement::*;

        match self {
            CreateCatalog(cmd) => write!(f, "CreateCatalog: name={}", cmd.name),
            DropCatalog(cmd) => write!(
                f,
                "DropCatalog: name={} if_exists={} cascade={}",
                cmd.name, cmd.if_exists, cmd.cascade
            ),
            CreateSchema(cmd) => write!(f, "CreateSchema: name={}", cmd.name),
            DropSchema(cmd) => write!(
                f,
                "DropSchema: name={} if_exists={} cascade={}",
                cmd.name, cmd.if_exists, cmd.cascade
            ),
            CreateManagedTable(cmd) => write!(
                f,
                "CreateManagedTable: name={} columns={} if_not_exists={}",
                cmd.name,
                cmd.columns.len(),
                cmd.if_not_exists
            ),
            CreateFunction(cmd) => write!(f, "CreateFunction: name={}", cmd.name),
            DropFunction(cmd) => write!(
                f,
                "DropFunction: name={} if_exists={}",
                cmd.name, cmd.if_exists
            ),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Hash, Debug, Clone)]
pub struct ExecuteUnityCatalogPlanNode {
    pub statement: UnityCatalogStatement,
}

impl UserDefinedLogicalNodeCore for ExecuteUnityCatalogPlanNode {
    fn name(&self) -> &str {
        self.statement.command_name()
    }

    fn inputs(&self) -> Vec<&LogicalPlan> {
        vec![]
    }

    fn schema(&self) -> &DFSchemaRef {
        self.statement.return_schema()
    }

    fn expressions(&self) -> Vec<Expr> {
        vec![]
    }

    fn fmt_for_explain(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.statement.fmt_for_explain_params(f)
    }

    fn with_exprs_and_inputs(&self, exprs: Vec<Expr>, inputs: Vec<LogicalPlan>) -> Result<Self> {
        if !exprs.is_empty() || !inputs.is_empty() {
            internal_err!("CreateCatalogPlanNode does not support exprs and inputs")
        } else {
            Ok(self.clone())
        }
    }
}

#[async_trait::async_trait]
impl ExecutableUnityCatalogStatement for UnityCatalogStatement {
    fn return_schema(&self) -> &DFSchemaRef {
        use UnityCatalogStatement::*;

        match &self {
            CreateCatalog(_) | CreateSchema(_) | CreateManagedTable(_) | CreateFunction(_) => {
                &CREATE_UC_RETURN_SCHEMA
            }
            DropCatalog(_) | DropSchema(_) | DropFunction(_) => &DROP_UC_RETURN_SCHEMA,
        }
    }

    async fn execute(&self, client: UnityCatalogClient) -> Result<RecordBatch> {
        use UnityCatalogStatement::*;

        match &self {
            CreateCatalog(cmd) => cmd.execute(client).await,
            DropCatalog(cmd) => cmd.execute(client).await,
            CreateSchema(cmd) => cmd.execute(client).await,
            DropSchema(cmd) => cmd.execute(client).await,
            CreateManagedTable(cmd) => cmd.execute(client).await,
            CreateFunction(cmd) => cmd.execute(client).await,
            DropFunction(cmd) => cmd.execute(client).await,
        }
    }
}

pub(crate) fn create_response_to_batch(
    name: impl ToString,
    type_name: impl ToString,
    object: impl Serialize,
) -> Result<RecordBatch> {
    let names = vec![name.to_string()];
    let types = vec![type_name.to_string()];
    let values = vec![serde_json::to_string(&object).unwrap()];
    let schema = Arc::new(CREATE_UC_RETURN_SCHEMA.as_arrow().clone());
    Ok(RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(names)),
            Arc::new(StringArray::from(types)),
            Arc::new(StringArray::from(values)),
        ],
    )?)
}

pub(crate) fn drop_response_to_batch(
    name: impl ToString,
    type_name: impl ToString,
    status: impl ToString,
) -> Result<RecordBatch> {
    let names = vec![name.to_string()];
    let types = vec![type_name.to_string()];
    let status = vec![status.to_string()];
    let schema = Arc::new(DROP_UC_RETURN_SCHEMA.as_arrow().clone());
    Ok(RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(names)),
            Arc::new(StringArray::from(types)),
            Arc::new(StringArray::from(status)),
        ],
    )?)
}

#[cfg(test)]
mod tests {
    use datafusion::sql::sqlparser::ast::Ident;

    use super::*;
    use crate::sql::{
        CreateCatalogStatement, CreateFunctionStatement, DropCatalogStatement,
        DropFunctionStatement, DropSchemaStatement, FunctionLanguage, SqlDataAccessKind,
    };
    use datafusion::sql::sqlparser::ast::DataType as SqlDataType;

    /// A minimal scalar SQL UDF statement for the contract tests.
    fn sample_function() -> CreateFunctionStatement {
        CreateFunctionStatement {
            name: name(&["c", "s", "f"]),
            or_replace: false,
            if_not_exists: false,
            params: vec![],
            returns: SqlDataType::Int(None),
            language: FunctionLanguage::Sql,
            deterministic: true,
            data_access: SqlDataAccessKind::ContainsSql,
            routine_definition: "1".to_string(),
            comment: None,
        }
    }

    fn name(parts: &[&str]) -> datafusion::sql::sqlparser::ast::ObjectName {
        parts
            .iter()
            .map(|p| Ident::new(*p))
            .collect::<Vec<_>>()
            .into()
    }

    #[test]
    fn create_statements_use_create_return_schema() {
        let create_catalog: UnityCatalogStatement = CreateCatalogStatement {
            name: name(&["c"]),
            if_not_exists: false,
            using_share: None,
            managed_location: None,
            default_collation: None,
            comment: None,
            options: None,
        }
        .into();
        let create_schema: UnityCatalogStatement = CreateSchemaStatement {
            name: name(&["c", "s"]),
            if_not_exists: false,
            managed_location: None,
            comment: None,
            properties: None,
        }
        .into();
        let create_function: UnityCatalogStatement = sample_function().into();
        for stmt in [create_catalog, create_schema, create_function] {
            assert_eq!(stmt.return_schema(), &*CREATE_UC_RETURN_SCHEMA);
            // The logical node mirrors the statement's return schema.
            let node = ExecuteUnityCatalogPlanNode { statement: stmt };
            assert_eq!(node.schema(), &*CREATE_UC_RETURN_SCHEMA);
        }
    }

    #[test]
    fn drop_statements_use_drop_return_schema() {
        let drop_catalog: UnityCatalogStatement = DropCatalogStatement {
            name: name(&["c"]),
            if_exists: false,
            cascade: false,
        }
        .into();
        let drop_schema: UnityCatalogStatement = DropSchemaStatement {
            name: name(&["c", "s"]),
            if_exists: false,
            cascade: false,
        }
        .into();
        let drop_function: UnityCatalogStatement = DropFunctionStatement {
            name: name(&["c", "s", "f"]),
            if_exists: false,
        }
        .into();
        for stmt in [drop_catalog, drop_schema, drop_function] {
            assert_eq!(stmt.return_schema(), &*DROP_UC_RETURN_SCHEMA);
        }
    }

    #[test]
    fn command_names_are_stable() {
        // These names are the contract the Cedar visitor matches on.
        let cases: [(UnityCatalogStatement, &str); 4] = [
            (
                CreateCatalogStatement {
                    name: name(&["c"]),
                    if_not_exists: false,
                    using_share: None,
                    managed_location: None,
                    default_collation: None,
                    comment: None,
                    options: None,
                }
                .into(),
                "CreateCatalog",
            ),
            (
                DropSchemaStatement {
                    name: name(&["c", "s"]),
                    if_exists: false,
                    cascade: false,
                }
                .into(),
                "DropSchema",
            ),
            (sample_function().into(), "CreateFunction"),
            (
                DropFunctionStatement {
                    name: name(&["c", "s", "f"]),
                    if_exists: false,
                }
                .into(),
                "DropFunction",
            ),
        ];
        for (stmt, expected) in cases {
            assert_eq!(stmt.command_name(), expected);
        }
    }

    #[test]
    fn explain_display_carries_securable_name() {
        // Cedar reads the securable from the `name=<...>` token in the node's
        // `Display`/`fmt_for_explain` output — this contract must hold across the
        // cross-repo boundary.
        let cases: [(UnityCatalogStatement, &str, &str); 6] = [
            (
                CreateCatalogStatement {
                    name: name(&["my_catalog"]),
                    if_not_exists: false,
                    using_share: None,
                    managed_location: None,
                    default_collation: None,
                    comment: None,
                    options: None,
                }
                .into(),
                "CreateCatalog",
                "my_catalog",
            ),
            (
                DropCatalogStatement {
                    name: name(&["my_catalog"]),
                    if_exists: false,
                    cascade: false,
                }
                .into(),
                "DropCatalog",
                "my_catalog",
            ),
            (
                CreateSchemaStatement {
                    name: name(&["c", "s"]),
                    if_not_exists: false,
                    managed_location: None,
                    comment: None,
                    properties: None,
                }
                .into(),
                "CreateSchema",
                "c.s",
            ),
            (
                DropSchemaStatement {
                    name: name(&["c", "s"]),
                    if_exists: false,
                    cascade: false,
                }
                .into(),
                "DropSchema",
                "c.s",
            ),
            (sample_function().into(), "CreateFunction", "c.s.f"),
            (
                DropFunctionStatement {
                    name: name(&["c", "s", "f"]),
                    if_exists: false,
                }
                .into(),
                "DropFunction",
                "c.s.f",
            ),
        ];
        for (stmt, expected_name, expected_securable) in cases {
            let node = ExecuteUnityCatalogPlanNode { statement: stmt };
            assert_eq!(node.name(), expected_name);
            let rendered = format!("{}", DisplayNode(&node));
            let securable = rendered
                .split("name=")
                .nth(1)
                .and_then(|rest| rest.split_whitespace().next())
                .unwrap_or("");
            assert_eq!(securable, expected_securable, "rendered: {rendered}");
        }
    }

    /// Mirror the Cedar visitor's `DisplayNode` wrapper so the contract test
    /// exercises the same `fmt_for_explain` path Cedar relies on.
    struct DisplayNode<'a>(&'a ExecuteUnityCatalogPlanNode);

    impl fmt::Display for DisplayNode<'_> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.0.fmt_for_explain(f)
        }
    }
}
