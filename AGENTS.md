# Docker sandboxing

Docker Sandboxes kits and template images for running OpenCode agents in sandboxed VMs.

## Documentation

Use based on your task:

- **[Worktrees](agent-guidance/worktrees.md)** — the mandatory isolated workspace for every code change. Read before writing any code in this repo.
- **[Versioning](agent-guidance/versioning.md)** — release-please flow, version bumps, image publishing, adding an image. Read when preparing a release or touching pinned versions.
- **[Commit Messages](agent-guidance/commit-messages.md)** — Conventional Commits style, types, scopes, examples. Read when writing commit messages.

## Artifacts

- kit-* - Workload kits (kind: workload, v3 descriptor + Dockerfile) building the stack toolchain on the shell base image
- mixins/* - Reusable config fragments for sandboxes (kind: mixin). Must work in isolation and composition. List all required domains and software for usage on a base kit.
  - Each mixin is a v3 descriptor (`mixins/<area>/<area>.yaml`, optional `<area>.dockerfile`) with a `README.md` describing its purpose, usage, and any special instructions. Keep it simple.

## Development

- Keep documentation simple and up to date. The docs are the main entry point for consumers of this project. The main audience is developers who has no knowledge of sbx.
- Keep mixins single-purpose and composable. They are the building blocks for sandboxes.