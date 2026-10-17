# Docker sandboxing

A repository of premade mixins for [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/): reusable building blocks (toolchains, tools, network egress, model providers) that compose with Docker's built-in template image into sandboxed VMs for coding agents. The shipped examples use OpenCode, but the sandboxes work with any coding agent.

## Documentation

Use based on your task:

- **[Worktrees](agent-guidance/worktrees.md)** — the mandatory isolated workspace for every code change. Read before writing any code in this repo.
- **[Versioning](agent-guidance/versioning.md)** — release-please flow, version bumps, adding a stack mixin. Read when preparing a release or touching pinned versions.
- **[Commit Messages](agent-guidance/commit-messages.md)** — Conventional Commits style, types, scopes, examples. Read when writing commit messages.

## Artifacts

- kit/ - The base sandbox (kind: sandbox) running on Docker's built-in template image (docker/sandbox-templates:opencode-docker). Stack toolchains come from the mixins, not the image.
- mixins/* - Premade, reusable config fragments for sandboxes (kind: mixin). Must work in isolation and composition. List all required domains and software for usage on a base kit.
  - Each mixin should have a `spec.yaml` and a `README.md` describing its purpose, usage, and any special instructions. Keep it simple.
- examples/*.sbxenv.yaml - Ready-to-copy environment examples per stack, showing how to compose the mixins.

## Development

- Keep documentation simple and up to date. The docs are the main entry point for consumers of this project. The main audience is developers who has no knowledge of sbx.
- Keep mixins single-purpose and composable. They are the building blocks for sandboxes — a new mixin is the primary contribution this repo accepts.