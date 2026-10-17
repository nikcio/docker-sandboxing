# Docker sandboxing

Docker Sandboxes kits for running OpenCode agents in sandboxed VMs, composed from Docker's built-in template images plus mixins.

## Documentation

Use based on your task:

- **[Worktrees](agent-guidance/worktrees.md)** — the mandatory isolated workspace for every code change. Read before writing any code in this repo.
- **[Versioning](agent-guidance/versioning.md)** — release-please flow, version bumps, adding a stack mixin. Read when preparing a release or touching pinned versions.
- **[Commit Messages](agent-guidance/commit-messages.md)** — Conventional Commits style, types, scopes, examples. Read when writing commit messages.

## Artifacts

- kit/ - The base sandbox (kind: sandbox) running on Docker's built-in template image (docker/sandbox-templates:opencode-docker). Stack toolchains come from the mixins, not the image.
- mixins/* - Reusable config fragments for sandboxes (kind: mixin). Must work in isolation and composition. List all required domains and software for usage on a base kit.
  - Each mixin should have a `spec.yaml` and a `README.md` describing its purpose, usage, and any special instructions. Keep it simple.

## Development

- Keep documentation simple and up to date. The docs are the main entry point for consumers of this project. The main audience is developers who has no knowledge of sbx.
- Keep mixins single-purpose and composable. They are the building blocks for sandboxes.