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
- the external [`trestle`](https://github.com/open-lakehouse/trestle) codegen
  tool, which holds the custom generation logic. It must be checked out as a
  sibling directory (`../trestle`) for `just generate*` to work.

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
   - Import new types in `python/client/src/lib.rs`; add the client wrapper in
     `python/client/src/client.rs`.
   - Re-run `just generate-code` (regenerates `_client.pyi`), then re-export any
     new pyclass at the package root — see **Hand-written PyO3 helpers** below
     for the `from ._client import Foo as Foo` convention.

### Hand-written PyO3 helpers

A few Python bindings are not proto-derived (e.g. `parse_uc_url`, ergonomic
`temporary_*_credential` methods that resolve name → UUID before calling the
generated RPC). They live in `python/client/src/{client,reference}.rs`; type
checkers can't read their attributes off the compiled `.so` and trestle codegen
doesn't see them. **Never hand-edit `_client.pyi`** (it is fully regenerated).
Instead:

- **Declare the symbol in `python/client/_client_supplement.pyi`.** `just
  generate-code` appends this fragment to the codegen-emitted `_client.pyi`, so
  the merged stub describes the full `_client` runtime surface. The supplement
  lives outside the package dir so type checkers don't validate it standalone.
- **Re-export from the package root** via the PEP 484 form `from ._client import
  Foo as Foo` in `python/client/python/unitycatalog_client/__init__.py` — the
  same idiom used for codegen-derived types. Do this whenever you
  `m.add_class::<Foo>()` (or register a new exception / free function) in
  `python/client/src/lib.rs`. Keep internal helpers (e.g. `parse_uc_url`) out of
  the root re-export list; consumers import them from `..._client` directly.
- **For proto-shaped surface** (a regular `Get/Update/Delete/Create/List` RPC, or
  a `Custom(Post|Patch)` RPC the Python emitter renders), prefer extending the
  proto so trestle generates everything end-to-end.

## Releases

Releases are driven by [release-plz](https://release-plz.dev) from
[Conventional Commits](https://www.conventionalcommits.org). You never bump versions or
write changelogs by hand. The PR title is the squash-merge commit, so CI lints it against
the convention (`commitlint.config.mjs`); the commit type drives the semver bump.

**Each crate versions independently.** release-plz bumps a crate from the commits that
touch it (and bumps its dependents automatically), so `unitycatalog-object-store` and
`datafusion-unitycatalog` can release on their own cadence. Config: `release-plz.toml`.

Because release-plz derives the bump and changelog *from the commits*, **small,
well-scoped commits with the right `type(scope):`** (per `~/.claude/CLAUDE.md`)
directly produce an accurate per-crate history. Prefer several focused commits
over one mixed commit; keep generated output in the same commit as its source.

**How a release happens:**

1. Merge PRs to `main` with conventional-commit titles (`feat:`, `fix:`, `feat(scope)!:`
   for breaking changes).
2. release-plz opens/updates a **Release PR** that bumps the affected crates' versions
   and updates their changelogs. Review it like any PR.
3. **Merging the Release PR** publishes: release-plz tags each changed crate
   (`<crate>-v<version>`) and creates its GitHub Release. The tag then triggers the build
   workflow that attaches artifacts.

**Tags that trigger artifact builds:**

| Tag                          | Builds & attaches            | Workflow                            |
|------------------------------|------------------------------|-------------------------------------|
| `unitycatalog-cli-v*`        | `uc` binaries + GHCR image   | release.yml, docker-release.yml     |
| `unitycatalog-client-py-v*`  | Python wheels (Linux + macOS)| python-release.yml                  |

Other crates get a tag + GitHub Release (changelog) but no extra artifact build. The crates
are not published to crates.io (`publish = false`); release-plz only tags and releases them.

**Notes:**

- Versions live committed in each `Cargo.toml`; release-plz writes them via the Release PR.
  Never edit a version manually and never use a placeholder — artifacts build from the
  committed version at a real commit SHA, which the provenance attestations bind to.
- Release notes are generated by git-cliff (`cliff.toml`); the artifact workflows *append*
  download/verify instructions to that body. To change note formatting, edit `cliff.toml`.
- release-plz needs a `RELEASE_PLZ_TOKEN` secret (PAT/App token) so the tags it creates
  trigger the downstream workflows — the default `GITHUB_TOKEN` cannot.

## AI-assisted contributions

AI-assisted changes are welcome. Understand the diff before submitting it, match
the surrounding style, and don't include code you can't explain. Every commit
carries the `AI-assisted-by: Isaac` trailer and PR bodies end with the
attribution line — both are defined in `~/.claude/CLAUDE.md`. The commit/sign
mechanics live in the `/commit` skill (`.claude/skills/commit/SKILL.md`).
