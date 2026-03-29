//! Node.js code generation module
//!
//! Split into two submodules:
//! - `bindings`: NAPI-RS binding generation (Rust → Node.js wrapper structs)
//! - `typescript`: TypeScript client generation for idiomatic Node.js API

mod bindings;
pub(crate) mod typescript;

pub(crate) use bindings::{generate, main_module};
