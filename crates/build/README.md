# Unity Catalog Build Crate

The `unitycatalog-build` crate is a sophisticated code generation system that transforms Unity Catalog protobuf
service definitions into production-ready Rust REST API implementations. It bridges the gap between protocol
buffer specifications and idiomatic Rust web services.

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
- Loads and parses protobuf file descriptors
- Extracts gnostic OpenAPI annotations and Google API HTTP rules
- Builds metadata structures from service definitions

### Phase 2: Analysis (`src/codegen/analysis.rs`)
- Processes collected metadata to understand service structure
- Classifies methods by type (List, Create, Get, Update, Delete)
- Extracts request parameters (path, query, body fields)
- Validates completeness of REST metadata

### Phase 3: Planning (`src/codegen/mod.rs`)
- Creates generation plans for each service
- Determines what code artifacts need to be generated
- Organizes methods and parameters for optimal code generation

### Phase 4: Generation (`src/codegen/generation/`)
- **Handler Generation** (`handler.rs`): Creates async trait definitions
- **Client Generation** (`client.rs`): Builds HTTP client implementations with query parameter support
- **Builder Generation** (`builder.rs`): Creates request builders with fluent API for Create/Update operations
- **Server Generation** (`server.rs`): Generates Axum route handlers and request extractors

### Phase 5: Output (`src/codegen/output.rs`)
- Formats generated code using `prettyplease`
- Writes files to appropriate directory structure
- Creates module definitions and re-exports

## Key Components

### Core Data Structures

- **`CodeGenMetadata`**: Container for all extracted protobuf metadata
- **`MethodMetadata`**: Information about individual service methods
- **`MessageField`**: Details about protobuf message fields
- **`GenerationPlan`**: High-level plan for code generation
- **`MethodPlan`**: Detailed plan for individual method generation

### Utilities (`src/utils.rs`)

The utils module provides specialized functionality organized into sub-modules:

- **`strings`**: Name conversion utilities (CamelCase ↔ snake_case)
- **`paths`**: URL template processing and path parameter extraction
- **`types`**: Protobuf to Rust type mappings and optional type handling
- **`requests`**: Request classification and body field determination
- **`validation`**: Generation plan validation and error checking

### Templates (`src/codegen/templates.rs`)

Provides code formatting and template utilities for consistent Rust code generation.

## Generated Code Features

### Request Builders

Generated builders provide a fluent API for Create and Update operations, allowing for cleaner and more ergonomic code:

```rust
// Traditional verbose approach
let request = CreateCatalogRequest {
    name: "my_catalog".to_string(),
    comment: Some("A catalog for my data".to_string()),
    properties: HashMap::from([
        ("owner".to_string(), "data_team".to_string()),
        ("env".to_string(), "prod".to_string()),
    ]),
    storage_root: Some("s3://my-bucket/catalogs/".to_string()),
    ..Default::default()
};
client.create_catalog(&request).await?;

// With generated builders - much cleaner!
client.create_catalog("my_catalog")
    .with_comment("A catalog for my data")
    .with_properties([("owner", "data_team"), ("env", "prod")])
    .with_storage_root("s3://my-bucket/catalogs/")
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
    let mut url = self.base_url.join("/catalogs")?;

    // Optional query parameters are conditionally added
    if let Some(ref value) = request.max_results {
        url.query_pairs_mut().append_pair("max_results", &value.to_string());
    }
    if let Some(ref value) = request.page_token {
        url.query_pairs_mut().append_pair("page_token", &value.to_string());
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
    async fn list_catalogs(&self, request: ListCatalogsRequest) -> Result<ListCatalogsResponse>;
    async fn create_catalog(&self, request: CreateCatalogRequest) -> Result<Catalog>;
    // ... other methods
}
```

### Axum Integration

Fully compatible with the Axum web framework:

```rust
// Generated route handlers with proper parameter extraction
pub async fn list_catalogs_handler<H: CatalogHandler>(
    State(handler): State<H>,
    query: Query<ListCatalogsRequest>,
) -> Result<Json<ListCatalogsResponse>, ErrorResponse> {
    // ... implementation
}
```

## Usage

### Command Line Interface

```bash
cargo run -p unitycatalog-build -- \
    --output crates/common/src/codegen \
    --descriptors crates/common/descriptors/descriptors.bin
```

### Environment Variables

- `UC_BUILD_OUTPUT`: Output directory for generated code
- `UC_BUILD_DESCRIPTORS`: Path to protobuf descriptors file

## Input Requirements

The crate expects protobuf file descriptors containing:

1. **Service Definitions**: Unity Catalog service protobuf definitions
2. **Gnostic Annotations**: OpenAPI v3 operation metadata
3. **Google API HTTP Rules**: HTTP method and path specifications
4. **Message Definitions**: Request and response type definitions

## Testing

The crate includes comprehensive unit tests covering:

- Individual utility functions
- Code generation components
- Integration scenarios
- Error handling paths

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

## Contributing

When adding new functionality:

1. **Add utilities** to the appropriate `utils::` submodule
2. **Add generation logic** to the relevant generation module
3. **Include comprehensive tests** for new functionality
4. **Update documentation** to reflect changes
