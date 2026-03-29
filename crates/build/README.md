# Unity Catalog Build Crate

The `unitycatalog-build` crate is a code generation system that transforms Unity Catalog protobuf
service definitions into production-ready Rust REST API implementations. It bridges the gap between
protocol buffer specifications and idiomatic Rust web services.

## Overview

This crate processes protobuf file descriptors containing Unity Catalog service
definitions and generates complete REST API implementations including:

- **Handler Traits**: Async trait definitions for service operations
- **HTTP Clients**: Feature-complete REST clients with query parameter support
- **Server Routes**: Axum-based route handlers and extractors
- **Type Mappings**: Seamless conversions between protobuf and Rust types

## Architecture

The crate follows a multi-phase pipeline architecture:

```
Protobuf Descriptors → Parsing → Analysis → Planning → Generation → Output
```

### Phase 1: Parsing (`src/parsing/`)

Loads and parses protobuf file descriptors, extracts gnostic OpenAPI annotations and Google API
HTTP rules, and builds metadata structures from service definitions.

### Phase 2: Analysis (`src/analysis/`)

Processes collected metadata to understand service structure, classifies methods by type
(List, Create, Get, Update, Delete), extracts request parameters (path, query, body fields),
and validates completeness of REST metadata.

### Phase 3: Planning (`src/codegen/mod.rs`)

Creates generation plans for each service, determines what code artifacts need to be generated,
and organizes methods and parameters for optimal code generation.

### Phase 4: Generation (`src/codegen/generation/`)

- **Handler Generation** (`handler.rs`): Creates async trait definitions
- **Client Generation** (`client.rs`): Builds HTTP client implementations with query parameter support
- **Builder Generation** (`builder.rs`): Creates request builders with fluent API for Create/Update operations
- **Server Generation** (`server.rs`): Generates Axum route handlers and request extractors

### Phase 5: Output (`src/output.rs`)

Formats generated code using `prettyplease`, writes files to appropriate directory structure,
and creates module definitions and re-exports.

## Key Components

### Core Data Structures

- **`CodeGenMetadata`**: Container for all extracted protobuf metadata
- **`MethodMetadata`**: Information about individual service methods
- **`MessageField`**: Details about protobuf message fields
- **`GenerationPlan`**: High-level plan for code generation
- **`MethodPlan`**: Detailed plan for individual method generation
- **`CodeGenConfig`**: Configuration for import paths and output directories (see External Usage)
- **`CodeGenOutput`**: Output directory configuration

## Generated Code Features

### Request Builders

Generated builders provide a fluent API for Create and Update operations:

```rust
// Traditional verbose approach
let request = CreateCatalogRequest {
    name: "my_catalog".to_string(),
    comment: Some("A catalog for my data".to_string()),
    ..Default::default()
};
client.create_catalog(&request).await?;

// With generated builders - much cleaner!
client.create_catalog("my_catalog")
    .with_comment("A catalog for my data")
    .await?;
```

Key builder features:
- **Required parameters** are constructor arguments
- **Optional parameters** use fluent `with_*` methods
- **Generic property setters** for HashMap fields accept various iterator types
- **IntoFuture implementation** allows direct `.await` on builders
- **Type flexibility** with `impl ToString` for string parameters

### HTTP Clients

Generated clients include sophisticated features:

```rust
// Automatic query parameter handling
pub async fn list_catalogs(&self, request: &ListCatalogsRequest) -> Result<ListCatalogsResponse> {
    let mut url = self.base_url.join("catalogs")?;

    // Optional query parameters are conditionally added
    if let Some(ref value) = request.max_results {
        url.query_pairs_mut().append_pair("max_results", &value.to_string());
    }

    let response = self.client.get(url).send().await?;
    // ... error handling and response parsing
}
```

### Handler Traits

Clean async trait definitions for easy implementation:

```rust
#[async_trait]
pub trait CatalogHandler {
    async fn list_catalogs(
        &self,
        request: ListCatalogsRequest,
        context: RequestContext,
    ) -> Result<ListCatalogsResponse>;
    // ... other methods
}
```

## Usage

### Command Line Interface

```bash
cargo run -p unitycatalog-build -- \
    --output-common   crates/common/src/codegen \
    --output-server   crates/server/src/codegen \
    --output-client   crates/client/src/codegen \
    --output-python   python/client/src/codegen \
    --output-node     node/client/src/codegen \
    --output-node-ts  node/client/unitycatalog \
    --descriptors     descriptors.bin
```

Within the Unity Catalog workspace, use `just generate-code`.

### Environment Variables

- `UC_BUILD_OUTPUT_COMMON` — output directory for common (extractor) code
- `UC_BUILD_OUTPUT_SERVER` — output directory for server handler and route code
- `UC_BUILD_OUTPUT_CLIENT` — output directory for HTTP client code
- `UC_BUILD_OUTPUT_PYTHON` — output directory for Python bindings (optional)
- `UC_BUILD_OUTPUT_NODE` — output directory for Node.js NAPI bindings (optional)
- `UC_BUILD_OUTPUT_NODE_TS` — output directory for Node.js TypeScript client (optional)
- `UC_BUILD_DESCRIPTORS` — path to protobuf descriptors file
- `UC_BUILD_CONTEXT_TYPE` — override context type path (see External Usage)
- `UC_BUILD_RESULT_TYPE` — override result type path (see External Usage)
- `UC_BUILD_MODELS_PATH_TEMPLATE` — override external models path template (see External Usage)
- `UC_BUILD_MODELS_PATH_CRATE_TEMPLATE` — override crate-local models path template (see External Usage)
- `UC_BUILD_PYTHON_TYPINGS_FILENAME` — override the generated `.pyi` filename (see External Usage)

## External Usage

`CodeGenConfig` makes the generator usable in codebases that have different runtime types.

```rust
use unitycatalog_build::{CodeGenConfig, CodeGenOutput};
use unitycatalog_build::codegen::generate_code;

// Server-only example — omit `client` (and python/node) to skip those outputs entirely.
let output = CodeGenOutput {
    common:  "path/to/common".into(),
    server:  Some("path/to/server".into()),
    client:  None,
    python:  None,
    node:    None,
    node_ts: None,
    python_typings_filename: "my_client.pyi".to_string(),
};

let mut config = CodeGenConfig::unitycatalog_defaults(output);

// Override import paths for your crate
config.context_type_path           = "my_crate::Context".to_string();
config.result_type_path            = "my_crate::Result".to_string();
config.models_path_template        = "my_crate::models::{service}".to_string();
config.models_path_crate_template  = "crate::models::{service}".to_string();

generate_code(&metadata, &config)?;
```

The same overrides are available as CLI flags:

```bash
cargo run -p unitycatalog-build -- \
    --context-type             my_crate::Context \
    --result-type              my_crate::Result \
    --models-path-template     "my_crate::models::{service}" \
    --models-path-crate-template "crate::models::{service}" \
    --output-common ... --output-server ... --output-client ... --descriptors ...
```

### Known Limitations (intentionally deferred)

The following import paths are still hardcoded and are **not yet configurable**:

| Symbol | Location | Rationale |
|--------|----------|-----------|
| `cloud_client::CloudClient` | generated `client.rs` | Abstracting the HTTP transport is a large refactor |
| `crate::policy::Principal` | generated `server.rs` | UC-specific axum middleware concern |
| `crate::error::parse_error_response` | generated `client.rs` | UC error helper |

Each of these is annotated with a `// TODO: make configurable` comment in the source.

## Input Requirements

The crate expects protobuf file descriptors containing:

1. **Service Definitions**: Unity Catalog service protobuf definitions
2. **Gnostic Annotations**: OpenAPI v3 operation metadata
3. **Google API HTTP Rules**: HTTP method and path specifications
4. **Message Definitions**: Request and response type definitions

## Testing

Run tests with:

```bash
cargo test -p unitycatalog-build
```

## Generated File Structure

```
output_directory/
├── mod.rs                    # Root module with re-exports
├── service_name/
│   ├── mod.rs               # Service module definition
│   ├── handler.rs           # Handler trait definition
│   ├── client.rs            # HTTP client implementation
│   ├── builders.rs          # Request builders (for Create/Update operations)
│   └── server.rs            # Axum route handlers (if axum feature enabled)
└── ...                      # Additional services
```
