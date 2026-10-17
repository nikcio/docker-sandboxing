# Getting started

Run a coding agent in a sandboxed VM inside your own repo in a few steps: **install `sbx`**, **sign in**, **copy the example for your stack**, **adjust the config**, and **run one command**. The sandboxes work with any coding agent; the shipped examples use [OpenCode](https://opencode.ai), which starts in a VM with your stack toolchain installed at creation.

The sandbox is the isolation boundary. Outbound network is deny by default, and a proxy injects API keys, so the sandbox only ever sees placeholders.

## 1. Install `sbx`

Install the [`sbx` CLI](https://docs.docker.com/ai/sandboxes/install/) for your platform (Docker Desktop is not required, `sbx` runs its own VMs):

| Platform | Command |
| -------- | ------- |
| macOS | `brew install docker/tap/sbx` |
| Windows | `winget install -h Docker.sbx` |
| Ubuntu 24.04+ | `curl -fsSL https://get.docker.com \| sudo SBX=1 sh` |

<details>
<summary>System requirements</summary>

- **macOS:** Sonoma 14 or later, Apple silicon
- **Windows:** Windows 11, 64-bit Intel/AMD, Windows Hypervisor Platform
- **Linux:** Ubuntu 24.04+, 64-bit CPU with KVM enabled, your user in the `kvm` group

See Docker's [installation guide](https://docs.docker.com/ai/sandboxes/install/) for the full requirements and manual-install options. Version 0.39.0 or newer is recommended.
</details>

## 2. Sign in

```bash
sbx login
```

The command opens a browser for Docker OAuth.

## 3. Copy the example for your stack

Copy the example matching your stack into your project's root and rename it `sbxenv.yaml`. Commit it so teammates get the same sandbox.

| Your stack | Copy this example |
| ---------- | ----------------- |
| .NET | [`examples/opencode-dotnet.sbxenv.yaml`](../examples/opencode-dotnet.sbxenv.yaml) |
| Node.js | [`examples/opencode-node.sbxenv.yaml`](../examples/opencode-node.sbxenv.yaml) |
| Python + uv | [`examples/opencode-python.sbxenv.yaml`](../examples/opencode-python.sbxenv.yaml) |
| Go | [`examples/opencode-go.sbxenv.yaml`](../examples/opencode-go.sbxenv.yaml) |
| Rust + cargo | [`examples/opencode-rust.sbxenv.yaml`](../examples/opencode-rust.sbxenv.yaml) |

## 4. Adjust the config to your project

In `sbxenv.yaml`:

| Setting | What to do |
| ------- | ---------- |
| `name:` | A unique name for this sandbox (used to scope its secrets). |
| `kits:` | Drop the mixin lines your project doesn't need. Include at least one model provider (`opencode-zeldoc`, `opencode-copilot`); see [mixins.md](mixins.md). Need project-specific settings (private feeds, env vars, agent notes)? See [project-kit.md](project-kit.md). |
| `workspace.path:` | Leave as is, it targets your repo. Point it elsewhere only if the env file sits outside the project. |

## 5. Run it

From your project root:

```bash
sbx env run
```

The sandbox starts and launches OpenCode automatically. When you quit OpenCode, the sandbox exits. Rerun `sbx env run` whenever you want it back.

## Before the first run: pick a model provider

The agent needs a model provider:

- [Zeldoc.ai](https://zeldoc.ai) (the default the examples use). Get, register, and approve your key: [zeldoc-api-key.md](zeldoc-api-key.md), or
- GitHub Copilot (no key needed; sign in inside the sandbox). See [copilot-setup.md](copilot-setup.md). Both compose.

Optional: a GitHub personal access token so the agent can push and open PRs, see [github-pat.md](github-pat.md). Cloning public repos (read-only) and git over SSH work without one; pushing over HTTPS always needs it.

The mixins are fetched from `github.com/nikcio/docker-sandboxing` and composed onto Docker's built-in `opencode` agent (the `docker/sandbox-templates:opencode-docker` image), so no builds are needed on your machine. The `opencode` mixin brings the OpenCode config layer and the locale env; stack toolchains install at sandbox creation via the stack mixin in the `kits:` list.

## Host settings

Two one-time `sbx` settings on your host:

- Allow the kit source (required, sbx only fetches kits from allowed sources). The setting replaces the whole list, so merge with your current entries. Check them first with `sbx settings get kit.allowedSources`:

  ```bash
  sbx settings set kit.allowedSources '["docker.io/","github.com/nikcio/"]'
  ```

- Optional: let the sandboxed agent read images you paste:

  ```bash
  sbx settings set clipboard.imagePaste true
  ```

## Daily use

- Start the sandbox again with `sbx env run` from your project root.
- Want personal tweaks (more memory, debug env vars)? Keep them in a gitignored `local.sbxenv.yaml`; see [local-overrides.md](local-overrides.md).
- Changed the `kits:` list? Kit changes only apply to new sandboxes. Recreate with `sbx rm <name>` and `sbx env run` again.
- Remove the sandbox (and its scoped secrets) with `sbx env rm`.
- Want your host's global agent skills available to the sandboxed agent? See [agent-skills.md](agent-skills.md).

## Troubleshooting

Blocked downloads, git auth, the `.env` guard, stale changes: [troubleshooting.md](troubleshooting.md).
