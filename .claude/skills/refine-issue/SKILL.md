---
name: refine-issue
description: Use this skill when the user asks to "refine issue #N", "improve the plan for #N", "review the issues for #N", or wants to strengthen issue specs before execution. Analyzes the full issue hierarchy for quality gaps (missing verification, vague work steps, missing key files, insufficient test coverage, wrong blocking structure) and performs Feature-level TDD decomposition assessment. Proposes all changes for user approval before writing anything to GitHub.
version: 0.1.0
---

# Refine Issue

Analyze the full issue hierarchy for a GitHub issue, identify quality gaps, and propose
improvements — including optional TDD task decomposition for complex Features. All
proposals require user approval before any GitHub mutations are made.

## Step 1 — Fetch the full hierarchy

Use the same read-only fetch logic as `/plan-issue`:
- Root issue: type, title, state, body
- All sub-issues recursively (depth ≤ 3)
- `blocked_by` links for every issue
- Full body of every Task and Bug (Context, Work, Verification, Key files sections)

```bash
# Root issue body
gh api repos/OWNER/REPO/issues/NUMBER --jq '{number, title, state, body}'

# Sub-issues
gh api repos/OWNER/REPO/issues/NUMBER/sub_issues \
  --jq '[.[] | {number, title, state}]'

# Issue type
gh api graphql -f query='{
  repository(owner: "OWNER", name: "REPO") {
    issue(number: NUMBER) { issueType { name } body }
  }
}'
```

## Step 2 — Task/Bug quality analysis

For every Task and Bug in the hierarchy, evaluate against these checks.
Flag issues that fail any check — do not silently skip.

### 2a. Verification section

| Criterion | Pass | Flag |
|-----------|------|------|
| Section exists | "## Verification" present | Missing entirely |
| Commands are specific | Commands that produce pass/fail output | Vague ("check it works") |
| All Work outputs are tested | Every created/edited artifact has a check | Orphan outputs |

### 2b. Work steps

| Criterion | Pass | Flag |
|-----------|------|------|
| Steps are numbered | 1. 2. 3. format | Prose or bullets only |
| Each step is atomic | One action per step | "do X and Y and Z" |
| File paths are explicit | `crates/server/src/api/foo.rs` | "the server file" |
| No guessing required | Agent can act without researching what to do | Vague verbs: "update", "handle", "make sure" |

### 2c. Key files

| Criterion | Pass | Flag |
|-----------|------|------|
| All files referenced in Work appear in Key files | Complete coverage | Missing entries |
| Purpose column is present | Each file has a `— purpose` annotation | Bare paths |
| Likely missing files | Known related files not listed | Infer from Work steps |

### 2d. Test coverage

| Criterion | Pass | Flag |
|-----------|------|------|
| Tests run in Verification | `cargo test`, `just test-*`, or equivalent | No test command |
| New behavior has new tests | If Work adds a function/handler, a test is written | Pure implementation with no test step |
| Test location specified | Which test file or module | "write tests" without location |

### 2e. Blocking structure

| Criterion | Pass | Flag |
|-----------|------|------|
| Correct blockers | Imports/depends on output of another task → blocked | Missing `blocked_by` |
| No over-blocking | Only genuinely dependent tasks are blocked | Unnecessary sequential constraints |
| Parallel shared-file conflict | Parallel tasks (no blocked-by between them) each touch distinct files | Two parallel tasks both edit the same file (e.g. `lib.rs`, `mod.rs`, `router.rs`) |

**Shared-file conflict resolution:** When two parallel tasks both modify the same file,
propose one of:
- Add a `blocked_by` between them (serialize), OR
- Extract the shared edit into a new prerequisite task that both depend on

Common shared files to watch for: `mod.rs`, `lib.rs`, any router/registry file that
all new resources register into.

## Step 3 — Feature-level complexity assessment

For each Feature, decide whether TDD decomposition is warranted.

**Decompose when the Feature:**
- Introduces non-trivial business logic (validation, transformation, state management)
- Defines a new API contract (handler trait, new HTTP endpoint behavior)
- Integrates with an external system (store, cloud provider, protocol)
- Has more than 3 Tasks already, suggesting scope complexity

**Do NOT decompose when the Feature:**
- Is purely mechanical/generative (proto authoring, code-gen registration, config changes)
- Has Tasks that are already fine-grained and independently verifiable
- Would produce trivially small skeleton and review tasks

**When decomposing, propose adding three ordered tasks under the Feature:**

1. **Skeleton + failing tests** task:
   - Define interfaces, types, and trait stubs
   - Write test stubs / integration test scaffolding that will fail
   - Verify: `cargo test` compiles but tests fail with "not implemented" or similar
   - Blocked by: whatever the original first task was blocked by

2. **Implementation** task:
   - Fill in the implementation to make tests pass
   - Verify: `cargo test` passes, `cargo clippy` clean
   - Blocked by: the skeleton task

3. **Review gate** task:
   - Acceptance criteria from the Feature's body all checked off
   - Docs/comments updated if public API changed
   - `just validate-examples` passes if examples affected
   - Blocked by: the implementation task

**The original Tasks** become sub-steps within these three, or are redistributed.
When proposing decomposition, show explicitly how existing tasks map into the new structure.

## Step 4 — Build the proposal

Collect all findings and present them as a structured proposal. Group by issue number.
Show **old → new** for edits; show **full body** for new issues.

Format:

```
══ REFINEMENT PROPOSAL ════════════════════════════════

ISSUE #N — <title>
  ⚠ Work step 3: vague ("update the handler") — proposed rewrite:
    OLD: 3. Update the handler to process function requests
    NEW: 3. In `crates/server/src/api/functions.rs`, add `create_function` method
            to the `impl<T: ResourceStore + Policy> FunctionHandler for T` block.
            Follow the pattern in `crates/server/src/api/catalogs.rs:14-43`.

  ⚠ Verification: no test command — proposed addition:
    ADD TO VERIFICATION:
    - `cargo test -p unitycatalog-server` — all tests pass

  ⚠ Key files: missing reference file — proposed addition:
    ADD TO KEY FILES:
    - `crates/server/src/api/catalogs.rs` — reference implementation pattern

FEATURE #M — <title>
  ℹ Complexity assessment: NEW API CONTRACT → TDD decomposition recommended
    Proposed new tasks (to be created as sub-issues of #M):
    - Task A: Skeleton + failing tests for FunctionHandler
    - Task B: Implementation — make tests pass
    - Task C: Review gate — acceptance criteria, docs, clippy
    Existing tasks #92, #93 redistributed as:
    - #92 content → Task A (interface definition) + Task B (implementation)
    - #93 content → Task B step 4 (router registration)

══ SUMMARY ════════════════════════════════════════════
  X issues need body edits
  Y new tasks proposed (TDD decomposition)
  Z missing blocked-by links

Apply all proposals? [Review each individually / Apply all / Skip]
```

**Always present the full proposal before making any changes.**
**Ask for explicit approval before proceeding to Step 5.**

## Step 5 — Apply approved changes

Apply only the changes the user approves. For each:

**Issue body edit:**
```bash
# Write updated body to temp file
cat > /tmp/issue_body_NUMBER.md << 'EOF'
<updated body>
EOF
gh issue edit NUMBER --body-file /tmp/issue_body_NUMBER.md
```

**New task creation:**
```bash
gh issue create --title "..." --body-file /tmp/issue_body_NEW.md
# Then set type:
gh api graphql -f query='
  mutation($issueId: ID!, $issueTypeId: ID!) {
    updateIssue(input: {id: $issueId, issueTypeId: $issueTypeId}) {
      issue { number }
    }
  }' -f issueId="NODE_ID" -f issueTypeId="TASK_TYPE_ID"
# Add as sub-issue:
gh api graphql -f query='
  mutation($parentId: ID!, $issueId: ID!) {
    addSubIssue(input: {issueId: $parentId, subIssueId: $issueId}) {
      issue { number }
    }
  }' -f parentId="PARENT_NODE_ID" -f issueId="NEW_NODE_ID"
```

**New blocked-by link:**
```bash
gh api graphql -f query='
  mutation($issueId: ID!, $blockingIssueId: ID!) {
    addBlockedBy(input: {issueId: $issueId, blockingIssueId: $blockingIssueId}) {
      clientMutationId
    }
  }' -f issueId="BLOCKED_NODE_ID" -f blockingIssueId="BLOCKER_NODE_ID"
```

## Step 6 — Post refinement summary

After all approved changes are applied, add a comment to the root issue:
```bash
gh issue comment NUMBER --body "Refinement applied via /refine-issue.

Changes made:
- <list each edit/creation/link>

Ready for execution: \`run-issue NUMBER\`

AI-assisted by Isaac"
```

Then remind the user:
```
──────────────────────────────────────────────────────
Refinement complete. Suggested next steps:
  /plan-issue NUMBER   — re-run dry run to verify the improved plan
  run-issue NUMBER     — execute
──────────────────────────────────────────────────────
```
