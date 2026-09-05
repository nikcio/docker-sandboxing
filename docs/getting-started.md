# Getting started

Use this repo's sandbox in three steps:

1. **Copy the example.** Copy `examples/opencode-node-dotnet.sbxenv.yaml` into a
   folder *next to* (not inside) your project and rename it `.sbxenv.yaml`:

   ```
   my-project-env/
   ├── .sbxenv.yaml          <- the example, renamed
   └── my-project/           <- your code (becomes the workspace)
   ```

2. **Adjust the config to your project.** In `.sbxenv.yaml`:
   - `name:` — a unique name for this sandbox (used by `sbx-env`, secrets, etc.)
   - `workspace.path:` — relative path to your project folder
   - `kits:` — drop the mixin lines your project doesn't need (see
     [mixins.md](mixins.md))

3. **Run it.**

   ```bash
   sbx env run
   ```

## Prerequisites

- [Docker Desktop](https://docs.docker.com/desktop/) with the `sbx` CLI
  installed and signed in (`sbx login`), version 0.39.0+
- A [Zeldoc.ai](https://zeldoc.ai) API key — see
  [zeldoc-api-key.md](zeldoc-api-key.md)
- Optional: a GitHub personal access token so the agent can push and open PRs
  — see [github-pat.md](github-pat.md). Public repos and git over SSH work
  without one.

The kits are fetched from `github.com/nikcio/docker-sandboxing` and the
template image is pulled from Docker Hub — no builds needed on your machine.

## First run

- The sandbox starts, prints a banner, and launches opencode automatically.
- Quit opencode and you drop into the sandbox's login shell (git, builds,
  `dotnet`, `pnpm`, …). Relaunch opencode anytime with `o`.
- Exiting that shell ends the sandbox session.

## Daily use

- Start the sandbox again with `sbx env run` from the folder containing the
  `.sbxenv.yaml`.
- Changed the `kits:` list? Kit changes only apply to **new** sandboxes —
  recreate with `sbx rm <name>` and `sbx env run` again.
- Remove the sandbox (and its scoped secrets) with `sbx env rm`.
