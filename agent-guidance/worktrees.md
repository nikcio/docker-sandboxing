# Worktrees

Multiple agent sessions work in this repo at the same time (parallel stack
additions, fixes, reviews). The main checkout belongs to no one: it can
switch branches under you, or carry another session's uncommitted work.

## Rule

Every code change happens in a dedicated git worktree branched from
`origin/main` — never in the main checkout, and never on top of whatever
branch it happens to have checked out.

```bash
git worktree add /tmp/opencode/<topic> -b <type>/<topic> origin/main
cd /tmp/opencode/<topic>
# work, validate, commit ...
git push -u origin <type>/<topic>
gh pr create ...
git worktree remove /tmp/opencode/<topic>   # after the PR is up
```

## Hazards

- **Swept files.** `git add .` in a shared checkout sweeps other sessions'
  untracked files into your commit. Stage paths explicitly, and remove your
  untracked files from a shared checkout as soon as they are committed on
  your branch.
- **Stale reads.** Re-read shared files (`images.json`,
  `.release-please-config.json`, `README.md`, `AGENTS.md`, `docs/`) in your
  worktree before editing them — an in-flight PR may have changed them.
  Keep shared-file hunks additive so parallel PRs merge cleanly.
- **Wrong tree.** `git status` in the main checkout says nothing about your
  worktree. Run every git command from inside your worktree (or with
  `git -C`).
