# Docker sandboxing

A repository of premade mixins for [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/): reusable building blocks (toolchains, tools, network egress, model providers) that compose with Docker's built-in agents into sandboxed VMs for coding agents.

## Before you start

Always pull the latest version of `main` before starting any work, and base your changes on it:

```bash
git fetch origin main
```

## Documentation

Use based on your task:

- [Worktrees](agent-guidance/worktrees.md) is the mandatory isolated workspace for every code change. Read before writing any code in this repo.
- [Versioning](agent-guidance/versioning.md) covers the release-please flow, version bumps, and adding a stack mixin. Read when preparing a release or touching pinned versions.
- [Commit Messages](agent-guidance/commit-messages.md) defines the Conventional Commits style, types, scopes, and examples. Read when writing commit messages.

## Code comments

- Every comment must be relevant to a developer landing on the code 6 months later: it should explain why the code is the way it is (constraints, non-obvious behavior, gotchas) — not narrate what the code obviously does, not reference the current task, ticket, or conversation, and not state something the reader can see or easily infer from the code itself.
- If a comment fails that test, omit it; when touching code with such comments, remove them. Temporary or conversational comments (e.g. "TODO: revisit after the migration", "this is a workaround for the bug I introduced above") do not belong in the repo.

## Artifacts

- mixins/* - Premade, reusable config fragments for sandboxes (kind: mixin). Must work in isolation and composition. Each mixin owns its install steps, env vars, and all the network domains its traffic needs (apt mirrors, revocation endpoints, download hosts).
  - Each mixin should have a `spec.yaml` and a `README.md` describing its purpose, usage, and any special instructions. Keep it simple.
  - Every software version a mixin installs or pins MUST be settable through a declared `args` entry (e.g. `version`) referenced as `${{ kit.args.version }}` in the spec, and MUST default to the latest version when the installer supplies no value — the latest upstream release, or (where the install resolves through a distro channel like apt or rustup's `stable`) the latest the channel offers. Never hardcode a software version in a command, URL, or package name — read it from an arg (or resolve latest dynamically when the arg is unset).
- examples/*.sbxenv.yaml - Ready-to-copy environment examples per stack, showing how to compose the mixins.

## Development

- Keep documentation simple and up to date. The docs are the main entry point for consumers of this project. The main audience is developers who has no knowledge of sbx.
- Keep mixins single-purpose and composable. They are the building blocks for sandboxes, and a new mixin is the primary contribution this repo accepts.