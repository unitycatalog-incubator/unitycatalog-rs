---
name: commit
description: This skill should be used when the user asks to "commit", "prepare a commit", "stage and commit", "commit my changes", or says the work is done and changes should be committed. Handles repos where git commit cannot be run directly by the agent (e.g. GPG signing, 2FA prompts) by preparing everything so the user pastes one command.
version: 0.1.0
---

# Commit Workflow

For repos where `git commit` cannot be run directly by the agent (e.g. GPG PIN
requires an interactive terminal), prepare everything so the user pastes one command.

## Workflow

### Step 1 — Auto-fix lint
Run the project's lint auto-fix command per CLAUDE.md
(e.g. `cargo clippy --workspace --fix --allow-dirty` for Rust).

Lint may rewrite code files. Always run this before formatting.

### Step 2 — Format all code
Run the project's formatter per CLAUDE.md (e.g. `cargo fmt --all` for Rust).

Always run after lint — lint rewrites may need reformatting.

### Step 3 — Update project documentation (if needed)
Before staging, check whether any project documentation files need updating
(e.g. CLAUDE.md, README) if commands, rules, or public APIs have changed.

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
2. Print the commit command in its own code block (substituting the actual resolved path):

```bash
git commit -F /tmp/commit_msg_<repo>_<branch>.txt && rm /tmp/commit_msg_<repo>_<branch>.txt
```

3. Tell the user: "Run `/copy` to copy the command to your clipboard, then paste and run it."

The `&& rm` cleans up the temp file automatically after a successful commit.
If the commit fails (e.g. hook rejection), the file is preserved for inspection.

**Never use heredocs or `\n`-escaped strings** — on macOS/zsh, pasting multiline
heredocs leaves the shell in an incomplete input state.

## Notes

- Steps 1 and 2 must run in order — lint first, then format
- Do not skip lint even for small changes; it may catch issues
- If lint or format changes files, include them in the staged set
- If the project uses code generation, commit generated files separately per
  CLAUDE.md conventions to keep review diffs readable
