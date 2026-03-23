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
- `examples/` - Type-checked code examples (Rust, Python, TypeScript) injected into docs

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
- `just validate-examples` - Type-check all examples and build docs (Rust/Python/TypeScript)

Standard Rust commands also work:
- `cargo test` - Run Rust unit tests
- `cargo build` - Build Rust workspace
- `cargo clippy` - Run Rust linter

## Code Examples in Documentation

Docs use a snippet injection system. The `examples/` directory contains type-checked
source files for Rust, Python, and TypeScript. In MDX docs, use `<CodeExample snippet="name" />`
to render a tabbed code block for all three languages automatically.

**Snippet tagging convention** (tag lines are stripped from the rendered output):
- Rust/TypeScript: `// [snippet:name]` ... `// [/snippet:name]`
- Python: `# [snippet:name]` ... `# [/snippet:name]`

Snippet names use `snake_case` matching the Rust method name (e.g., `list_catalogs`).

**Adding a new example:**
1. Add the tagged region to `examples/rust/src/*.rs`, `examples/python/*.py`, and `examples/typescript/examples/*.ts`
2. Add the new file imports to `docs/src/components/CodeExample.astro` and include them in `buildRegistry()`
3. Verify with `just validate-examples`
4. Use `<CodeExample snippet="your_snippet_name" />` in any MDX doc page (import the component first: `import CodeExample from "../../../components/CodeExample.astro"`)

**The `CodeExample` Astro component** (`docs/src/components/CodeExample.astro`) uses Vite `?raw`
imports to load snippet files at build time and expands `<CodeExample />` to Starlight
`<Tabs syncKey="language">` blocks — so users only need to select their language once site-wide.
It throws a hard build error if a snippet name is not found, ensuring docs always stay in sync.
Tabs are synced across the entire site via Starlight's built-in `localStorage` persistence.

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

GPG commit signing is required. **Never run `git commit` directly** — the GPG PIN prompt needs an interactive terminal and will time out.

Use the `/commit` skill (`.claude/skills/commit/SKILL.md`) for the full pre-commit workflow: clippy → fmt → stage → commit message file → paste command.

**Code Generation**: Many files are auto-generated. Run `just generate` after proto changes and commit generated code separately when possible.

### Pull Request workflow

1. **Create a feature branch** before starting any implementation — never work on `main`:
   ```bash
   git checkout -b feat/<short-description>
   ```

2. **Create follow-up issues** (via `gh issue create`) *before* opening the PR so they can be referenced in the PR body. Common follow-up patterns:
   - Migrations deferred to keep the PR focused
   - Related work in sibling crates not touched by this PR

3. **Open the PR** with `gh pr create` targeting `main`:
   - Title format: `<type>: <description> (#<issue>)` — reference the issue being closed
   - Body: bullet-point summary, test plan checklist, `Closes #N` line, follow-up issue references, and the `AI-assisted by Isaac` attribution line

4. **Commit generated code separately** when a feature PR regenerates many files (e.g. `just generate-code` produces 70+ diffs). Stage only the hand-written changes in the feature commit, then open a follow-up PR (`chore: sync generated code`) on a branch off the feature branch to commit the generated output. This keeps review diffs readable.

### GitHub Issues workflow

GitHub issues serve as a machine-readable execution schedule. Issue type, sub-issue
hierarchy, and `blocked-by` relationships together encode *what* to do, *how much* to
do per session, and *in what order* — so an agent pointed at an issue can autonomously
schedule and execute the work.

#### Issue type taxonomy

| Type | Scope | Agent execution model |
|------|-------|----------------------|
| **Epic** | Multi-session, multi-PR | Read direct sub-issues, build DAG, implement wave-by-wave with stacked PRs |
| **Feature** | Single session, one PR | Read Task sub-issues, build DAG, dispatch parallel sub-agents per wave |
| **Task** | Single agent, atomic | Read body, implement, verify, commit on parent Feature branch |
| **Bug** | Single agent, atomic | Same as Task; may appear under Epic, Feature, or standalone |

Tasks/Bugs may appear directly under an Epic (not inside a Feature) when they represent
standalone prerequisite work — e.g. a dependency upgrade that unblocks multiple Features.

#### Agent execution protocol

**Given a Task or Bug:** implement the work in the body, run the verification steps, commit.

**Given a Feature:**
1. Fetch all Task sub-issues + their `blocked-by` links
2. Group into waves: wave 1 = no blockers, wave N = blocked only by wave N-1
3. Dispatch each wave as parallel sub-agents; wait before starting the next wave
4. Open one PR for the Feature branch

**Given an Epic:**
1. Fetch all direct sub-issues (Features, Tasks, Bugs) + their `blocked-by` links
2. Build a DAG across all direct children using the same wave logic
3. Features trigger the Feature protocol above; Tasks/Bugs are atomic
4. Sequentially dependent items get stacked branches; PRs merged in dependency order

#### Issue body conventions

**Feature body:**
```markdown
## Context
## Acceptance criteria
- [ ] <testable outcome>
## Key files
- `path/to/file` — purpose
```

**Task/Bug body:**
```markdown
## Context
## Work
<numbered steps>
## Verification
<commands to run, tests to pass>
## Key files
- `path/to/file` — purpose
```

Never encode blocked-by or sub-issue relationships as text in the body — use the GitHub APIs below.

#### GitHub API operations

All relationship operations require node IDs:
```bash
gh api repos/OWNER/REPO/issues/NUMBER --jq '.node_id'
```

**Set issue type** (on create or update):
```bash
# Get type IDs for this repo
gh api graphql -f query='{
  repository(owner: "OWNER", name: "REPO") {
    issueTypes(first: 10) { nodes { id name } }
  }
}'

# Set type
gh api graphql -f query='
  mutation($issueId: ID!, $issueTypeId: ID!) {
    updateIssue(input: {id: $issueId, issueTypeId: $issueTypeId}) {
      issue { number issueType { name } }
    }
  }' -f issueId="NODE_ID" -f issueTypeId="TYPE_ID"
```

**Add sub-issue** (`--parent` flag does not exist in the CLI):
```bash
gh api graphql -f query='
  mutation($parentId: ID!, $issueId: ID!) {
    addSubIssue(input: {issueId: $parentId, subIssueId: $issueId}) {
      issue { number } subIssue { number }
    }
  }' -f parentId="PARENT_NODE_ID" -f issueId="CHILD_NODE_ID"
```

**Add blocked-by relationship:**
```bash
gh api graphql -f query='
  mutation($issueId: ID!, $blockingIssueId: ID!) {
    addBlockedBy(input: {issueId: $issueId, blockingIssueId: $blockingIssueId}) {
      clientMutationId
    }
  }' -f issueId="BLOCKED_NODE_ID" -f blockingIssueId="BLOCKER_NODE_ID"
```

Sub-issue hierarchies are sufficient for planning. Do not use Milestones. GitHub Projects
will be adopted later for richer planning views.