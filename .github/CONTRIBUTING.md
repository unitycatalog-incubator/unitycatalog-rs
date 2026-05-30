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
   - Re-run `just generate-code` — the type stub at
     `python/client/python/unitycatalog_client/_client.pyi` is **fully
     regenerated** from the proto + Rust client surface (the justfile
     `mv`s a freshly-built stub over the existing file), so do **not**
     edit it by hand.
   - If you added a new pyclass to `python/client/src/lib.rs`'s
     `_client` module, also add the matching
     `from ._client import Foo as Foo` line (and `"Foo"` in `__all__`)
     to `python/client/python/unitycatalog_client/__init__.py` so the
     symbol surfaces at the package root with a deliberate, typed
     re-export. The package root has **no** `__init__.pyi`: the
     explicit `import X as X` form is sufficient for static type
     checkers to resolve `unitycatalog_client.X` to the typed entry
     declared in `_client.pyi`.

### Hand-written PyO3 helpers

A small number of Python bindings are not proto-derived (e.g.
`parse_uc_url`, ergonomic methods like `temporary_*_credential` that
do name → UUID resolution before calling the generated RPC). These
live in `python/client/src/{client,reference}.rs` and have PyO3 names,
signatures, and docstrings — but Python type checkers can't read those
attributes off the compiled `.so`, and the trestle codegen doesn't see
them either.

The convention is **not** to hand-edit `_client.pyi` directly. Instead:

- **Declare the symbol in `python/client/_client_supplement.pyi`.**
  The `just generate-code` recipe appends this hand-written fragment
  to the codegen-emitted `_client.pyi` after the move, so the merged
  stub describes the full runtime surface of the `_client` PyO3
  module. The supplement lives outside the package directory so
  static type checkers don't try to validate it standalone (it is a
  fragment, not a complete stub).

- **Re-export from the package root via `__init__.py`** using the
  PEP 484 `from ._client import Foo as Foo` form, exactly like the
  codegen-derived types. The merged `_client.pyi` describes
  hand-written and codegen-emitted names uniformly, so the same
  re-export idiom serves both. Keep internal helpers (e.g.
  `parse_uc_url`) out of the package-root re-export list — consumers
  that legitimately need them can import from `unitycatalog_client._client`
  directly.

- **For proto-shaped surface** (a regular `Get/Update/Delete/Create/List`-
  flavoured RPC, or a `Custom(Post|Patch)`-flavoured RPC that the
  Python emitter knows how to render), prefer extending the proto so
  trestle generates everything end-to-end.

When you `m.add_class::<NewType>()` in `python/client/src/lib.rs`
(or hand-register a new exception / free function), remember to add
the matching `from ._client import NewType as NewType` line to
`python/client/python/unitycatalog_client/__init__.py` so the symbol
surfaces at the package root.
