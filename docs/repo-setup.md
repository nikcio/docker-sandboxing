# GitHub repository setup

One-time setup that makes this repo's automations work: CI on every PR and release-please release PRs. Do this once as a repo admin (Settings are admin-only).

## The workflows

| Workflow | Runs on | Needs |
| -------- | ------- | ----- |
| Validate (`validate.yml`) | every PR, pushes to `main`, manual | nothing, works out of the box |
| Release Please (`release-please.yml`) | pushes to `main` | a GitHub App (steps 1 and 2) |
| Preview Release (`preview-release.yml`) | pushes to `main`, manual | the same GitHub App |

Validate runs `sbx kit pack` on every mixin (the offline equivalent of `sbx kit validate`). No secrets involved.

## 1. Create the release-please GitHub App

The release PR is created with a GitHub App token because the default `GITHUB_TOKEN` does not trigger workflow runs for anything it creates. A `GITHUB_TOKEN`-authored release PR could never collect the required checks.

1. GitHub → Settings → Developer settings → GitHub Apps → **New GitHub App**.
2. Fill in name and homepage URL; set the webhook URL to empty (no webhook needed).
3. Repository permissions:
   - **Contents.** Read and write (tags, releases)
   - **Pull requests.** Read and write (the release PR)
   - Metadata stays read-only (mandatory).
4. Under "Where can this app be installed?" keep **Only on this account**, then create the app.
5. Install it on `docker-sandboxing` with repository access **Only select repositories**, then pick this repo.
6. On the app's page, under **General → About**, copy the **Client ID** (`APP_CLIENT_ID`), then under **Private keys → Generate a private key** keep the downloaded `.pem` (`APP_PRIVATE_KEY`).

## 2. Add the repository secrets

Settings → Secrets and variables → Actions → **New repository secret**:

| Secret | Value |
| ------ | ----- |
| `APP_CLIENT_ID` | the GitHub App's Client ID from step 1 |
| `APP_PRIVATE_KEY` | full contents of the `.pem` from step 1 |

## 3. Protect `main`

Recommended policies and why they matter when an autonomous agent works in this repo: [Branch policies for AI agents](agent-branch-protection.md).

Settings → Rules → Rulesets → **New branch ruleset** (or classic branch protection under Settings → Branches) for `main`:

- Require a pull request before merging. Work happens in worktrees and lands via PRs (see [agent-guidance/worktrees.md](../agent-guidance/worktrees.md)).
- **Require status checks to pass**, then select:
  - `Mixins (sbx kit pack)`

The checks appear after the first PR runs the Validate workflow. Validate runs on every PR (no path filtering) and always produces exactly this check, so the fixed required set is safe. Two caveats keep it that way: don't add path filters to `validate.yml` (a filtered-out PR never reports the required checks and can never merge).

The release PR is an ordinary PR: it runs the same checks and must pass before you merge it. Don't require checks from workflows that only run on pushes to `main` (release-please), because they never appear on PRs and would deadlock every merge.

## How a release flows

1. Conventional Commits (`feat`/`fix`/`deps`) land on `main`.
2. Release Please opens or updates a release PR (CHANGELOG entry and the `&ref=` pins in the examples).
3. You merge the release PR, which tags `vX.Y.Z` and publishes the GitHub release.

## Preview releases

Run the Preview Release workflow (Actions → Preview Release → Run workflow) to
cut a prerelease (`v3.1.0-beta001`) from current `main`. It opens and queues its
own release PR for auto-merge, then tags and publishes the Sandy builds as a
GitHub prerelease. Previews never run on their own — only manual dispatches
open preview release PRs. No extra setup beyond the stable flow — same GitHub
App, plus **Allow auto-merge** enabled in Settings → General → Pull Requests
(the workflow queues its release PR for auto-merge and waits for the required
checks). Details: [agent-guidance/versioning.md](../agent-guidance/versioning.md).

Commit and versioning conventions: [agent-guidance/commit-messages.md](../agent-guidance/commit-messages.md), [agent-guidance/versioning.md](../agent-guidance/versioning.md).
