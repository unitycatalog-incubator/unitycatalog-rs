# Contribution Guide

## Getting Started

### Prerequisites

- Rust toolchain ([install instructions](https://www.rust-lang.org/tools/install))
- buf ([install instructions](https://buf.build/docs/installation))
- just ([install instructions](https://just.systems/man/en/))

## Generated Code

We heavily rely on code generation to ensure consistency with the API spec and to reduce the maintenance burden.
The most important components involved in our code generation are:

- The `protobuf` definitions which define the API surface.
- [`buf.gen.yaml`](buf.gen.yaml) which defines the code we generate using `buf`
- the [`build`](crates/build) crate which holds custom generation logic
- the [`derive`](crates/derive) crate which holds custom derive macros

The Unity Catalog API is specified as a REST API, but we maintain API definitions in
protobuf for more flexible code generation and better maintainability. To map protobuf
messages/services to REST endpoints, we annotate definitions with
[`google.api.http`](https://github.com/googleapis/googleapis/blob/master/google/api/http.proto)
and [`gnostic`](https://github.com/google/gnostic) options.

These annotations are used by the `buf` compiler to generate OpenAPI specifications
and by our custom code to provide boilerplate server/client implementations.

Run the complete generation sequence:

```sh
just generate
```

### Adding new resources

To add a new resource/API surface, follow these steps:

1. **Define protobuf schema**: Create the resource in `proto/unitycatalog/<resource>/v1/`
   - Define messages (e.g., `Volume`, `CreateVolumeRequest`)
   - Define service with RPC methods
   - Annotate with `google.api.http` and `gnostic.openapi.v3.operation`

2. **Generate base code**: Run `just generate-proto` to generate common models

3. **Update exports**: Add new types to `unitycatalog_common::models` module exports

4. **Generate clients**: Run `just generate-code` for server/client boilerplate

5. **Implement high-level client**:
   - Create `crates/client/src/<resource>.rs` with ergonomic methods
   - Add to `lib.rs` exports and main client struct
   - Add streaming support for list operations

6. **Add Python bindings**:
   - Import new types in `python/client/src/lib.rs`
   - Add Python client wrapper in `python/client/src/client.rs`
   - Update type definitions in `python/client/unitycatalog_client.pyi`
