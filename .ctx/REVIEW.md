# Codebase Review: unitycatalog-rs

> **Status:** Early development — not production-ready.
> This document records findings from a multi-dimensional review across all crates,
> covering code quality, documentation, test coverage, performance, security, and
> architectural design. It is intended to guide prioritization toward production readiness.

---

## Production Readiness Summary

| Dimension | Rating | Notes |
|---|---|---|
| **Architecture** | ✅ Strong | Trait-based, composable, extensible codegen pipeline |
| **Code quality** | ⚠️ Good | 30+ TODOs, some panics in production code paths |
| **Security** | ❌ Not ready | No real auth; dummy encryption key fallback in postgres |
| **Test coverage** | ⚠️ Partial | `cloud-client` well-covered; `postgres`/`server`/`object-store` weak |
| **Documentation** | ⚠️ Partial | Core types documented; generated code and CLI weak |
| **Feature completeness** | ⚠️ Partial | ~80% of CRUD complete; audit trail, ownership, filtering missing |
| **Performance** | ⚠️ Unknown | Keyset pagination good; no benchmarks; delta log service unoptimized |

---

## Architecture Overview

### Crate Dependency Graph

```
cli ──────────────────────────────────────────────────┐
                                                       ▼
postgres ──► server ──► common ◄── derive (proc macro)
                ▲           ▲
          sharing-client  (types)
                ▲
          cloud-client
                ▲
          object-store ──► client ◄── acceptance
```

### Code Generation Pipeline

```
proto/ (buf.yaml)
  ├──► prost           → protobuf message types
  ├──► tonic           → gRPC stubs
  └──► crates/build    → custom codegen:
        ├── handler.rs     → trait definitions per service (e.g. CatalogHandler<Cx>)
        ├── server.rs      → axum route extractors + dispatch
        ├── client.rs      → HTTP client methods
        ├── builders.rs    → fluent builder API
        └── python/node/   → FFI binding helpers
```

The `crates/build` generator operates in three phases:
1. **Parse** — extract service/method/field metadata from protobuf `FileDescriptorSet`
2. **Analyze** — organize into `ServicePlan`/`MethodPlan` structures, classify HTTP patterns
3. **Generate** — emit Rust source via `quote!` macros

### Server Handler Design: Trait-Based Architecture

Each resource type has a generated generic trait:

```rust
#[async_trait]
pub trait CatalogHandler<Cx = RequestContext>: Send + Sync + 'static {
    async fn list_catalogs(&self, request: ListCatalogsRequest, context: Cx)
        -> Result<ListCatalogsResponse>;
    async fn create_catalog(&self, request: CreateCatalogRequest, context: Cx)
        -> Result<Catalog>;
    // ...
}
```

A single struct can implement all handler traits and be registered via `Router::merge`:

```rust
let handler = Arc::new(MyBackend::new(...));
let app = Router::new()
    .merge(catalogs::routes(handler.clone()))
    .merge(tables::routes(handler.clone()))
    .merge(schemas::routes(handler.clone()));
```

**Trade-off: Trait objects vs pure generics (monomorphization)**

| | Current (trait + blanket impl) | Pure generics / monomorphization |
|---|---|---|
| Composability | ✅ One struct implements all handlers | ❌ Each type combo generates new code |
| Vtable overhead | ~8 bytes/call (acceptable) | None |
| Middleware wrapping | ✅ Easy: `LoggingWrapper<T: TableHandler>` | ❌ Complex |
| Axum integration | ✅ Clean `.with_state()` | More complex |
| Compile time | ✅ Fast | ❌ Slower |
| Custom context | ✅ Generic `Cx` type supported | ❌ Harder to extend |

**Verdict:** The trait-based approach is correct for this domain. It enables external implementors to swap backends crate-wide, supports middleware composition cleanly, and allows custom `Cx` context types for different principal representations.

### Authorization Pattern

Request objects implement the `SecuredAction` trait to declare their own authorization requirements:

```rust
impl SecuredAction for CreateCatalogRequest {
    fn resource(&self) -> ResourceIdent { ... }
    fn permission(&self) -> &Permission { &Permission::Create }
}
```

Handlers call `check_required()` before any side effects, and `process_resources()` filters list results by authorization — avoiding N+1 authorization checks.

---

## Per-Crate Analysis

### `crates/common` — Shared Types & Error Definitions

**Strengths:**
- `ResourceName` has extensive doctests and edge-case coverage (escaped backticks, special chars)
- `ResourceExt` trait has clean blanket implementations for all resource types
- Error enum includes machine-readable error codes (useful for client-side branching)
- Conditional compilation via feature flags (`grpc`, `axum`, `sqlx`, `python`)

**Issues:**
- `PropertyMap` type alias lacks a doc comment
- Model fields rely on proto documentation — not visible in `cargo doc` output
- Error type conversions between crates are inconsistent: some use `#[from]`, others are manual

**Test coverage:** Good for `ResourceName` (doctests + unit tests); minimal for error types and model serialization round-trips.

---

### `crates/client` — HTTP REST Client

**Strengths:**
- Rich error hierarchy: 8 top-level variants + 8 `UcApiError` subtypes matching the UC API spec
- Ergonomic helpers: `is_not_found()`, `is_already_exists()`, `is_permission_denied()`
- Stream-based pagination via `BoxStream` — composable with async consumers
- Builder pattern for all mutating operations

**Issues:**
- `create_function()` has 11 parameters with `#[allow(clippy::too_many_arguments)]` (`src/lib.rs:329`) — a builder is needed
- Generated builder methods lack doc comments
- Pagination stream logic (`stream_paginated()`) is undocumented and untested in isolation
- Happy-path client operations are only tested via the acceptance crate (integration)

**Test coverage:** Good for error parsing; poor for client operations and pagination.

---

### `crates/server` — Axum REST Server

**Strengths:**
- Clean codegen/hand-written split: traits and routes are generated; business logic is hand-written
- `SecuredAction` pattern elegantly keeps authz requirements with the request type
- `process_resources()` prevents N+1 authorization on list operations
- Error enum maps to both HTTP status codes and gRPC status codes (feature-gated)

**Issues — TODOs (20+ occurrences):**
- Ownership not set on create: actor is not recorded as owner
  - `src/api/catalogs.rs:39-41`
  - `src/api/tables.rs:198-199`
  - `src/api/schemas.rs:31`
- Audit trail associations (`CreatedBy`/`UpdatedBy`) not implemented; blocked by missing `AssociationLabel` variants (`src/api/mod.rs:31`)
- `like_pattern` filtering not implemented for table listing (`src/api/tables.rs:107`, `tables.rs:132`)
- `max_results` bound not enforced for table listing (≤50 per spec) (`tables.rs:130`)
- Storage location not validated against registered locations for volumes (`src/api/volumes.rs:31`)
- External locations: no accessibility validation with credential (`external_locations.rs:40`); no in-use check before delete (`external_locations.rs:54`)
- Delta log predicates not applied to query streams (`src/services/delta_log.rs:190`)
- Delta log partition value formatting incomplete (`delta_log.rs:266`)
- O(n) scan of all external locations on every object-store request (`src/services/object_store.rs:50`)

**Auth:** Only `AnonymousAuthenticator` is implemented. `ReverseProxyAuthenticator` (extract `X-Forwarded-User`) is a TODO (`src/rest/auth.rs:29-30`).

**Test coverage:** Good for auth middleware, input validation, and in-memory store; gaps in handler business logic, resource associations, and delta log service.

---

### `crates/postgres` — PostgreSQL Backend

**Strengths:**
- SQLx compile-time checked queries — no SQL injection risk
- Keyset pagination with UUID cursor — scalable, no offset drift problems
- PGP symmetric encryption for secrets via `pgp_sym_encrypt`/`pgp_sym_decrypt`
- Smart PostgreSQL constraint code mapping: 23505 (unique violation) → `AlreadyExists`, 23503 (FK violation) → `NotFound`

**Critical Security Issue:**
- `src/secrets.rs:8` — `const DEFAULT_ENCRYPTION_KEY: &str = "dummy"` is used as a fallback when `encryption_key` is `None`. Secrets are silently encrypted with a publicly known key. This must be a hard error, not a silent default.

**Issues:**
- Connection pool max size hardcoded at 96 (`src/graph/store.rs:37`) — not configurable
- `remove_association` is a `todo!()` stub (`src/resources.rs:180`) — association cleanup is broken
- `src/secrets.rs:73` — `.unwrap()` on UTF-8 conversion of secret value — panics on non-UTF-8 secrets

**Test coverage:** Only `pagination.rs` has unit tests (3 tests for token serialization). `secrets.rs`, core store operations, and association logic have no tests.

---

### `crates/cloud-client` — Cloud Credential & HTTP Client

**Strengths:**
- `TokenCache` with min TTL prevents thundering herd on concurrent credential expiry
- Explicit warning/error for `allow_invalid_certificates` — good security hygiene
- Comprehensive retry logic with exponential backoff
- Best test coverage in the repo: 13 test modules using `mockito` for HTTP mocking

**Issues:**
- `#[allow(unused, dead_code)]` at module level in `lib.rs:1` — overly broad; dead code should be removed
- `BearerTokenSigner` clones the token string on every request — minor allocation overhead
- Unwraps in the recording feature (`lib.rs:515-516`) — dev/test code but could panic unexpectedly
- `RequestSigner` and `CredentialProvider` traits lack doc comments despite being public APIs

**Test coverage:** Good (mocked via `mockito`); no end-to-end tests with real cloud providers.

---

### `crates/object-store` — Object Storage Abstraction

**Strengths:**
- Proper credential type-checking: `as_aws`, `as_azure`, `as_gcp` return typed errors on mismatch
- `TokenCache` integration for transparent credential refresh
- Good doc comments on factory builder methods

**Issues:**
- Only one `#[ignore]`d integration test requiring live UC server + credentials (`src/lib.rs:255`)
- No unit tests for `credential.rs` conversion logic
- `src/credential.rs:172` — `access_point` field from UC response is silently ignored (TODO)

**Test coverage:** Effectively zero unit test coverage.

---

### `crates/sharing-client` — Delta Sharing Protocol

**Strengths:**
- Full discovery → metadata → query protocol flow
- Streaming pagination via `BoxStream` — consistent with `crates/client` API style
- Standalone: usable without a Unity Catalog server
- Builder pattern with `IntoFuture` for ergonomic query construction

**Issues:**
- `.unwrap()` on URL parsing and joining: `src/client.rs:23,38,40,256` — should propagate as `Result`
- `.unwrap()` on JSON deserialization in `src/models/mod.rs:73,74,91,351,370,406,424` — panics on malformed server responses

**Test coverage:** No unit tests. Deserialization and URL construction are entirely untested.

---

### `crates/build` — Code Generator

**Strengths:**
- Clean two-phase architecture: parse → analyze → generate
- Unified type system bridges proto types ↔ Rust types with context awareness (builder, client, constructor)
- Generates handler traits, axum extractors, HTTP clients, builders, and FFI bindings from one proto input
- Custom `google.api.http` annotation parsing drives REST path/query parameter extraction

**Issues:**
- `#[allow(unused)]` in `src/lib.rs:13,21` — dead code should be removed
- `TODO: make configurable` at `src/codegen/client.rs:27`
- Oneof documentation extraction is a stub (`src/parsing/message.rs:150`)

---

### `crates/derive` — Procedural Macros

**Strengths:**
- `object_conversions!` macro generates compile-time-verified `TryFrom<Object>` / `TryFrom<T>` / `From<T> for Resource` for all domain types
- Declarative and zero-cost (inlined by compiler)
- Eliminates significant repetitive conversion boilerplate

**No significant issues.** This crate is well-factored and stable.

---

### `crates/acceptance` — Journey-Based Testing Framework

**Strengths:**
- Tests full workflows (create → list → get → delete) rather than isolated functions
- HTTP request/response recording enables deterministic CI replay without a live server
- `JourneyLogger` tracks per-step performance metrics
- Markdown progress reporting with step-level status

**Issues:**
- `#[allow(unused_variables)]` at `src/execution/mod.rs:35,44` — suggests setup/cleanup lifecycle is incomplete
- Only one journey implemented (`CatalogSimpleJourney`); no journeys for tables, schemas, volumes, functions, or sharing protocol

---

## Cross-Cutting Concerns

### Panic Points in Non-Test Production Code

| File | Lines | Issue |
|---|---|---|
| `crates/sharing-client/src/client.rs` | 23, 38, 40, 256 | `.unwrap()` on URL joining |
| `crates/sharing-client/src/models/mod.rs` | 73, 74, 91, 351, 370, 406, 424 | `.unwrap()` on JSON deserialization |
| `crates/server/src/services/delta_log.rs` | 38, 40 | `.unwrap()` on Arrow schema conversions |
| `crates/server/src/services/object_store.rs` | 66 | `.unwrap()` on `StorageLocationUrl` parsing |
| `crates/postgres/src/secrets.rs` | 73 | `.unwrap()` on UTF-8 conversion of secret value |
| `crates/cli/src/client.rs` | 92 | `.unwrap()` on URL construction |

### Error Type Inconsistency

Each crate correctly defines its own `Error` type with `thiserror`. However, `From` conversions between crate error types are inconsistent — some use `#[from]`, others require manual mapping. This creates gaps where error context is silently dropped during propagation.

### Documentation Coverage

| Crate | Estimated Public API Coverage |
|---|---|
| `common` | ~80% |
| `client` | ~60% (generated builders undocumented) |
| `server` | ~70% (trait impls undocumented) |
| `postgres` | ~70% |
| `cli` | ~40% |
| `cloud-client` | ~85% |
| `object-store` | ~75% |
| `sharing-client` | ~65% |
| `build` | ~60% |
| `derive` | ~80% |

---

## Functionality Gaps

### Missing or Stubbed Features

1. **Ownership tracking** — `create_*` handlers don't record the actor as resource owner
2. **Audit trail** — `CreatedBy`/`UpdatedBy` associations not implemented; blocked by missing `AssociationLabel` variants
3. **`remove_association`** — `todo!()` stub in `crates/postgres/src/resources.rs:180`
4. **`ReverseProxyAuthenticator`** — no way to extract a real user identity from headers
5. **Storage location validation** — volumes and external locations don't verify URL accessibility with credential
6. **Table `like_pattern` filtering** — `schema_name_pattern` and `table_name_pattern` not applied
7. **`max_results` enforcement** — no upper bound enforced on table listing (spec requires ≤50)
8. **Delta log query predicates** — not pushed down into stream; queries return all data
9. **Partition value type formatting** — incomplete type handling in delta log service
10. **CLI `migrate` command** — `todo!()` at `crates/cli/src/main.rs:65`
11. **Metastore management** — no API surface for metastore-level configuration
12. **Workspace configuration** — no workspace-level settings endpoints
13. **Lineage / audit querying** — no endpoints to query the audit trail
14. **Acceptance journeys** — only `CatalogSimpleJourney` exists; tables, schemas, volumes, functions, sharing have none

### Security Gaps (Production Blockers)

1. **`DEFAULT_ENCRYPTION_KEY = "dummy"`** — `crates/postgres/src/secrets.rs:8` — secrets encrypted with known key if config omitted
2. **No real authentication** — `AnonymousAuthenticator` always passes; no JWT, OAuth, OIDC, or header-based extraction
3. **No input length limits** — resource names and descriptions are not length-bounded
4. **No rate limiting** — server has no request rate limiting middleware
5. **HTTP default in CLI** — default server URL is `http://localhost:8080` (cleartext)

---

## Recommended Next Steps

### P0 — Security / Correctness Blockers

1. **Remove `DEFAULT_ENCRYPTION_KEY` fallback** (`crates/postgres/src/secrets.rs:8`)
   Replace `unwrap_or(DEFAULT_ENCRYPTION_KEY)` with a hard startup error if the encryption key is not configured.

2. **Implement `ReverseProxyAuthenticator`** (`crates/server/src/rest/auth.rs:29-30`)
   Extract `X-Forwarded-User` (and optionally `X-Forwarded-Groups`) from request headers; make the header name configurable via CLI config.

3. **Fix production panics** — replace `.unwrap()` with `?` propagation in:
   - `crates/sharing-client/src/client.rs:23,38,40,256`
   - `crates/server/src/services/delta_log.rs:38,40`
   - `crates/server/src/services/object_store.rs:66`
   - `crates/postgres/src/secrets.rs:73`
   - `crates/cli/src/client.rs:92`

### P1 — Core Feature Completeness

4. **Implement `remove_association`** — `crates/postgres/src/resources.rs:180`

5. **Add `AssociationLabel::CreatedBy` / `UpdatedBy` variants** — unblocks ownership tracking and audit trail across all API handlers

6. **Wire ownership tracking** — after P1.5, update `create_catalog`, `create_table`, `create_schema` to set the current actor as owner

7. **Enforce `max_results` bounds** — `crates/server/src/api/tables.rs:130` (≤50 per UC spec)

8. **Implement `like_pattern` filtering** — `tables.rs:107,132` for schema and table name filtering

9. **Validate storage locations** — external locations and volumes should verify URL accessibility with the provided credential before accepting the resource

### P2 — Test Coverage

10. **`postgres` unit tests** — `secrets.rs` (mock PG via `sqlx::test`), `store.rs` CRUD, association logic

11. **`server` handler tests** — test business logic in all API handlers beyond auth and validation

12. **`sharing-client` tests** — unit tests for URL construction, model deserialization, and pagination

13. **`object-store` unit tests** — test `as_aws`, `as_azure`, `as_gcp` credential conversion functions

14. **Acceptance journeys** — add journeys for tables, schemas, volumes, functions, and the sharing protocol

### P3 — Ergonomics / Developer Experience

15. **Builder for `create_function`** (`crates/client/src/lib.rs:329`) — replace 11-argument function with a fluent builder

16. **Configurable connection pool** — make postgres max connections a config option (currently hardcoded 96 in `crates/postgres/src/graph/store.rs:37`)

17. **Doc comments on public traits** — `RequestSigner`, `CredentialProvider` in `cloud-client`; `ServerHandler`/`ServerHandlerInner` in `server`

18. **Remove broad `#[allow]` suppressions** — clean up `crates/cloud-client/src/lib.rs:1` and `crates/build/src/lib.rs:13,21`

19. **CLI subcommand documentation** — add `///` doc comments to all clap enum variants in `crates/cli/`

### P4 — Architecture Improvements

20. **Consistent `#[from]` error conversions** — audit all inter-crate error `From` paths and apply `#[from]` systematically with `thiserror`

21. **Delta log predicate pushdown** — implement predicate application in `crates/server/src/services/delta_log.rs:190` to reduce data scanned per query

22. **Object store location lookup** — replace full external_locations scan with targeted lookup (`crates/server/src/services/object_store.rs:50`)

23. **Metastore / workspace API surface** — define proto RPCs and generate stubs for missing API surfaces

24. **Rate limiting middleware** — add Tower-based rate limiter (e.g., `tower_governor`) to the axum router in the CLI server startup

25. **HTTPS default** — change CLI default server URL to `https://` and add TLS certificate config options
