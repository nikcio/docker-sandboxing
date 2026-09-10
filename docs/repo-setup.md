# GitHub repository setup

One-time setup that makes this repo's automations work: CI on every PR,
release-please release PRs, and image publishing to Docker Hub. Do this
once as a repo admin (Settings are admin-only).

## The workflows

| Workflow | Runs on | Needs |
| -------- | ------- | ----- |
| Validate (`validate.yml`) | every PR, pushes to `main`, manual | nothing — works out of the box |
| Release Please (`release-please.yml`) | pushes to `main` | a GitHub App (steps 1–2) |
| Publish images (`publish-image.yml`) | a release is published | Docker Hub secrets (step 2) |

Validate runs `sbx kit validate` on every kit and mixin plus a static
BuildKit check on every template Dockerfile. No secrets involved.

## 1. Create the release-please GitHub App

The release PR is created with a GitHub App token because the default
`GITHUB_TOKEN` does not trigger workflow runs for anything it creates —
a `GITHUB_TOKEN`-authored release PR could never collect the required
checks.

1. GitHub → Settings → Developer settings → GitHub Apps → **New GitHub
   App**.
2. Fill in name and homepage URL; set the webhook URL to empty (no
   webhook needed).
3. Repository permissions:
   - **Contents**: Read and write (tags, releases)
   - **Pull requests**: Read and write (the release PR)
   - Metadata stays read-only (mandatory).
4. Under "Where can this app be installed?" keep **Only on this account**,
   then create the app.
5. Install it on `docker-sandboxing` — repository access: **Only select
   repositories** → this repo.
6. On the app's page: **General → About** — copy the **Client ID**
   (`APP_CLIENT_ID`), then **Private keys → Generate a private key** and
   keep the downloaded `.pem` (`APP_PRIVATE_KEY`).

## 2. Add the repository secrets

Settings → Secrets and variables → Actions → **New repository secret**:

| Secret | Value |
| ------ | ----- |
| `APP_CLIENT_ID` | the GitHub App's Client ID from step 1 |
| `APP_PRIVATE_KEY` | full contents of the `.pem` from step 1 |
| `DOCKERHUB_USERNAME` | the Docker Hub account that owns `nikcio/<image>` |
| `DOCKERHUB_TOKEN` | a Docker Hub **access token** with Read & Write (Docker Hub → Account Settings → Security → New Access Token) — not the account password |

Publishing pushes every image in `images.json` as public
`docker.io/nikcio/<name>:vX.Y.Z` + `:latest` on Docker Hub.

## 3. Protect `main`

Recommended policies and why they matter when an autonomous agent works in
this repo: [Branch policies for AI agents](agent-branch-protection.md).

Settings → Branches → **Add branch ruleset** (or classic branch
protection) for `main`:

- **Require a pull request before merging** — work happens in worktrees
  and lands via PRs (see
  [agent-guidance/worktrees.md](../agent-guidance/worktrees.md)).
- **Require status checks to pass**, then select:
  - `Kits & mixins (sbx kit validate)`
  - `Dockerfile (buildx check) (template-node-dotnet)`
  - `Dockerfile (buildx check) (template-node)`
  - `Dockerfile (buildx check) (template-python)`
  - `Dockerfile (buildx check) (template-go)`
  - `Dockerfile (buildx check) (template-rust)`

The checks appear after the first PR runs the Validate workflow. Validate
runs on every PR (no path filtering) and always produces exactly these
six checks, so the fixed required set is safe.

The release PR is an ordinary PR: it runs the same checks and must pass
before you merge it.

## How a release flows

1. Conventional Commits (`feat`/`fix`/`deps`) land on `main`.
2. Release Please opens or updates a release PR (version bump, published
   kit tags, `&ref=` pins).
3. You merge the release PR → tag `vX.Y.Z` + GitHub release.
4. Publish images builds `images.json` and pushes to Docker Hub.

Commit and versioning conventions:
[agent-guidance/commit-messages.md](../agent-guidance/commit-messages.md),
[agent-guidance/versioning.md](../agent-guidance/versioning.md).
