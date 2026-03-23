---
name: plan-issue
description: Use this skill when the user asks to "dry run issue #N", "show me the execution plan for #N", "plan issue #N", or wants to preview what run-issue would do without executing anything. Fetches the full issue hierarchy, resolves the DAG, and emits a human-readable execution narrative showing every protocol step, consolidation gate, and decision point. Read-only — makes no GitHub mutations.
version: 0.1.0
---

# Plan Issue (Dry Run)

Simulate the full `run-issue` execution plan for a GitHub issue without doing any work.
Output a human-readable narrative showing every step the agent would take, tagged by
source (`[issue]` / `[protocol]` / `[decision]`), followed by a YAML machine-readable
block for future tooling.

**This skill is read-only. You MUST NOT:**
- Create or check out branches
- Write or edit any files
- Execute sub-agents
- Create, edit, or close GitHub issues
- Make any GitHub mutations (GraphQL or REST writes)

## Step 1 — Parse the issue number

Extract the issue number from the user's input (e.g. `/plan-issue 85` → `85`).
Determine the repo from the current git remote:
```bash
gh repo view --json owner,name --jq '"owner: \(.owner.login), name: \(.name)"'
```

## Step 2 — Fetch the issue hierarchy (read-only)

Get the root issue type, title, and state:
```bash
gh api repos/OWNER/REPO/issues/NUMBER --jq '{number, title, state}'
gh api graphql -f query='{
  repository(owner: "OWNER", name: "REPO") {
    issue(number: NUMBER) { issueType { name } }
  }
}'
```

Fetch sub-issues (depth 1):
```bash
gh api repos/OWNER/REPO/issues/NUMBER/sub_issues \
  --jq '[.[] | {number, title, state}]'
```

For each sub-issue, fetch its own sub-issues (depth 2 — Tasks under Features):
```bash
gh api repos/OWNER/REPO/issues/SUB_NUMBER/sub_issues \
  --jq '[.[] | {number, title, state}]'
```

For each issue at every depth, fetch blocked-by relationships using the `blockedBy` field:
```bash
gh api graphql -f query='{
  repository(owner: "OWNER", name: "REPO") {
    issue(number: NUMBER) {
      blockedBy(first: 10) { nodes { number title } }
    }
  }
}'
```

> **Note:** `trackedInIssues` and `trackedByIssues` are different fields and do NOT
> return blocked-by relationships. The correct field is `blockedBy`. If you are unsure
> which fields are available, introspect the schema first:
> ```bash
> gh api graphql -f query='{
>   __type(name: "Issue") { fields { name } }
> }' --jq '[.data.__type.fields[].name] | sort | .[] | select(test("block|track|depend"; "i"))'
> ```

## Step 3 — Resolve the DAG into waves

For each parent scope (Epic, Feature), build the wave ordering:
- **Wave 1**: issues with no `blocked_by` within the same parent scope
- **Wave N**: issues whose all blockers appear in waves < N
- Issues blocked by items in *other* scopes (e.g. Feature blocked by another Feature) count as cross-scope dependencies — handled at the parent level, not the inner wave level

## Step 4 — Emit the execution narrative

Output in this exact format:

### Part 1 — Summary header

```
═══════════════════════════════════════════════════════
<Type> #N — <title>
repo: OWNER/REPO
<X> epic-waves · <Y> Features · <Z> Tasks · peak parallelism: <P>
═══════════════════════════════════════════════════════
```

### Part 2 — Full execution narrative

Use this step-tagging convention consistently:
- `[issue]` — step derived from the issue body (Work steps, Verification)
- `[protocol]` — step the agent inserts from its own instructions (gates, sync, branching)
- `[decision]` — branch point where the agent may take different paths at runtime

Number steps hierarchically (1, 2, 2.1, 2.1.1, etc.) so reviewers can reference them.

For each **Task** expanded in the narrative, include:
- The dispatch step (`[protocol]`)
- Each Work step from the issue body (`[issue]`)
- Each Verification step from the issue body (`[issue]`)
- GitHub sync: completion comment + close (`[protocol]`)
- `/commit` skill invocation (`[protocol]`)

For each **wave consolidation gate**, always include these decision points:
- Did all tasks in the wave complete? → retry or escalate
- Discovered blocking work? → create issue, insert into DAG
- Discovered follow-up work? → create issue as sub, note for PR
- **Parallel wave only**: Did multiple tasks in this wave write to the same file?
  → flag as a merge conflict risk; note which tasks and which files conflict

For each **Feature**, include the PR-open step at the end:
- Create follow-up issues before opening PR (`[protocol]`)
- `gh pr create` with title format (`[protocol]`)

For **Epics**, include epic-level consolidation gates between waves.

Elide deeply repeated inner patterns after the first full example with `... [same pattern]`
but always show the first Feature in full detail so the reviewer can verify the template.

### Part 3 — YAML machine-readable block

Append this after the narrative, inside a fenced code block labeled `yaml`:

```yaml
# /plan-issue output — machine-readable
issue: NUMBER
title: "..."
type: Epic|Feature|Task|Bug
repo: OWNER/REPO
waves:
  - wave: 1
    parallel: true|false
    items:
      - number: N
        title: "..."
        type: Feature|Task|Bug
        state: open|closed
        blocked_by: []
        # For Features, include inner_waves:
        inner_waves:
          - wave: 1
            parallel: true|false
            items:
              - {number: N, type: Task, state: open, blocked_by: []}
```

## Step 5 — Stop

After emitting the narrative and YAML, stop. Do not proceed to implement anything.
Remind the user of the next steps:

```
──────────────────────────────────────────────────────
Next steps:
  /refine-issue NUMBER   — analyze and improve the issue specs
  run-issue NUMBER       — execute the plan
──────────────────────────────────────────────────────
```
