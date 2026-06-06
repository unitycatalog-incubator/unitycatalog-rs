//! The message type that drives the explorer.
//!
//! Raw terminal events and the results of async client calls are both
//! normalized into [`Action`]s, which the [`App`](super::app::App) applies in
//! one place. Long-running client calls run on spawned tasks and report back by
//! sending an `Action` on the channel, so the render loop never blocks on I/O.

use unitycatalog_common::{Catalog, Function, Schema, Table, Volume};

/// A node's path in the tree, e.g. `["cat", "schema"]`. Doubles as the
/// `tui-tree-widget` identifier.
pub type NodePath = Vec<String>;

pub enum Action {
    /// Periodic tick; also the cue to redraw.
    Tick,
    /// The terminal was resized.
    Resize,
    /// Expand (and lazily load) the currently selected node.
    ExpandSelected,
    /// Collapse the currently selected node / move up a level.
    CollapseSelected,
    /// Move the selection.
    Up,
    Down,
    /// Quit the app.
    Quit,

    /// Root catalogs finished loading.
    LoadedCatalogs(Vec<Catalog>),
    /// A catalog's schemas finished loading.
    LoadedSchemas {
        catalog: String,
        schemas: Vec<Schema>,
    },
    /// A schema's leaf children (tables/volumes/functions) finished loading.
    LoadedChildren {
        path: NodePath,
        tables: Vec<Table>,
        volumes: Vec<Volume>,
        functions: Vec<Function>,
    },
    /// A load failed for the node at `path`.
    LoadError {
        path: NodePath,
        message: String,
    },
}
