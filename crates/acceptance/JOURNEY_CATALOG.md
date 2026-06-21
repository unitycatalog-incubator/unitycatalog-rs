# Unity Catalog Acceptance Journey Catalog

This document describes the acceptance testing strategy for `unitycatalog-rs`, the available
journeys, their recording status, and how to run them against different Unity Catalog implementations.

---

## Overview

The acceptance crate implements end-to-end integration tests as **user journeys** — sequences of
API calls that exercise real workflows. Each journey can operate in two modes:

| Mode | Description |
|------|-------------|
| **Replay** (default) | Loads pre-recorded HTTP interactions from `recordings/<profile>/` and replays them against a mockito mock server, once per implementation profile. Fast and deterministic — no live server needed. |
| **Live** | Executes against a real Unity Catalog server. Set `UC_INTEGRATION_RECORD=true` to also capture the interactions as new recordings. |

---

## Journey Tiers

Journeys are grouped into tiers based on complexity and external dependencies:

| Tier | Name | Description |
|------|------|-------------|
| 1 | `Tier1Crud` | Basic CRUD — no external dependencies, compatible with all implementations |
| 2 | `Tier2Governance` | Credentials, external locations, volumes, temporary credentials |
| 3 | `Tier3Sharing` | Delta Sharing — shares and recipients |
| 4 | `Tier4Advanced` | UDFs, cross-resource multi-step workflows |

---

## Journey Catalog

### Tier 1 — Basic CRUD

Most Tier 1 journeys are compatible with **all implementations**
(`ImplementationTag::All`); the table/metric-view journeys are the exceptions
noted below (OSS Java limitations).

| Journey Name | File | Resources | Steps | Recording Status |
|---|---|---|---|---|
| `enhanced_catalog` | `tier1/catalog_simple.rs` | Catalogs | create → list → inspect → delete | ✅ Recorded (databricks, oss_rust, oss_java) |
| `catalog_hierarchy` | `tier1/catalog_hierarchy.rs` | Catalogs, Schemas | catalog + 3 schemas → list → verify → delete all | ✅ Recorded (databricks, oss_rust, oss_java) |
| `schema_lifecycle` | `tier1/schema_lifecycle.rs` | Catalogs, Schemas | create catalog → update catalog comment → create schema → get → list → update comment → delete | ✅ Recorded (databricks, oss_rust, oss_java) |
| `table_managed_lifecycle` | `tier1/table_managed_lifecycle.rs` | Catalogs, Schemas, Tables | catalog + schema → create MANAGED DELTA table → get → list → list summaries → exists → delete | ⏳ Pending recording |
| `metric_view_lifecycle` | `tier1/metric_view_lifecycle.rs` | Catalogs, Schemas, Tables | catalog + schema → create METRIC_VIEW (YAML `view_definition`) → get → list → delete | ✅ Recorded (oss_rust). OssRust + ManagedDatabricks only — OSS Java v0.4.1 lacks the `METRIC_VIEW` type and `view_definition` field (added upstream in v0.5.0); add `OssJava` once a v0.5.0+ image is available. |

### Tier 2 — Governance

| Journey Name | File | Compatible Impls | Requires | Resources | Steps | Recording Status |
|---|---|---|---|---|---|---|
| `credential_lifecycle` | `tier2/credential_lifecycle.rs` | ManagedDatabricks | — | Credentials | create → get → list → update comment → delete | ⏳ Pending recording |
| `external_location_lifecycle` | `tier2/external_location_lifecycle.rs` | ManagedDatabricks | External storage | ExternalLocations, Credentials | credential → external location → list → delete | ⏳ Pending recording |
| `volume_managed_lifecycle` | `tier2/volume_managed_lifecycle.rs` | All | — | Volumes, Catalogs, Schemas | catalog + schema → MANAGED volume → get → list → delete | ⏳ Pending recording |
| `volume_external_lifecycle` | `tier2/volume_external_lifecycle.rs` | ManagedDatabricks | External storage | Volumes, ExternalLocations, Credentials | full chain → EXTERNAL volume → get → delete | ⏳ Pending recording |
| `table_external_lifecycle` | `tier2/table_external_lifecycle.rs` | ManagedDatabricks | External storage | Tables, ExternalLocations, Credentials | full chain → EXTERNAL table → get → delete | ⏳ Pending recording |
| `temporary_table_credentials` | `tier2/temporary_table_credentials.rs` | ManagedDatabricks | — | TemporaryCredentials, Tables | managed table → generate read + read-write temp creds | ⏳ Pending recording |
| `temporary_path_credentials` | `tier2/temporary_path_credentials.rs` | ManagedDatabricks | External storage | TemporaryCredentials, ExternalLocations | external location → generate read + read-write path creds | ⏳ Pending recording |
| `temporary_volume_credentials` | `tier2/temporary_volume_credentials.rs` | ManagedDatabricks | — | TemporaryCredentials, Volumes | managed volume → generate read + read-write temp creds | ⏳ Pending recording |

### Tier 3 — Delta Sharing

| Journey Name | File | Compatible Impls | Resources | Steps | Recording Status |
|---|---|---|---|---|---|
| `share_lifecycle` | `tier3/share_lifecycle.rs` | ManagedDatabricks, OssRust | Shares, Tables | table → create share → get → list → delete | ⏳ Pending recording |
| `recipient_lifecycle` | `tier3/recipient_lifecycle.rs` | ManagedDatabricks, OssRust | Recipients | create TOKEN recipient → get → list → delete | ⏳ Pending recording |
| `provider_lifecycle` | `tier3/provider_lifecycle.rs` | ManagedDatabricks, OssRust | Providers | create TOKEN provider → get → list → update comment → delete | ✅ Recorded (databricks) |

### Tier 4 — Advanced

| Journey Name | File | Compatible Impls | Resources | Steps | Recording Status |
|---|---|---|---|---|---|
| `function_lifecycle` | `tier4/function_lifecycle.rs` | OssRust, ManagedDatabricks | Functions, Catalogs, Schemas | catalog + schema → create SQL UDF → get → list → delete | ⏳ Pending recording |
| `lakehouse_hierarchy` | `cross_resource/lakehouse_hierarchy.rs` | All | Catalogs, Schemas, Tables, Volumes | catalog → 2 schemas → managed table + volume in each → verify → delete | ⏳ Pending recording |

### Cross-Resource (Tier 4)

| Journey Name | File | Compatible Impls | Requires | Resources | Steps | Recording Status |
|---|---|---|---|---|---|---|
| `governance_setup` | `cross_resource/governance_setup.rs` | ManagedDatabricks | External storage | Catalogs, Schemas, Credentials, ExternalLocations, Tables | full governance chain: catalog → schema → credential → ext_loc → external table | ⏳ Pending recording |

---

## Environment Variables

### Connection & Authentication

| Variable | Default | Description |
|---|---|---|
| `UC_INTEGRATION_URL` | `http://localhost:8080` | Base URL of the Unity Catalog server |
| `UC_INTEGRATION_TOKEN` | — | Bearer token for authentication |
| `UC_INTEGRATION_STORAGE_ROOT` | `file:///tmp/uc-test/` | Storage root threaded into every journey via `JourneyContext`; used as the catalog `MANAGED LOCATION` and for external resources. Override per profile (e.g. `s3://my-bucket/uc-test/` for Databricks). |
| `UC_INTEGRATION_RECORD` | `false` | Set to `true` to record live interactions as new fixture files |
| `UC_INTEGRATION_DIR` | `recordings/` | Base directory for recordings. Cassettes are namespaced per profile: `<dir>/<profile>/<journey>/`. |

### Resource-specific (managed Databricks)

Some journeys need real workspace values to succeed against Databricks. When the relevant
variable is unset, the journey either skips (credentials) or falls back to an OSS-friendly
default (recipient owner).

| Variable | Used by | Description |
|---|---|---|
| `UC_TEST_AWS_ROLE_ARN` | `credential_lifecycle` | AWS IAM role ARN for the storage credential. If set, an AWS credential is created. |
| `UC_TEST_AZURE_ACCESS_CONNECTOR_ID` | `credential_lifecycle` | Azure Databricks Access Connector resource ID (alternative to the AWS role). If neither AWS nor Azure var is set, `credential_lifecycle` is skipped. |
| `UC_TEST_RECIPIENT_OWNER` | `recipient_lifecycle` | Principal (user/group/SP) that owns the recipient. Must exist on Databricks. Defaults to `account users`. |

### Journey Selection

| Variable | Default | Description |
|---|---|---|
| `UC_JOURNEY_INCLUDE` | — | Comma-separated journey names to run (empty = all) |
| `UC_JOURNEY_EXCLUDE` | — | Comma-separated journey names to skip |
| `UC_JOURNEY_IMPL` | — | Filter by implementation: `oss_rust`, `oss_java`, `managed_databricks` |
| `UC_JOURNEY_MAX_TIER` | — | Only run journeys up to this tier: `tier1`, `tier2`, `tier3`, `tier4` |

### Profile Selection (for live tests)

| Variable | Values | Description |
|---|---|---|
| `UC_INTEGRATION_PROFILE` | `oss_rust`, `oss_java`, `managed_databricks` | Activates the `journey_tests_live` test with the named profile |

---

## How to Run

### Run replay tests (CI, no server needed)

```bash
cargo test -p unitycatalog-acceptance
```

### Run only Tier 1 journeys in replay

```bash
UC_JOURNEY_MAX_TIER=tier1 cargo test -p unitycatalog-acceptance
```

### Skip a specific journey

```bash
UC_JOURNEY_EXCLUDE=catalog_hierarchy cargo test -p unitycatalog-acceptance
```

### Run against the local Rust OSS server

```bash
# Start the server first:
just rest

# Run live (without recording)
UC_INTEGRATION_PROFILE=oss_rust \
  UC_INTEGRATION_URL=http://localhost:8080 \
  cargo test -p unitycatalog-acceptance -- journey_tests_live

# Run live and record new fixtures
UC_INTEGRATION_PROFILE=oss_rust \
  UC_INTEGRATION_URL=http://localhost:8080 \
  UC_INTEGRATION_RECORD=true \
  cargo test -p unitycatalog-acceptance -- journey_tests_live
```

### Run against the open-source Java Unity Catalog server

This boots the Java OSS server in Docker, waits for its healthcheck, and runs every
journey compatible with the `OssJava` implementation tag. This is the same flow the
`integration-oss-java` CI job runs.

```bash
just integration-oss-java

# Tear down when done:
docker compose -f dev/uc-oss.compose.yaml down -v
```

To run it by hand against an already-running Java server:

```bash
UC_INTEGRATION_PROFILE=oss_java \
  UC_INTEGRATION_URL=http://localhost:8080 \
  cargo test -p unitycatalog-acceptance -- journey_tests_live
```

### Record against Databricks managed Unity Catalog (reference implementation)

```bash
UC_INTEGRATION_PROFILE=managed_databricks \
  UC_INTEGRATION_URL=https://your-workspace.azuredatabricks.net \
  UC_INTEGRATION_TOKEN=dapi... \
  UC_INTEGRATION_STORAGE_ROOT=s3://your-bucket/uc-test/ \
  UC_INTEGRATION_RECORD=true \
  cargo test -p unitycatalog-acceptance -- journey_tests_live --nocapture
```

This records to `recordings/managed_databricks/<journey>/`. `UC_INTEGRATION_STORAGE_ROOT`
**must** be a real, writable location your workspace can use as a catalog `MANAGED
LOCATION` — Databricks rejects catalog creation without it. For storage-credential and
recipient coverage, also set `UC_TEST_AWS_ROLE_ARN` (or `UC_TEST_AZURE_ACCESS_CONNECTOR_ID`)
and `UC_TEST_RECIPIENT_OWNER` (see the resource-specific env vars above).

After recording, commit the new fixture files in `recordings/<profile>/<journey>/` alongside
the journey source code in the same commit.

---

## How to Add a New Journey

1. **Implement the `UserJourney` trait** in an appropriate tier module:
   - Create `crates/acceptance/src/journeys/tierN/my_journey.rs`
   - Implement `name()`, `description()`, `metadata()`, `execute()`, and optionally `setup()`,
     `cleanup()`, `save_state()`, `load_state()`. The `execute`/`setup`/`cleanup` methods receive
     a `&JourneyContext` — use `ctx.client()` for the client and `ctx.storage_root` as the catalog
     `MANAGED LOCATION` / external-resource root (do **not** hardcode a bucket).
   - Return accurate `JourneyMetadata` (resources, implementations, tier, requires_external_storage).
     Journeys that touch external storage must set `requires_external_storage: true` so they're
     filtered out for the OSS profiles.

2. **Register the journey** in `crates/acceptance/src/journeys/mod.rs`:
   - Add it to the appropriate `mod.rs` in its tier
   - Add it to `all_journeys()` (all journeys live here now; the per-profile/tier/storage
     filtering happens at runtime via `JourneyMetadata` + `JourneyFilter`)

3. **Record fixtures** against the appropriate reference implementation:
   ```bash
   UC_INTEGRATION_RECORD=true UC_JOURNEY_INCLUDE=my_journey \
     UC_INTEGRATION_PROFILE=managed_databricks \
     ... cargo test -p unitycatalog-acceptance -- journey_tests_live
   ```
   Commit the generated `recordings/<profile>/my_journey/` directory with the journey source.

---

## Recording Fixture Format

Recordings are **namespaced per implementation profile** so the same journey recorded
against different servers doesn't collide:

```
recordings/
  managed_databricks/
    enhanced_catalog/
    schema_lifecycle/
    ...
  oss_rust/
    enhanced_catalog/
    ...
```

Each journey gets a directory: `recordings/<profile>/<journey_name>/`

- **`0000.json`, `0001.json`, ...**  — numbered HTTP request/response pairs in execution order
- **`journey_state.json`** — key-value snapshot of the journey's state (e.g. generated names,
  IDs) needed to restore context during replay

During replay, the mock server registers each recording as an independent mock with `expect(1)`.
When a recording has a JSON request body, the mock also matches on that body using
`mockito::Matcher::Json` — this correctly disambiguates multiple calls to the same endpoint with
different payloads (e.g. three `POST /schemas` with different names).

---

## Implementation Compatibility Matrix

| Journey | All Impls | OssRust | OssJava | ManagedDatabricks |
|---|---|---|---|---|
| `enhanced_catalog` | ✅ | ✅ | ✅ | ✅ |
| `catalog_hierarchy` | ✅ | ✅ | ✅ | ✅ |
| `schema_lifecycle` | ✅ | ✅ | ✅ | ✅ |
| `table_managed_lifecycle` | — | ✅ | —¹ | ✅ |
| `volume_managed_lifecycle` | — | ✅ | —¹ | ✅ |
| `lakehouse_hierarchy` | — | ✅ | —¹ | ✅ |
| `credential_lifecycle` | — | — | — | ✅ |
| `external_location_lifecycle` | — | — | — | ✅ |
| `volume_external_lifecycle` | — | — | — | ✅ |
| `table_external_lifecycle` | — | — | — | ✅ |
| `temporary_table_credentials` | — | — | — | ✅ |
| `temporary_path_credentials` | — | — | — | ✅ |
| `temporary_volume_credentials` | — | — | — | ✅ |
| `governance_setup` | — | — | — | ✅ |
| `share_lifecycle` | — | ✅ | — | ✅ |
| `recipient_lifecycle` | — | ✅ | — | ✅ |
| `provider_lifecycle` | — | ✅ | — | ✅ |
| `function_lifecycle` | — | ✅ | — | ✅ |

¹ The Java OSS server (`v0.4.1`, local file storage) returns `500 [INTERNAL]
  "stagingLocation is null"` when creating MANAGED tables/volumes, so these
  journeys are not OssJava-compatible without configured cloud storage.

  (The earlier `VOLUME_TYPE_MANAGED` wire-format mismatch — the Java server
  expects bare `MANAGED` — was fixed by renaming the proto enum values; see
  `API_COMPATIBILITY.md`.)
