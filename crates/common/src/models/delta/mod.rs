//! Hand-written serde models for the UC Delta REST API (`/delta/v1/...`).
//!
//! Unlike most of the workspace, the Delta API is a standalone REST protocol
//! (not a generated resource API), so its wire types are hand-maintained. They
//! live here in `common` so both the server router and the client share one
//! definition.

pub mod v1;
