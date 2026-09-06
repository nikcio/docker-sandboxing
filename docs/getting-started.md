# Getting started

Use this repo's sandbox in three steps:

1. **Copy the example.** Copy the example matching your stack —
   `examples/opencode-node-dotnet.sbxenv.yaml` (.NET + Node),
   `examples/opencode-python.sbxenv.yaml` (Python + uv), or
   `examples/opencode-go.sbxenv.yaml` (Go), or
   `examples/opencode-rust.sbxenv.yaml` (Rust + cargo) — into your
   project's root and rename it `.sbxenv.yaml`. Commit it so teammates get
   the same sandbox.

2. **Adjust the config to your project.** In `.sbxenv.yaml`:
   - `name:` — a unique name for this sandbox (used to scope its secrets)
   - `kits:` — drop the mixin lines your project doesn't need, but keep
     `base` (the shared baseline: OpenCode config, agent guidance,
     entrypoint runtime — every kit requires it; see
     [mixins.md](mixins.md)); need project-specific settings (private
     feeds, env vars, agent notes)? see
     [project-kit.md](project-kit.md)
   - `workspace.path` is `.` (the repo itself); point it elsewhere only if
     the env file sits outside the project

3. **Run it** from your project root.

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

## Host settings

Two one-time `sbx` settings on your host:

- **Allow the kit source** (required — sbx only fetches kits from allowed
  sources). The setting replaces the whole list, so merge with your current
  entries — check them first with `sbx settings get kit.allowedSources`:

  ```bash
  sbx settings set kit.allowedSources '["docker.io/","github.com/nikcio/"]'
  ```

- **Optional:** let the sandboxed agent read images you paste:

  ```bash
  sbx settings set clipboard.imagePaste true
  ```

## First run

- The sandbox starts, prints a banner, and launches opencode automatically.
- Quit opencode and you drop into the sandbox's login shell (git, builds,
  `dotnet`, `pnpm`, …). Relaunch opencode anytime with `o`.
- Exiting that shell ends the sandbox session.

## Daily use

- Start the sandbox again with `sbx env run` from your project root.
- Want personal tweaks (more memory, debug env vars)? Keep them in a
  gitignored `local.sbxenv.yaml` — see [local-overrides.md](local-overrides.md).
- Changed the `kits:` list? Kit changes only apply to **new** sandboxes —
  recreate with `sbx rm <name>` and `sbx env run` again.
- Remove the sandbox (and its scoped secrets) with `sbx env rm`.
- Want your host's global agent skills available to the sandboxed agent? See
  [agent-skills.md](agent-skills.md).
