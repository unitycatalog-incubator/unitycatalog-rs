//! Unity Catalog DDL statements lowered for DataFusion execution.
//!
//! This module owns the Unity Catalog DDL surface that runs *inside* a
//! DataFusion plan: the statement types (`CREATE`/`DROP CATALOG`, `CREATE`/`DROP
//! SCHEMA`, managed `CREATE TABLE`), their execution against a live
//! [`UnityCatalogClient`](unitycatalog_client::UnityCatalogClient), the
//! [`ExecuteUnityCatalogPlanNode`] DataFusion `Extension` node, and the
//! [`UnityCatalogPlanner`] that lowers it to a physical plan.
//!
//! The *SQL front end* that recognizes this DDL (a custom `sqlparser`-based
//! parser) lives in the host application, not here: it is coupled to host-only
//! concerns such as Delta-table maintenance commands (`VACUUM`/`OPTIMIZE`). The
//! host parses SQL into a [`UnityCatalogStatement`] and wraps it in an
//! [`ExecuteUnityCatalogPlanNode`]; everything from there is owned here.
//!
//! Authorization for the DDL is the host's Cedar policy layer's responsibility.
//! That layer matches the extension node purely by its `name()` string
//! (`CreateCatalog`/`DropCatalog`/`CreateSchema`/`DropSchema`/`CreateManagedTable`)
//! and reads the securable from the `name=<...>` token in `fmt_for_explain` — a
//! stable string contract this crate must preserve.

mod unity;

pub use unity::*;
