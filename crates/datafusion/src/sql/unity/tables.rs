use std::sync::Arc;

use datafusion::arrow::array::RecordBatch;
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use datafusion::common::{DataFusionError, Result};
use datafusion::sql::sqlparser::ast::{ColumnDef, DataType as SqlDataType, ObjectName};
use unitycatalog_client::UnityCatalogClient;

use crate::managed::create_managed_table;
use crate::sql::unity::create_response_to_batch;

/// `CREATE TABLE <catalog>.<schema>.<table> (<cols>) USING DELTA` — a Unity
/// Catalog **managed** Delta table (no `LOCATION`; UC allocates the storage
/// location). Lowered to an `Extension` node like the other UC DDL so it rides
/// through the `allow_ddl=false` SQL gate and is authorized by Cedar instead.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct CreateManagedTableStatement {
    pub name: ObjectName,
    pub columns: Vec<ColumnDef>,
    pub if_not_exists: bool,
}

/// Engine identifier recorded in the v0 `commitInfo` for tables created through
/// this SQL path.
const ENGINE_INFO: &str = concat!("datafusion-unitycatalog/", env!("CARGO_PKG_VERSION"));

impl CreateManagedTableStatement {
    pub(crate) async fn execute(&self, client: UnityCatalogClient) -> Result<RecordBatch> {
        let (catalog, schema, table) = split_table_name(&self.name)?;
        let delta = Arc::new(client.delta_v1());

        // Honor `IF NOT EXISTS`: a successful load means the table already
        // exists, so this is a no-op rather than a hard 409 from createTable.
        if self.if_not_exists && delta.load_table(&catalog, &schema, &table).await.is_ok() {
            return create_response_to_batch(self.name.to_string(), "Table", "exists");
        }

        let arrow_schema = sql_columns_to_arrow_schema(&self.columns)?;

        let created = create_managed_table(
            delta,
            &catalog,
            &schema,
            &table,
            arrow_schema,
            // Partitioned managed tables are out of scope for this path (the
            // append path is unpartitioned-only); always create unpartitioned.
            Vec::new(),
            ENGINE_INFO,
        )
        .await
        .map_err(|e| DataFusionError::External(Box::new(e)))?;

        create_response_to_batch(
            self.name.to_string(),
            "Table",
            serde_json::json!({
                "table_id": created.table_id,
                "location": created.location,
            }),
        )
    }
}

/// Split a table [`ObjectName`] into `(catalog, schema, table)`.
///
/// Managed tables must be fully qualified — hydrofoil has no "current catalog"
/// to resolve a shorter name against (matching [`super::schemas`]' two-part
/// requirement for schemas).
fn split_table_name(name: &ObjectName) -> Result<(String, String, String)> {
    match name.0.as_slice() {
        [catalog, schema, table] => {
            Ok((catalog.to_string(), schema.to_string(), table.to_string()))
        }
        _ => Err(DataFusionError::Execution(format!(
            "Managed table name '{name}' must be a three-part identifier (<catalog>.<schema>.<table>)"
        ))),
    }
}

/// Map parsed SQL column definitions to an Arrow [`SchemaRef`].
///
/// Only the primitive types the managed connector supports are accepted (see
/// `arrow_primitive_to_delta` in the connector); nested / decimal / unsupported
/// types are rejected here with a clear error rather than failing deeper in the
/// create path. Recognizes both standard SQL names and the Spark aliases
/// `LONG` / `STRING` (which the generic dialect surfaces as `Custom`).
pub(crate) fn sql_columns_to_arrow_schema(columns: &[ColumnDef]) -> Result<SchemaRef> {
    if columns.is_empty() {
        return Err(DataFusionError::Execution(
            "managed CREATE TABLE requires at least one column".to_string(),
        ));
    }
    let fields = columns
        .iter()
        .map(|col| {
            let data_type = sql_type_to_arrow(&col.data_type, &col.name.value)?;
            // Columns are nullable unless an explicit NOT NULL option is present;
            // matching Spark/Delta defaults keeps the create schema permissive.
            let nullable = !col.options.iter().any(|o| {
                matches!(
                    o.option,
                    datafusion::sql::sqlparser::ast::ColumnOption::NotNull
                )
            });
            Ok(Field::new(&col.name.value, data_type, nullable))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Arc::new(Schema::new(fields)))
}

fn sql_type_to_arrow(dt: &SqlDataType, col: &str) -> Result<DataType> {
    let unsupported = |dt: &SqlDataType| {
        DataFusionError::NotImplemented(format!(
            "column '{col}' has unsupported type '{dt}' for a managed Delta table \
             (supported: boolean, tinyint/byte, smallint/short, int, bigint/long, \
             real/float, double, string/varchar/char/text, binary, date, timestamp)"
        ))
    };
    let mapped = match dt {
        SqlDataType::Boolean | SqlDataType::Bool => DataType::Boolean,
        SqlDataType::TinyInt(_) => DataType::Int8,
        SqlDataType::SmallInt(_) => DataType::Int16,
        SqlDataType::Int(_) | SqlDataType::Integer(_) => DataType::Int32,
        SqlDataType::BigInt(_) => DataType::Int64,
        SqlDataType::Real | SqlDataType::Float4 | SqlDataType::Float(_) => DataType::Float32,
        SqlDataType::Double(_)
        | SqlDataType::DoublePrecision
        | SqlDataType::Float8
        | SqlDataType::Float64 => DataType::Float64,
        SqlDataType::Char(_)
        | SqlDataType::Varchar(_)
        | SqlDataType::Text
        | SqlDataType::String(_) => DataType::Utf8,
        SqlDataType::Binary(_) | SqlDataType::Bytea => DataType::Binary,
        SqlDataType::Date => DataType::Date32,
        SqlDataType::Timestamp(_, _) | SqlDataType::TimestampNtz(_) => {
            DataType::Timestamp(TimeUnit::Microsecond, None)
        }
        // Spark spells some types (`LONG`, `STRING`, `BYTE`, `SHORT`) as bare
        // identifiers the generic dialect parses as `Custom`. Match on the name.
        SqlDataType::Custom(name, _) => match name.to_string().to_ascii_lowercase().as_str() {
            "long" => DataType::Int64,
            "string" => DataType::Utf8,
            "byte" => DataType::Int8,
            "short" => DataType::Int16,
            _ => return Err(unsupported(dt)),
        },
        _ => return Err(unsupported(dt)),
    };
    Ok(mapped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::sql::sqlparser::ast::Ident;

    fn col(name: &str, dt: SqlDataType) -> ColumnDef {
        ColumnDef {
            name: Ident::new(name),
            data_type: dt,
            options: vec![],
        }
    }

    #[test]
    fn maps_supported_primitives() {
        let cols = vec![
            col("a", SqlDataType::BigInt(None)),
            col("b", SqlDataType::Varchar(None)),
            col("c", SqlDataType::Boolean),
            col(
                "d",
                SqlDataType::Custom(vec![Ident::new("LONG")].into(), vec![]),
            ),
            col(
                "e",
                SqlDataType::Custom(vec![Ident::new("string")].into(), vec![]),
            ),
        ];
        let schema = sql_columns_to_arrow_schema(&cols).unwrap();
        assert_eq!(schema.field(0).data_type(), &DataType::Int64);
        assert_eq!(schema.field(1).data_type(), &DataType::Utf8);
        assert_eq!(schema.field(2).data_type(), &DataType::Boolean);
        assert_eq!(schema.field(3).data_type(), &DataType::Int64);
        assert_eq!(schema.field(4).data_type(), &DataType::Utf8);
    }

    #[test]
    fn rejects_unsupported_type() {
        let cols = vec![col(
            "amount",
            SqlDataType::Decimal(datafusion::sql::sqlparser::ast::ExactNumberInfo::None),
        )];
        let err = sql_columns_to_arrow_schema(&cols).unwrap_err();
        assert!(err.to_string().contains("unsupported type"));
    }

    #[test]
    fn rejects_empty_columns() {
        let err = sql_columns_to_arrow_schema(&[]).unwrap_err();
        assert!(err.to_string().contains("at least one column"));
    }

    #[test]
    fn requires_three_part_name() {
        let two: ObjectName = vec![Ident::new("s"), Ident::new("t")].into();
        assert!(split_table_name(&two).is_err());
        let three: ObjectName = vec![Ident::new("c"), Ident::new("s"), Ident::new("t")].into();
        assert_eq!(
            split_table_name(&three).unwrap(),
            ("c".to_string(), "s".to_string(), "t".to_string())
        );
    }
}
