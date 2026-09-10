# docker-sandboxing

Run [OpenCode](https://opencode.ai) in a sandboxed [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/)
VM inside your own repo: copy the example for your stack into your project,
run one command, and OpenCode starts in a VM with your toolchain preinstalled.

The sandbox is the isolation boundary: outbound network is deny-by-default and
API keys are injected by a proxy — the sandbox only ever sees placeholders.

## Use it in your project

### 1. Copy the example for your stack

Copy the example matching your stack into your project's root and rename it
`.sbxenv.yaml`. Commit it so teammates get the same sandbox.

| Your stack | Copy this example |
| ---------- | ----------------- |
| .NET + Node.js | [`examples/opencode-node-dotnet.sbxenv.yaml`](examples/opencode-node-dotnet.sbxenv.yaml) |
| Node.js | [`examples/opencode-node.sbxenv.yaml`](examples/opencode-node.sbxenv.yaml) |
| Python + uv | [`examples/opencode-python.sbxenv.yaml`](examples/opencode-python.sbxenv.yaml) |
| Go | [`examples/opencode-go.sbxenv.yaml`](examples/opencode-go.sbxenv.yaml) |
| Rust + cargo | [`examples/opencode-rust.sbxenv.yaml`](examples/opencode-rust.sbxenv.yaml) |

### 2. Adjust the config to your project

| Setting | What to do |
| ------- | ---------- |
| `name:` | Set it to your project's name. |
| `kits:` | Drop the lines your project doesn't need — the mixin reference below marks which mixins are required. |
| `workspace.path: .` | Leave as is — it targets your repo. |

### 3. Run it

From your project root:

```bash
sbx env run
```

OpenCode starts automatically. Quitting it drops you into the sandbox's login
shell (git, builds, `dotnet`/`pnpm`, …) — relaunch opencode anytime with `o`,
and exit the shell when you're done with the sandbox.

Full walkthrough (prerequisites, first run, daily use):
[docs/getting-started.md](docs/getting-started.md).

## Guides

| Guide | Covers |
| ----- | ------ |
| [Getting started](docs/getting-started.md) | prerequisites, the three steps, first run, daily use |
| [Set your Zeldoc API key](docs/zeldoc-api-key.md) | get, register, and approve the model provider key |
| [Use GitHub Copilot](docs/copilot-setup.md) | run the agent on your Copilot subscription (or both providers) |
| [Create a GitHub PAT](docs/github-pat.md) | correct permissions and scope, store it per sandbox, rotate it |
| [Set your Uniform API key](docs/uniform-api-key.md) | create a Uniform service-account key, register it with `sbx secret set uniform` |
| [Set your Omnium API token](docs/omnium-api-key.md) | create an Omnium API user, mint and refresh the bearer token |
| [Mixins](docs/mixins.md) | what each mixin adds, common sets, changing them |
| [Project-specific config](docs/project-kit.md) | an in-project kit: project feeds, env vars, files, agent notes |
| [Local overrides](docs/local-overrides.md) | personal settings in a gitignored `local.sbxenv.yaml`, merged over the team's `.sbxenv.yaml` |
| [Agent skills](docs/agent-skills.md) | share your host's global agent skills with the sandboxed agent (`sbx skills import`) |
| [Agent memory](docs/agents-md.md) | which `AGENTS.md` the agent loads, the sandbox environment file, where to put your rules |
| [Troubleshooting](docs/troubleshooting.md) | blocked downloads, git auth, the .env guard, stale changes |
| [GitHub repo setup](docs/repo-setup.md) | maintainer setup: release App, secrets, branch protection, Docker Hub |
| [Branch policies for AI agents](docs/agent-branch-protection.md) | protect your repo when an autonomous agent works in it: PRs, required checks, no force pushes, release tags, secret scanning |

## Mixin reference

| Mixin | Adds | Required |
| ----- | ---- | -------- |
| `base` | the entrypoint runtime (mixin hook runner, opencode autostart, login shell on exit), agent guidance (shared `AGENTS.md` base + guidance files), the AGENTS.md rebuild hook, MCP gateway registration | every kit |
| `global-opencode-config` | permissive OpenCode config (edit/bash/webfetch allowed — the sandbox is the isolation boundary), dropped into the global config layer, plus the combined provider config (`OPENCODE_CONFIG` merge) | with a model provider (`zeldoc`, `copilot`) |
| `env-guard` | workspace `.env` guard: removes `.env` files (clone mode) or refuses to start (direct mode) | optional |
| `banner` | the startup banner | optional |
| `opencode-runtime` | egress the agent itself needs (updates, models.dev, plugins) | — |
| `zeldoc` | Zeldoc.ai model provider (proxy-managed key, config, hosts) | — |
| `copilot` | GitHub Copilot model provider (device-flow sign-in, config fragment, hosts) | — |
| `uniform` | Uniform DXP egress: docs, dashboard + Management API, Edge Delivery API (incl. EU + image CDN), proxy-managed `x-api-key` | — |
| `omnium` | Omnium OMS/e-commerce API egress (proxy-managed bearer token) | — |
| `git` | git hosting egress, proxy-managed GitHub auth, worktree workflow | — |
| `node` | nodejs.org + npm registry egress | — |
| `openapi-ts` | openapi-ts.dev docs egress (openapi-typescript / openapi-fetch) | — |
| `dotnet` | NuGet/Microsoft egress, telemetry opt-out | — |
| `nikcio-openapi-codegen` | openapi-code-generator .NET tool install (C# codegen from OpenAPI 3.x) + openapi.nikcio.com docs egress | needs `dotnet` |
| `python` | PyPI egress for uv/pip | — |
| `go` | Go module proxy + checksum DB egress (`go get`/`go install`, GOTOOLCHAIN downloads) | — |
| `rust` | crates.io + rustup egress for cargo | — |
| `docker` | registry egress for the in-sandbox Docker engine | — |
| `apt` | Ubuntu/Microsoft package mirrors for `sudo apt-get` + background package-cache update at start | — |
| `browser` | Google Chrome install (software only) | — |
| `playwright` / `playwright-chromium` / `playwright-all` | Playwright + the listed browsers | needs `apt` |
| `sbx` | the `sbx` CLI inside the sandbox (kit authoring) | needs `apt` |

Full mixin details: [docs/mixins.md](docs/mixins.md).

---

## Developing this repo

Everything below is for developing the template, kit, and mixins.

```text
├── template-node-dotnet/Dockerfile   # → opencode-node-dotnet:v1: .NET SDK, Node LTS via NVM, PNPM
├── template-node/Dockerfile          # → opencode-node:v1: Node LTS via NVM, PNPM
├── template-python/Dockerfile        # → opencode-python:v1: uv-managed CPython, uv
├── template-go/Dockerfile            # → opencode-go:v1: official Go toolchain (GO_VERSION build-arg,
│                                     #   GOTOOLCHAIN=auto for newer toolchains via the module proxy)
├── template-rust/Dockerfile          # → opencode-rust:v1: rustup-managed stable toolchain
│                                     #   (clippy + rustfmt + rust-analyzer), cargo, C build toolchain
├── kit-node-dotnet/                  # sandbox kit (kind: sandbox): template image + one-line
├── kit-node/                         #   entrypoint wrapper that execs the shared runtime from the
├── kit-python/                       #   base mixin (mixin hooks → auto-start opencode → login shell
├── kit-go/                           #   on exit); errors clearly when that mixin is missing
├── kit-rust/
├── kit-published-<stack>/            # same kits, sandbox.image pinned to the public Docker Hub
│                                     #   tags — release-please bumps versions + tags; keep in
│                                     #   sync with kit-<stack>/
├── mixins/
│   ├── base/                         # the shared runtime the kit wrappers exec: the entrypoint
│   │                                 #   (~/.sandbox-kit/entrypoint.sh), agent guidance
│   │                                 #   (~/.sandbox-agents/*.md), hooks (agents-md.sh,
│   │                                 #   mcp-gateway.sh) — what it adds: mixin reference above
│   ├── global-opencode-config/       # owns OPENCODE_CONFIG: permissive OpenCode config
│   │                                 #   (~/.config/opencode/opencode.jsonc) + provider-fragment
│   │                                 #   merge (merge-global-opencode-config.sh) fed by
│   │                                 #   provider mixins' providers.d/ fragments
│   ├── env-guard/                    # the .env guard hook (hooks.d/env-guard.sh)
│   ├── banner/                       # the banner hook (hooks.d/banner.sh)
│   └── <area>/                       # one single-purpose mixin per area (kind: mixin): network
│                                     #   rules, env vars, credentials, install steps, agent
│   │                                 #   memory note (.sbx-agents.d/<area>.md) — composed
│                                     #   explicitly at launch (--kit flags or kits: list)
├── scripts/
│   ├── bootstrap.ps1 / bootstrap.sh  # local dev setup (build, load, secrets, validate)
│   └── new-sandbox.ps1 / new-sandbox.sh  # sandbox creation wizard
├── examples/*.sbxenv.yaml            # consumer environment examples (one per stack)
├── docs/                             # user guides (see above)
├── agent-guidance/                   # worktrees, versioning, commit conventions
└── opencode-builtin-kit.spec.yaml    # reference snapshot of Docker's built-in opencode kit —
                                      #   upstream of the template image; nothing loads it
```

All five templates extend `docker/sandbox-templates:opencode-docker` and ship
Git (+ git-lfs), `gh`, and the `o` PATH shim for relaunching opencode.

### Working on the repo

Work in a git worktree branched from `main`
([agent-guidance/worktrees.md](agent-guidance/worktrees.md)) — other agent
sessions share this checkout concurrently.

```bash
./scripts/bootstrap.sh     # or bootstrap.ps1: build + load the template,
                           # register the Zeldoc key, validate kit + mixins
sbx kit validate kit-node-dotnet/   # validate a kit or mixin after edits
./scripts/new-sandbox.sh   # wizard launcher for a dev sandbox
                           # (--source local picks up uncommitted edits)
```

Kit changes only apply to new sandboxes: `sbx rm <name>` + `sbx env run`
(or the wizard again).
Releases are cut by release-please from Conventional Commits on `main` —
see [agent-guidance/versioning.md](agent-guidance/versioning.md) and
[agent-guidance/commit-messages.md](agent-guidance/commit-messages.md).
Repo conventions: [AGENTS.md](AGENTS.md).
