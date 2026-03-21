# Repository Guidelines

## Project Structure & Module Organization

The project follows a multi-crate Rust workspace architecture with language bindings:

- `crates/` - Core Rust implementation organized by functionality:
  - `common/` - Shared types and utilities
  - `client/` - Unity Catalog client library
  - `server/` - REST API server implementation
  - `postgres/` - PostgreSQL backend integration
  - `cli/` - Command-line interface
  - `cloud-client/` - Cloud provider integration
  - `object-store/` - Cloud object storage abstraction (AWS, Azure, GCP)
  - `sharing-client/` - Delta Sharing protocol client
  - `build/` - Code generation utilities (internal)
  - `derive/` - Custom derive macros (internal)
  - `acceptance/` - Acceptance testing utilities
- `python/client/` - Python bindings via PyO3
- `node/client/` - Node.js bindings via NAPI
- `proto/` - Protocol buffer definitions
- `openapi/` - Generated OpenAPI specifications
- `dev/` - Docker Compose config, development scripts, and Marimo notebooks
- `docs/` - Astro-based documentation site

## Build, Test, and Development Commands

We use [`just`](https://just.systems/) as the primary task runner. Key commands:

**Code generation:**
- `just generate` - Run complete code generation pipeline
- `just generate-full` - Run full generation including external types
- `just generate-code` - Run internal code generation pipeline
- `just generate-proto` - Run code generation pipeline for Protocol Buffers
- `just generate-node` - Generate types for Node.js client
- `just generate-openapi` - Update OpenAPI specification

**Development server:**
- `just rest` - Start development REST server
- `just rest-db` - Start server with PostgreSQL backend

**Building:**
- `just build-py` - Build Python bindings
- `just build-node` - Build Node.js bindings
- `just build-docker` - Build Docker image
- `just build-sqlx` - Build SQLx queries for offline mode

**Testing:**
- `just test-node` - Run Node.js binding tests
- `just test-node-integration` - Run Node.js integration tests (starts a UC server)
- `just integration` - Run integration tests with mocked responses
- `just integration-record` - Record integration tests against a live server

**Code quality:**
- `just fix` - Auto-fix Rust and Node.js code issues
- `just fix-rust` - Auto-fix Rust code only
- `just fix-node` - Auto-fix Node.js code only
- `just fix-py` - Auto-fix Python code only
- `just lint-node` - Lint JavaScript/TypeScript code
- `just fmt` - Format all code (Rust, protobuf, etc.)

**Documentation:**
- `just docs` - Start the Astro documentation development server

Standard Rust commands also work:
- `cargo test` - Run Rust unit tests
- `cargo build` - Build Rust workspace
- `cargo clippy` - Run Rust linter

## Coding Style & Naming Conventions

**Rust**: Follow standard Rust conventions with 4-space indentation.
Use `cargo fmt` for formatting and `cargo clippy` for linting.
Requires Rust **1.85+** with Edition **2024** (configured in workspace `Cargo.toml`).

**JavaScript/TypeScript**: 2-space indentation, managed by Biome formatter/linter.

**Python**: 4-space indentation, 100-character line limit, enforced by Ruff.

**General**:
- Use descriptive names following each language's conventions
- Trim trailing whitespace and ensure final newline
- Proto definitions use snake_case, generated Rust uses PascalCase/snake_case appropriately

Pre-commit hooks enforce formatting with Biome, Ruff, and typos checking.

## Tooling

- **`uv`** - Python package manager; manages the `python/*` UV workspace
- **Maturin** - Builds Python wheels from PyO3 Rust crates
- **`buf`** - Protocol buffer compiler; config in `buf.yaml` / `buf.gen.yaml`
- **NAPI** - Node.js native addon bindings for Rust
- **Marimo** - Reactive Python notebook environment; notebooks live in `dev/`

## Testing Guidelines

**Framework**: Uses `rstest` for parameterized Rust tests, Jest for Node.js tests.

**Test Organization**:
- Unit tests: `#[cfg(test)]` modules in source files
- Integration tests: `tests/` directory and `crates/acceptance/`
- Test naming: `test_<functionality>` pattern

**Running Tests**:
- Rust: `cargo test` or `cargo test --workspace`
- Node.js: `just test-node`
- Integration: `just integration` (uses recorded responses)

**Coverage**: No specific coverage requirements, but comprehensive testing expected for new features.

## Commit & Pull Request Guidelines

GPG commit signing is required on this repository. The GPG PIN prompt requires an interactive terminal, which AI agents cannot provide. **Never attempt `git commit` directly** — it will time out and fail.

### Correct workflow

1. Run `cargo clippy --workspace --fix --allow-dirty` to apply automatic lint fixes (may change code)
2. Run `cargo fmt --all` to format all code (including anything clippy rewrote)
3. **Update documentation** — before staging, check and update any affected docs:
   - `CLAUDE.md` — update if commands, rules, or public APIs documented here have changed
4. Stage files with `git add <specific files>` — do this programmatically (via Bash tool)
5. Output a ready-to-paste `git commit` command for the user to run in their terminal

Steps 1 and 2 must always run in this order before staging — clippy may rewrite code that then needs formatting. Documentation updates (step 3) must happen before staging so all changes land in the same commit. Step 4 (staging) must be done by the agent so the user only needs to paste and run a single commit command.

### Commit message format

The agent writes the commit message to `/tmp/commit_msg.txt`, then:
1. Prints the full commit message in a code block so the user can read it
2. Provides the single-line command for the user to paste:

```bash
git commit -F /tmp/commit_msg.txt
```

**Do not use heredocs or `\n`-escaped strings** — on macOS/zsh, pasting multiline heredocs leaves the shell in an incomplete input state, and escaped newlines produce unreadable messages.

The `Co-authored-by: Isaac` trailer must be included on every commit.

### Commit message conventions

- **Types**: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`
- Subject line ≤ 72 characters, imperative mood ("add", not "added")
- Body explains *what* changed and *why*

**Code Generation**: Many files are auto-generated. Run `just generate` after proto changes and commit generated code separately when possible.