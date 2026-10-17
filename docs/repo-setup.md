# GitHub repository setup

One-time setup that makes this repo's automations work: CI on every PR and release-please release PRs. Do this once as a repo admin (Settings are admin-only).

## The workflows

| Workflow | Runs on | Needs |
| -------- | ------- | ----- |
| Validate (`validate.yml`) | every PR, pushes to `main`, manual | nothing — works out of the box |
| Release Please (`release-please.yml`) | pushes to `main` | a GitHub App (steps 1–2) |

Validate runs `sbx kit pack` on every kit and mixin (the offline equivalent of `sbx kit validate`). No secrets involved.

## 1. Create the release-please GitHub App

The release PR is created with a GitHub App token because the default `GITHUB_TOKEN` does not trigger workflow runs for anything it creates — a `GITHUB_TOKEN`-authored release PR could never collect the required checks.

1. GitHub → Settings → Developer settings → GitHub Apps → **New GitHub App**.
2. Fill in name and homepage URL; set the webhook URL to empty (no webhook needed).
3. Repository permissions:
   - **Contents**: Read and write (tags, releases)
   - **Pull requests**: Read and write (the release PR)
   - Metadata stays read-only (mandatory).
4. Under "Where can this app be installed?" keep **Only on this account**, then create the app.
5. Install it on `docker-sandboxing` — repository access: **Only select repositories** → this repo.
6. On the app's page: **General → About** — copy the **Client ID** (`APP_CLIENT_ID`), then **Private keys → Generate a private key** and keep the downloaded `.pem` (`APP_PRIVATE_KEY`).

## 2. Add the repository secrets

Settings → Secrets and variables → Actions → **New repository secret**:

| Secret | Value |
| ------ | ----- |
| `APP_CLIENT_ID` | the GitHub App's Client ID from step 1 |
| `APP_PRIVATE_KEY` | full contents of the `.pem` from step 1 |

## 3. Protect `main`

Recommended policies and why they matter when an autonomous agent works in this repo: [Branch policies for AI agents](agent-branch-protection.md).

Settings → Rules → Rulesets → **New branch ruleset** (or classic branch protection under Settings → Branches) for `main`:

- **Require a pull request before merging** — work happens in worktrees and lands via PRs (see [agent-guidance/worktrees.md](../agent-guidance/worktrees.md)).
- **Require status checks to pass**, then select:
  - `Kits & mixins (sbx kit pack)`

The checks appear after the first PR runs the Validate workflow. Validate runs on every PR (no path filtering) and always produces exactly this check, so the fixed required set is safe. Two caveats keep it that way: don't add path filters to `validate.yml` (a filtered-out PR never reports the required checks and can never merge).

The release PR is an ordinary PR: it runs the same checks and must pass before you merge it. Don't require checks from workflows that only run on pushes to `main` (release-please) — they never appear on PRs and would deadlock every merge.

## How a release flows

1. Conventional Commits (`feat`/`fix`/`deps`) land on `main`.
2. Release Please opens or updates a release PR (CHANGELOG entry and the `&ref=` pins in the examples).
3. You merge the release PR → tag `vX.Y.Z` + GitHub release.

Commit and versioning conventions: [agent-guidance/commit-messages.md](../agent-guidance/commit-messages.md), [agent-guidance/versioning.md](../agent-guidance/versioning.md).
