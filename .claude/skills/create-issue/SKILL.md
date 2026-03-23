---
name: create-issue
description: This skill should be used when the user asks to "create issues for X", "plan this as GitHub issues", "set up issues for a feature", "break this into tasks", or wants to structure work using the GitHub issue taxonomy. Provides the issue type hierarchy, body templates, and GraphQL API operations for this repository.
version: 0.1.0
---

# Create Issue

Structures work as GitHub issues using the Epic → Feature → Task/Bug hierarchy.
Issues serve as a machine-readable execution schedule: type, sub-issue hierarchy,
and `blocked-by` relationships encode *what* to do, *how much* per session, and
*in what order*.

## Issue Type Taxonomy

| Type | Scope | Execution model |
|------|-------|----------------|
| **Epic** | Multi-session, multi-PR | Build DAG across direct children; implement wave-by-wave with stacked PRs |
| **Feature** | Single session, one PR | Fetch Task sub-issues, build DAG, dispatch parallel sub-agents per wave |
| **Task** | Single agent, atomic | Read body, implement, verify, commit on parent Feature branch |
| **Bug** | Single agent, atomic | Same as Task; may appear under Epic, Feature, or standalone |

Tasks/Bugs may appear directly under an Epic (not inside a Feature) for standalone
prerequisite work (e.g. a dependency upgrade that unblocks multiple Features).

## Issue Body Templates

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

Never encode blocked-by or sub-issue relationships as text in the body — use the
GitHub API operations below.

## Creating Issues

Use `gh issue create` with `--title` and `--body`. Then set the issue type and
relationships via GraphQL (see below).

**Create follow-up issues *before* opening a PR** so they can be referenced in the
PR body.

## GitHub API Operations

All relationship operations require node IDs:
```bash
gh api repos/OWNER/REPO/issues/NUMBER --jq '.node_id'
```

**Get issue type IDs for this repo:**
```bash
gh api graphql -f query='{
  repository(owner: "OWNER", name: "REPO") {
    issueTypes(first: 10) { nodes { id name } }
  }
}'
```

**Set issue type:**
```bash
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

## Notes

- Sub-issue hierarchies are sufficient for planning. Do not use Milestones.
- GitHub Projects will be adopted later for richer planning views.
- The `run-issue` agent handles *executing* issues once they are created.
