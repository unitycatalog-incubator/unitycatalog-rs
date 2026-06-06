//! Explorer state and rendering.
//!
//! The app keeps its own data model (a map of node paths to [`NodeData`]) and
//! rebuilds the `tui-tree-widget` items from it every frame — the immediate-mode
//! pattern ratatui uses. Expanding a node lazily spawns a client call whose
//! result returns as an [`Action`].

use std::collections::BTreeMap;

use futures::TryStreamExt;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tokio::sync::mpsc::UnboundedSender;
use tui_tree_widget::{Tree, TreeItem, TreeState};
use unitycatalog_client::UnityCatalogClient;

use super::action::{Action, NodePath};
use crate::render::TableView;

/// What a node holds and how far its children have been loaded.
enum NodeData {
    Catalog {
        model: unitycatalog_common::Catalog,
        children: LoadState<Vec<String>>,
    },
    Schema {
        model: unitycatalog_common::Schema,
        children: LoadState<Vec<LeafNode>>,
    },
    Leaf(LeafNode),
}

/// A loadable child set: not yet requested, in flight, loaded, or errored.
/// The error message itself is surfaced in the status bar (see [`App::status`]).
enum LoadState<T> {
    Collapsed,
    Loading,
    Loaded(T),
    Error,
}

/// A table/volume/function leaf plus its rendered detail.
#[derive(Clone)]
struct LeafNode {
    key: String,
    label: String,
    detail: Vec<(String, String)>,
}

pub struct App {
    pub running: bool,
    client: UnityCatalogClient,
    tx: UnboundedSender<Action>,
    state: TreeState<String>,
    /// Catalog order (root level) and per-node data, keyed by path joined with '\u{1}'.
    catalogs: LoadState<Vec<String>>,
    nodes: BTreeMap<NodePath, NodeData>,
    status: String,
}

impl App {
    pub fn new(client: UnityCatalogClient, tx: UnboundedSender<Action>) -> Self {
        let app = Self {
            running: true,
            client,
            tx,
            state: TreeState::default(),
            catalogs: LoadState::Loading,
            nodes: BTreeMap::new(),
            status: "loading catalogs…".into(),
        };
        app.spawn_load_catalogs();
        app
    }

    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Tick => {}
            Action::Quit => self.running = false,
            Action::Resize => {}
            Action::Up => {
                self.ensure_selection();
                self.state.key_up();
            }
            Action::Down => {
                self.ensure_selection();
                self.state.key_down();
            }
            Action::CollapseSelected => {
                // If open, close; otherwise step to the parent.
                if !self.state.key_left() {
                    self.state.key_up();
                }
            }
            Action::ExpandSelected => {
                self.ensure_selection();
                self.expand_selected();
            }
            Action::LoadedCatalogs(catalogs) => {
                let mut order = Vec::new();
                for c in catalogs {
                    let path = vec![c.name.clone()];
                    order.push(c.name.clone());
                    self.nodes.insert(
                        path,
                        NodeData::Catalog {
                            model: c,
                            children: LoadState::Collapsed,
                        },
                    );
                }
                self.status = format!("{} catalogs", order.len());
                self.catalogs = LoadState::Loaded(order);
            }
            Action::LoadedSchemas { catalog, schemas } => {
                let mut order = Vec::new();
                for s in schemas {
                    let path = vec![catalog.clone(), s.name.clone()];
                    order.push(s.name.clone());
                    self.nodes.insert(
                        path,
                        NodeData::Schema {
                            model: s,
                            children: LoadState::Collapsed,
                        },
                    );
                }
                if let Some(NodeData::Catalog { children, .. }) =
                    self.nodes.get_mut(&vec![catalog.clone()])
                {
                    *children = LoadState::Loaded(order);
                }
                self.state.open(vec![catalog]);
            }
            Action::LoadedChildren {
                path,
                tables,
                volumes,
                functions,
            } => {
                let mut leaves = Vec::new();
                for t in &tables {
                    leaves.push(LeafNode {
                        key: format!("table:{}", t.name),
                        label: format!("▸ {}", t.name),
                        detail: pairs(t),
                    });
                }
                for v in &volumes {
                    leaves.push(LeafNode {
                        key: format!("volume:{}", v.name),
                        label: format!("◰ {}", v.name),
                        detail: pairs(v),
                    });
                }
                for f in &functions {
                    leaves.push(LeafNode {
                        key: format!("fn:{}", f.name),
                        label: format!("ƒ {}", f.name),
                        detail: pairs(f),
                    });
                }
                // Register each leaf under its full tree path so the detail
                // pane can resolve it when selected.
                for leaf in &leaves {
                    let mut leaf_path = path.clone();
                    leaf_path.push(leaf.key.clone());
                    self.nodes.insert(leaf_path, NodeData::Leaf(leaf.clone()));
                }
                if let Some(NodeData::Schema { children, .. }) = self.nodes.get_mut(&path) {
                    *children = LoadState::Loaded(leaves);
                }
                self.state.open(path);
            }
            Action::LoadError { path, message } => {
                self.set_error(&path, message);
            }
        }
    }

    /// `TreeState` starts with no selection; on the first navigation key,
    /// select the first rendered node so subsequent keys have a target.
    /// (`select_first` reads the identifiers captured by the last render.)
    fn ensure_selection(&mut self) {
        if self.state.selected().is_empty() {
            self.state.select_first();
        }
    }

    /// Expand the selected node, lazily kicking off a load if needed.
    fn expand_selected(&mut self) {
        let selected = self.state.selected().to_vec();
        match selected.len() {
            // Catalog node: load its schemas if not yet loaded.
            1 => {
                let catalog = selected[0].clone();
                let needs_load = matches!(
                    self.nodes.get(&selected),
                    Some(NodeData::Catalog {
                        children: LoadState::Collapsed | LoadState::Error,
                        ..
                    })
                );
                if needs_load {
                    if let Some(NodeData::Catalog { children, .. }) = self.nodes.get_mut(&selected)
                    {
                        *children = LoadState::Loading;
                    }
                    self.spawn_load_schemas(catalog);
                } else {
                    self.state.toggle_selected();
                }
            }
            // Schema node: load its tables/volumes/functions.
            2 => {
                let needs_load = matches!(
                    self.nodes.get(&selected),
                    Some(NodeData::Schema {
                        children: LoadState::Collapsed | LoadState::Error,
                        ..
                    })
                );
                if needs_load {
                    if let Some(NodeData::Schema { children, .. }) = self.nodes.get_mut(&selected) {
                        *children = LoadState::Loading;
                    }
                    self.spawn_load_children(selected);
                } else {
                    self.state.toggle_selected();
                }
            }
            _ => {} // leaves have nothing to expand
        }
    }

    fn set_error(&mut self, path: &NodePath, message: String) {
        match self.nodes.get_mut(path) {
            Some(NodeData::Catalog { children, .. }) => *children = LoadState::Error,
            Some(NodeData::Schema { children, .. }) => *children = LoadState::Error,
            _ => {}
        }
        if path.is_empty() {
            self.catalogs = LoadState::Error;
        }
        self.status = format!("error: {message}");
    }

    // --- async loaders -----------------------------------------------------

    fn spawn_load_catalogs(&self) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let res = client
                .list_catalogs()
                .into_stream()
                .try_collect::<Vec<_>>()
                .await;
            let _ = tx.send(match res {
                Ok(catalogs) => Action::LoadedCatalogs(catalogs),
                Err(e) => Action::LoadError {
                    path: vec![],
                    message: e.to_string(),
                },
            });
        });
    }

    fn spawn_load_schemas(&self, catalog: String) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let res = client
                .catalog(&catalog)
                .list_schemas()
                .into_stream()
                .try_collect::<Vec<_>>()
                .await;
            let _ = tx.send(match res {
                Ok(schemas) => Action::LoadedSchemas { catalog, schemas },
                Err(e) => Action::LoadError {
                    path: vec![catalog],
                    message: e.to_string(),
                },
            });
        });
    }

    fn spawn_load_children(&self, path: NodePath) {
        let client = self.client.clone();
        let tx = self.tx.clone();
        let catalog = path[0].clone();
        let schema = path[1].clone();
        tokio::spawn(async move {
            let tables = client
                .list_tables(&catalog, &schema)
                .into_stream()
                .try_collect::<Vec<_>>()
                .await;
            let volumes = client
                .list_volumes(&catalog, &schema)
                .into_stream()
                .try_collect::<Vec<_>>()
                .await;
            let functions = client
                .list_functions(&catalog, &schema)
                .into_stream()
                .try_collect::<Vec<_>>()
                .await;
            let action = match (tables, volumes, functions) {
                (Ok(tables), Ok(volumes), Ok(functions)) => Action::LoadedChildren {
                    path,
                    tables,
                    volumes,
                    functions,
                },
                (t, v, f) => {
                    let message = [t.err(), v.err(), f.err()]
                        .into_iter()
                        .flatten()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>()
                        .join("; ");
                    Action::LoadError { path, message }
                }
            };
            let _ = tx.send(action);
        });
    }

    // --- rendering ---------------------------------------------------------

    pub fn draw(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(frame.area());
        let panes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        self.draw_tree(frame, panes[0]);
        self.draw_detail(frame, panes[1]);
        self.draw_status(frame, chunks[1]);
    }

    fn draw_tree(&mut self, frame: &mut Frame, area: Rect) {
        let items = self.build_items();
        match Tree::new(&items) {
            Ok(tree) => {
                let tree = tree
                    .block(Block::default().borders(Borders::ALL).title(" Catalog "))
                    .highlight_style(
                        Style::default()
                            .bg(Color::Indexed(236))
                            .add_modifier(Modifier::BOLD),
                    );
                frame.render_stateful_widget(tree, area, &mut self.state);
                // The render just populated the widget's identifier list, so a
                // first-time selection now resolves to the first node. Without
                // this, nothing is selected until the user presses a key.
                if !items.is_empty() && self.state.selected().is_empty() {
                    self.state.select_first();
                }
            }
            Err(_) => {
                let p = Paragraph::new("failed to build tree")
                    .block(Block::default().borders(Borders::ALL).title(" Catalog "));
                frame.render_widget(p, area);
            }
        }
    }

    fn draw_detail(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title(" Detail ");
        let selected = self.state.selected();
        let lines: Vec<Line> = match self.detail_for(selected) {
            Some(pairs) => pairs
                .iter()
                .map(|(k, v)| {
                    Line::from(vec![
                        Span::styled(
                            format!("{k}: "),
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(v.clone()),
                    ])
                })
                .collect(),
            None => vec![Line::from(Span::styled(
                "select a node",
                Style::default().fg(Color::DarkGray),
            ))],
        };
        let p = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(p, area);
    }

    fn draw_status(&self, frame: &mut Frame, area: Rect) {
        let help = " ↑/↓ move  ·  →/Enter expand  ·  ←/Esc collapse  ·  q quit ";
        let line = Line::from(vec![
            Span::styled(
                format!(" {} ", self.status),
                Style::default().bg(Color::Indexed(238)),
            ),
            Span::styled(help, Style::default().fg(Color::DarkGray)),
        ]);
        frame.render_widget(Paragraph::new(line), area);
    }

    /// Resolve the detail rows for a given selected path.
    fn detail_for(&self, path: &[String]) -> Option<Vec<(String, String)>> {
        match self.nodes.get(path) {
            Some(NodeData::Catalog { model, .. }) => Some(pairs(model)),
            Some(NodeData::Schema { model, .. }) => Some(pairs(model)),
            Some(NodeData::Leaf(leaf)) => Some(leaf.detail.clone()),
            None => None,
        }
    }

    /// Rebuild the tree items from the current data model.
    fn build_items(&self) -> Vec<TreeItem<'static, String>> {
        let LoadState::Loaded(catalog_order) = &self.catalogs else {
            return Vec::new();
        };
        let mut items = Vec::new();
        for catalog in catalog_order {
            let path = vec![catalog.clone()];
            let Some(NodeData::Catalog { children, .. }) = self.nodes.get(&path) else {
                continue;
            };
            let schema_items = self.build_schema_items(catalog, children);
            let item = TreeItem::new(catalog.clone(), node_label(catalog, children), schema_items)
                .unwrap_or_else(|_| TreeItem::new_leaf(catalog.clone(), catalog.clone()));
            items.push(item);
        }
        items
    }

    fn build_schema_items(
        &self,
        catalog: &str,
        children: &LoadState<Vec<String>>,
    ) -> Vec<TreeItem<'static, String>> {
        let LoadState::Loaded(schema_order) = children else {
            return Vec::new();
        };
        let mut items = Vec::new();
        for schema in schema_order {
            let path = vec![catalog.to_string(), schema.clone()];
            let Some(NodeData::Schema {
                children: leaves, ..
            }) = self.nodes.get(&path)
            else {
                continue;
            };
            let leaf_items: Vec<TreeItem<'static, String>> = match leaves {
                LoadState::Loaded(leaves) => leaves
                    .iter()
                    .map(|l| TreeItem::new_leaf(l.key.clone(), l.label.clone()))
                    .collect(),
                _ => Vec::new(),
            };
            let item = TreeItem::new(schema.clone(), schema_label(schema, leaves), leaf_items)
                .unwrap_or_else(|_| TreeItem::new_leaf(schema.clone(), schema.clone()));
            items.push(item);
        }
        items
    }
}

/// Build a `(key, value)` detail listing from a model's `TableView` row.
fn pairs<T: TableView>(model: &T) -> Vec<(String, String)> {
    T::headers()
        .into_iter()
        .map(String::from)
        .zip(model.row())
        .collect()
}

fn node_label(name: &str, children: &LoadState<Vec<String>>) -> String {
    format!("🗂  {name}{}", load_suffix(children))
}

fn schema_label(name: &str, children: &LoadState<Vec<LeafNode>>) -> String {
    format!("📁 {name}{}", load_suffix(children))
}

fn load_suffix<T>(state: &LoadState<T>) -> &'static str {
    match state {
        LoadState::Loading => "  (loading…)",
        LoadState::Error => "  (error)",
        _ => "",
    }
}
