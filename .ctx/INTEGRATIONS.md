# Integration Extension Plan

## Context

The server has clean, trait-based extension points for storage, secrets, and authorization but currently only ships with in-memory and PostgreSQL implementations. The goal is to extend coverage across cloud secret backends, policy engines, and optionally other database backends — while keeping the dependency tree lean and maintenance burden manageable. The `cloud-client` crate already provides auth for AWS, Azure, GCP, and Databricks, making HTTP-based integrations cheap.

## Current Extension Points

| Trait | Location | Current Impls |
|---|---|---|
| `ResourceStore` | `crates/server/src/store.rs` | In-memory, PostgreSQL (graph model) |
| `SecretManager` | `crates/server/src/services/secrets.rs` | In-memory only |
| `Policy<Cx>` | `crates/server/src/policy/mod.rs` | `ConstantPolicy` (always allow/deny) |
| `Authenticator<I>` | `crates/server/src/rest/auth.rs` | `AnonymousAuthenticator` |

## Cross-Cutting Patterns

- Each new integration lives in its own crate under `crates/` (e.g., `crates/secrets-aws/`)
- Feature-flagged in the server/CLI `Cargo.toml`: `features = ["secrets-aws"]` etc. — off by default
- All HTTP-based integrations use `CloudClient` from `crates/cloud-client/` — no new HTTP client deps
- Follow the existing `AmazonBuilder` / `DatabricksBuilder` builder pattern with a `ConfigKey` enum
- Map provider HTTP errors to `unitycatalog_server::Error` variants (`NotFound`, `NotAllowed`, etc.)
- Integration tests gated behind `integration-<provider>` feature flags (mirrors postgres pattern)

## Prioritized Roadmap

### Priority 1 — Quick Wins (no new crates needed)

**ReverseProxyAuthenticator** — `crates/server/src/rest/auth.rs`
- Read `X-Forwarded-User` header (configurable) → `Principal::User(name)`; missing header → `Principal::Anonymous`
- Also parse `X-Forwarded-Groups` for future group-based policy
- Zero new dependencies; unlocks real auth for nginx/Envoy/ALB deployments
- Effort: 2 days | Maintenance: Negligible

### Priority 2 — Cloud Secret Backends (Tier 1)

All three follow the same pattern: new crate, `SecretManager` impl, `CloudClient` for auth, REST API calls.

**AWS Secrets Manager** — `crates/secrets-aws/`
- `CloudClient::new_aws()` with SigV4; call `GetSecretValue`, `CreateSecret`, `PutSecretValue`, `DeleteSecret`
- Map AWS `VersionId` string → `Uuid` (parse directly; AWS VersionIds are UUID-formatted)
- Config env vars: `SECRETS_AWS_REGION`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, optional `SECRETS_AWS_NAME_PREFIX`
- Effort: 4 days | Maintenance: Low

**GCP Secret Manager** — `crates/secrets-gcp/`
- `CloudClient::new_google()` with bearer token; call `projects/{project}/secrets/{name}/versions/latest`
- GCP versions are integers; use UUID v5 namespace keyed on `(name, version_int)` for stable mapping
- Effort: 4 days | Maintenance: Low

**Azure Key Vault** — `crates/secrets-azure/`
- `CloudClient::new_azure()` — note: Key Vault uses scope `https://vault.azure.net/.default`, not ARM scope; verify `AzureConfig` builder supports `with_scope()` before starting
- REST API v7.4: `GET/PUT /secrets/{name}`, `DELETE /secrets/{name}`
- Azure version strings are 32-char hex UUIDs — parse directly
- Effort: 5 days | Maintenance: Low-medium (track API versioning)

### Priority 3 — Policy Engines (Tier 2)

**OPA (Open Policy Agent)** — `crates/policy-opa/`
- `Policy<Cx: Serialize>` impl; single `POST /v1/data/unitycatalog/authz/allow` call per check
- Batch `authorize_many` → single OPA query with list of resources to minimize round-trips
- `CloudClient::new_unauthenticated()` or `new_with_token()` for secured OPA endpoints
- Optional TTL cache for repeated identical checks (`moka` crate behind a feature flag)
- Effort: 7 days | Maintenance: Low (OPA REST API is very stable)

**Cedar** — `crates/policy-cedar/`
- Use `cedar-policy` crate (in-process, no HTTP); no network latency
- Map `Principal` → `User::"alice"`, `ResourceIdent` → `Catalog::"name"`, `Permission` → `UC::Action::"Read"`
- Policy store loaded from `.cedar` files or config string; Amazon Verified Permissions is a future extension
- Watch cedar-policy version (v3→v4 was breaking); pin carefully
- Effort: 10 days | Maintenance: Medium (track crate releases)

**HashiCorp Vault** — `crates/secrets-vault/`
- Static token mode: `CloudClient::new_with_token(VAULT_TOKEN)`
- AppRole mode: custom `VaultAppRoleSigner` implementing `RequestSigner`, caches `client_token` via `TokenCache`, re-auths on 403
- KV v2 API: `GET/POST /v1/secret/data/{path}`, `DELETE /v1/secret/metadata/{path}`
- Effort: 7 days | Maintenance: Medium (KV v1 vs v2 confusion, AppRole rotation)

**OpenFGA** — `crates/policy-openfga/` (lower priority)
- REST API: `POST /stores/{store_id}/check`; Zanzibar relation model requires schema design upfront
- Authorization model must be kept in sync with changes to `Permission` enum
- Effort: 12 days | Maintenance: Medium-high

### Priority 4 — SQLite ResourceStore (Tier 3, deferred)

**Recommendation: Defer until there is demonstrated user demand.** The `InMemoryResourceStore` already serves embedded/dev/test use cases well.

If implemented:
- New crate `crates/sqlite/` mirroring `crates/postgres/` structure
- PostgreSQL-specific features to port:
  - ENUMs → `TEXT CHECK`
  - `Text[]` → JSON
  - `JSONB` → `TEXT` JSON
  - `pgcrypto` → app-layer AES-256-GCM (`aes-gcm` pure Rust crate, no C deps)
  - `uuidv7()` → generate in Rust before INSERT
  - `RETURNING` → `last_insert_rowid()` + SELECT
  - Triggers rewritten in SQLite syntax
  - ICU collation → `lower()` in application code
- Run the existing `crates/acceptance/` test suite against both backends to prevent drift
- Effort: 3-4 weeks | Maintenance: High (two complete SQL query sets to maintain)

## Summary Table

| Priority | Integration | Crate | Effort | Maintenance | Uses cloud-client? |
|---|---|---|---|---|---|
| 1 | ReverseProxyAuthenticator | `server` inline | 2 days | Negligible | No |
| 2 | AWS Secrets Manager | `secrets-aws` | 4 days | Low | Yes (SigV4) |
| 3 | GCP Secret Manager | `secrets-gcp` | 4 days | Low | Yes (GCP bearer) |
| 4 | OPA Policy Engine | `policy-opa` | 7 days | Low | Yes |
| 5 | Azure Key Vault | `secrets-azure` | 5 days | Low-medium | Yes (Azure bearer) |
| 6 | HashiCorp Vault | `secrets-vault` | 7 days | Medium | Partially |
| 7 | Cedar Policy | `policy-cedar` | 10 days | Medium | No (in-process) |
| 8 | OpenFGA | `policy-openfga` | 12 days | Medium-high | Yes |
| 9 | SQLite ResourceStore | `sqlite` | 3-4 weeks | High | No |

## Critical Files

- `crates/server/src/services/secrets.rs` — `SecretManager` trait to implement
- `crates/server/src/policy/mod.rs` — `Policy<Cx>` trait to implement
- `crates/server/src/rest/auth.rs` — `Authenticator<I>` + placeholder for ReverseProxyAuthenticator
- `crates/cloud-client/src/lib.rs` — `CloudClient` factory methods and `RequestSigner` pattern
- `crates/postgres/src/secrets.rs` — Reference `SecretManager` impl (error mapping, versioning patterns)
- `crates/postgres/src/graph/store.rs` — Reference `ResourceStore` impl (PostgreSQL-specific queries)

## Verification

Each integration crate should have:
1. Unit tests with mock HTTP responses (`mockito`) for cloud APIs
2. An `integration-<provider>` feature-gated test that hits a real or localstack endpoint
3. The acceptance test suite from `crates/acceptance/` passing for any new `ResourceStore` backend
