# Repository Guidelines

## Project Structure & Module Organization

The project follows a multi-crate Rust workspace architecture with language bindings:

- `crates/` - Core Rust implementation organized by functionality:
  - `common/` - Shared types and utilities
  - `client/` - Unity Catalog client library
  - `server/` - REST API server implementation
  - `postgres/` - PostgreSQL backend integration
  - `cli/` - Command-line interface
  - `build/` - Code generation utilities (internal)
  - `derive/` - Custom derive macros (internal)
  - `acceptance/` - Acceptance testing utilities
- `python/client/` - Python bindings via PyO3
- `node/client/` - Node.js bindings via NAPI
- `proto/` - Protocol buffer definitions
- `openapi/` - Generated OpenAPI specifications

## Build, Test, and Development Commands

We use [`just`](https://just.systems/) as the primary task runner. Key commands:

- `just generate` - Run complete code generation pipeline
- `just generate-code` - Run internal code generation pipeline
- `just generate-proto` - Run code generation pipeline for Protocol Buffers
- `just rest` - Start development REST server
- `just rest-db` - Start server with PostgreSQL backend
- `just test-node` - Run Node.js binding tests
- `just integration` - Run integration tests with mocked responses
- `just build-py` - Build Python bindings
- `just build-node` - Build Node.js bindings
- `just lint-node` - Lint JavaScript/TypeScript code
- `just fix` - Auto-fix Rust and Node.js code issues
- `just fmt` - Format all code (Rust, protobuf, etc.)

Standard Rust commands also work:
- `cargo test` - Run Rust unit tests
- `cargo build` - Build Rust workspace
- `cargo clippy` - Run Rust linter

## Coding Style & Naming Conventions

**Rust**: Follow standard Rust conventions with 4-space indentation.
Use `cargo fmt` for formatting and `cargo clippy` for linting.

**JavaScript/TypeScript**: 2-space indentation, managed by Biome formatter/linter.

**Python**: 4-space indentation, 100-character line limit, enforced by Ruff.

**General**:
- Use descriptive names following each language's conventions
- Trim trailing whitespace and ensure final newline
- Proto definitions use snake_case, generated Rust uses PascalCase/snake_case appropriately

Pre-commit hooks enforce formatting with Biome, Ruff, and typos checking.

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

**Commit Messages**: Use conventional commit format:
- `feat:` - New features
- `fix:` - Bug fixes
- `refactor:` - Code restructuring
- `chore:` - Maintenance tasks
- `test:` - Test additions/modifications

**Code Generation**: Many files are auto-generated. Run `just generate` after proto changes and commit generated code separately when possible.