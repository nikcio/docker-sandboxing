# Getting started

Run [OpenCode](https://opencode.ai) in a sandboxed VM inside your own repo in
three steps: **install `sbx`**, **copy the example for your stack**, and
**run one command**. OpenCode starts in a VM with your toolchain preinstalled.

The sandbox is the isolation boundary: outbound network is deny-by-default and
API keys are injected by a proxy — the sandbox only ever sees placeholders.

## 1. Install `sbx`

Install the [`sbx` CLI](https://docs.docker.com/ai/sandboxes/install/) for
your platform (Docker Desktop is **not** required — `sbx` runs its own VMs):

| Platform | Command |
| -------- | ------- |
| macOS | `brew install docker/tap/sbx` |
| Windows | `winget install -h Docker.sbx` |
| Ubuntu 24.04+ | `curl -fsSL https://get.docker.com \| sudo SBX=1 sh` |

<details>
<summary>System requirements</summary>

- **macOS:** Sonoma 14 or later, Apple silicon
- **Windows:** Windows 11, 64-bit Intel/AMD, Windows Hypervisor Platform
- **Linux:** Ubuntu 24.04+, 64-bit CPU with KVM enabled, your user in the
  `kvm` group

See Docker's [installation guide](https://docs.docker.com/ai/sandboxes/install/)
for the full requirements and manual-install options. Version 0.39.0 or
newer is recommended.
</details>

## 2. Sign in

```bash
sbx login
```

The command opens a browser for Docker OAuth.

## 3. Copy the example for your stack

Copy the example matching your stack into your project's root and rename it
`.sbxenv.yaml`. Commit it so teammates get the same sandbox.

| Your stack | Copy this example |
| ---------- | ----------------- |
| .NET + Node.js | [`examples/opencode-node-dotnet.sbxenv.yaml`](../examples/opencode-node-dotnet.sbxenv.yaml) |
| Python + uv | [`examples/opencode-python.sbxenv.yaml`](../examples/opencode-python.sbxenv.yaml) |
| Go | [`examples/opencode-go.sbxenv.yaml`](../examples/opencode-go.sbxenv.yaml) |
| Rust + cargo | [`examples/opencode-rust.sbxenv.yaml`](../examples/opencode-rust.sbxenv.yaml) |

## 4. Adjust the config to your project

In `.sbxenv.yaml`:

| Setting | What to do |
| ------- | ---------- |
| `name:` | A unique name for this sandbox (used to scope its secrets). |
| `kits:` | Drop the mixin lines your project doesn't need, but keep `base` (every kit requires it) and add `global-opencode-config` when using a model provider (`zeldoc`, `copilot` — their config fragments only merge through it). See [mixins.md](mixins.md). Need project-specific settings (private feeds, env vars, agent notes)? See [project-kit.md](project-kit.md). |
| `workspace.path:` | Leave as is — it targets your repo. Point it elsewhere only if the env file sits outside the project. |

## 5. Run it

From your project root:

```bash
sbx env run
```

The sandbox starts, prints a banner, and launches OpenCode automatically.
Quitting OpenCode drops you into the sandbox's login shell (git, builds,
`dotnet`/`pnpm`, …) — relaunch OpenCode anytime with `o`, and exit the shell
when you're done with the sandbox.

## Before the first run: pick a model provider

The agent needs a model provider:

- **[Zeldoc.ai](https://zeldoc.ai)** (the default the examples use) — get,
  register, and approve your key: [zeldoc-api-key.md](zeldoc-api-key.md), or
- **GitHub Copilot** (no key needed; sign in inside the sandbox) —
  [copilot-setup.md](copilot-setup.md). Both compose.

Optional: a GitHub personal access token so the agent can push and open PRs
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

## Daily use

- Start the sandbox again with `sbx env run` from your project root.
- Want personal tweaks (more memory, debug env vars)? Keep them in a
  gitignored `local.sbxenv.yaml` — see [local-overrides.md](local-overrides.md).
- Changed the `kits:` list? Kit changes only apply to **new** sandboxes —
  recreate with `sbx rm <name>` and `sbx env run` again.
- Remove the sandbox (and its scoped secrets) with `sbx env rm`.
- Want your host's global agent skills available to the sandboxed agent? See
  [agent-skills.md](agent-skills.md).

## Troubleshooting

Blocked downloads, git auth, the `.env` guard, stale changes:
[troubleshooting.md](troubleshooting.md).
