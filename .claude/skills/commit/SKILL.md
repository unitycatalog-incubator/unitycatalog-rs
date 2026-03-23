---
name: commit
description: This skill should be used when the user asks to "commit", "prepare a commit", "stage and commit", "commit my changes", or says the work is done and changes should be committed. Handles the full pre-commit pipeline (clippy → fmt → stage → commit message) for this GPG-signed repository where git commit cannot be run directly.
version: 0.1.0
---

# Commit Workflow

This repository requires GPG commit signing. The GPG PIN prompt needs an interactive
terminal — Claude cannot run `git commit` directly (it will time out). Instead, prepare
everything so the user can paste and run a single command.

## Workflow

### Step 1 — Auto-fix lint
```bash
cargo clippy --workspace --fix --allow-dirty
```
Clippy may rewrite code files. Always run this before formatting.

### Step 2 — Format all code
```bash
cargo fmt --all
```
Run after clippy, not before — clippy rewrites may need reformatting.

### Step 3 — Update documentation (if needed)
Before staging, check whether any of these need updating:
- `CLAUDE.md` — if commands, rules, or public APIs documented there have changed

### Step 4 — Stage specific files
```bash
git add <file1> <file2> ...
```
Stage only relevant files by name. Never use `git add -A` or `git add .` — this can
accidentally include generated files, `.env`, or large binaries.

### Step 5 — Write commit message to file
Derive the filename from the repo name and current branch to avoid collisions across
sessions or worktrees:

```bash
# Determine the path (run via Bash tool to get live values)
REPO=$(basename $(git rev-parse --show-toplevel))
BRANCH=$(git rev-parse --abbrev-ref HEAD | tr '/' '-')
MSG_FILE="/tmp/commit_msg_${REPO}_${BRANCH}.txt"
```

Write the commit message to that path using the Write tool (not echo/heredoc).

**Format:**
```
<type>: <subject ≤72 chars, imperative mood>

<body: what changed and why>

Co-authored-by: Isaac
```

**Types:** `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

The `Co-authored-by: Isaac` trailer is required on every commit.

### Step 6 — Print message and provide command
1. Print the full commit message in a code block so the user can review it
2. Provide this command for the user to paste (substituting the actual resolved path):

```bash
git commit -F /tmp/commit_msg_<repo>_<branch>.txt && rm /tmp/commit_msg_<repo>_<branch>.txt
```

The `&& rm` cleans up the temp file automatically after a successful commit.
If the commit fails (e.g. hook rejection), the file is preserved for inspection.

**Never use heredocs or `\n`-escaped strings** — on macOS/zsh, pasting multiline
heredocs leaves the shell in an incomplete input state.

## Notes

- Steps 1 and 2 must run in order — clippy first, then fmt
- Do not skip clippy even for small changes; it may catch issues
- If clippy or fmt changes files, include them in the staged set
- Generated files (from `just generate`) should be committed separately in a
  follow-up `chore: sync generated code` commit to keep review diffs readable
