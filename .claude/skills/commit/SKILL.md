---
name: commit
description: This skill should be used when the user asks to "commit", "prepare a commit", "stage and commit", "commit my changes", or says the work is done and changes should be committed. Commits unsigned (no interactive GPG PIN) and defers signing to a single bulk step before opening a PR.
version: 0.2.0
---

# Commit Workflow

The commit-message contract (types, `AI-assisted-by: Isaac` trailer, granularity)
lives in `~/.claude/CLAUDE.md` — follow it; this skill covers the mechanics.

Commits here are GPG-signed, and the PIN needs an interactive terminal. The agent
**commits unsigned** (`git commit --no-gpg-sign`, which needs no PIN), and the
user **signs once before pushing / opening a PR**. No per-commit paste.

## Workflow

### Step 1 — Auto-fix lint
`cargo clippy --workspace --fix --allow-dirty` (and `just fix-node` / `just fix-py`
if those files changed). Lint may rewrite code; always run before formatting.

### Step 2 — Format all code
`cargo fmt --all` (plus the relevant language formatter if you touched node/py).
Always after lint — lint rewrites may need reformatting.

### Step 3 — Stage specific files (and split commits)
Stage only relevant files by name — never `git add -A` / `git add .` (avoids
generated files, `.env`, large binaries).

```bash
git add <file1> <file2> ...
```

When the working tree spans **multiple logical changes**, make **multiple small,
well-scoped commits** (one type/scope each) rather than one mixed commit — signing
is a single bulk step per branch, so small commits cost nothing and give
release-plz a richer per-crate history. Don't over-fragment. Commit generated
output in the **same commit** as the change that produced it (see CLAUDE.md
codegen rules).

### Step 4 — Write the message, then commit unsigned
Derive a temp filename from repo + branch to avoid collisions across worktrees:

```bash
REPO=$(basename $(git rev-parse --show-toplevel))
BRANCH=$(git rev-parse --abbrev-ref HEAD | tr '/' '-')
MSG_FILE="/tmp/commit_msg_${REPO}_${BRANCH}.txt"
```

Write the message to that path with the **Write tool** (not echo/heredoc — on
macOS/zsh a pasted multiline heredoc leaves the shell in incomplete-input state).
Format per `~/.claude/CLAUDE.md` (`<type>: <subject>` / body / `AI-assisted-by:
Isaac`). Then commit directly — no PIN needed:

```bash
git commit --no-gpg-sign -F "$MSG_FILE" && rm "$MSG_FILE"
```

**Pre-commit hook:** this repo has an active `.pre-commit-config.yaml` (Biome,
typos, Ruff, cargo fmt + cargo-check) that runs on every commit, signed or not,
and can rewrite files or abort. If it rewrites files, re-stage them and retry the
commit once. If it still fails, surface the hook output and stop — don't loop.
The `&& rm` only runs on success, so a failed commit preserves the message file.

### Step 5 — Push and open the PR (don't wait on signing)
Commit unsigned, then **push and open the PR in the same pass** — don't stop to
wait for the GPG PIN mid-flow. Tell the user the commits are unsigned and will be
signed in one bulk step at the end. Don't offer a re-sign after each commit.

## Signing — one bulk step at the end (after the PR is open)

Signing rewrites the commits (amend), so the already-pushed branch needs a
`--force-with-lease` re-push. That's safe on a solo feature branch and is
preferred over splitting work across handoffs. Surface ONE combined command for
the user to run (one GPG PIN); signatures aren't required to merge, so this can
happen any time before merge:

- One commit (HEAD):
  ```bash
  git commit --amend --no-edit -S && git push --force-with-lease
  ```
- Range (the normal case):
  ```bash
  git rebase --exec 'git commit --amend --no-edit -S' "$(git merge-base main HEAD)" && git push --force-with-lease
  ```
- Verify: `git log --format='%h %G? %s' main..HEAD` — every commit shows `G`.

## Notes

- Steps 1 and 2 run in order — lint first, then format.
- Don't skip lint even for small changes.
- If lint or format changes files, include them in the staged set.
