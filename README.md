# docker-sandboxing

Run [OpenCode](https://opencode.ai) in a sandboxed [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/)
VM inside your own repo: copy the example for your stack into your project,
run one command, and OpenCode starts in a VM with your toolchain preinstalled.

The sandbox is the isolation boundary: outbound network is deny-by-default and
API keys are injected by a proxy — the sandbox only ever sees placeholders.

## Use it in your project

### 1. Copy the example for your stack

Copy the example matching your stack into your project's root and rename it
`sbxenv.yaml`. Commit it so teammates get the same sandbox.

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
shell (git, builds, `dotnet`/`pnpm`, …) — exit the shell when you're done with
the sandbox.

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
| [Local overrides](docs/local-overrides.md) | personal settings in a gitignored `local.sbxenv.yaml`, merged over the team's `sbxenv.yaml` |
| [Agent skills](docs/agent-skills.md) | share your host's global agent skills with the sandboxed agent (`sbx skills import`) |
| [Agent memory](docs/agents-md.md) | which `AGENTS.md` the agent loads, the sandbox environment file, where to put your rules |
| [Troubleshooting](docs/troubleshooting.md) | blocked downloads, git auth, the .env guard, stale changes |
| [GitHub repo setup](docs/repo-setup.md) | maintainer setup: release App, secrets, branch protection, Docker Hub |
| [Branch policies for AI agents](docs/agent-branch-protection.md) | protect your repo when an autonomous agent works in it: PRs, required checks, no force pushes, release tags, secret scanning |

## Mixin reference

| Mixin | Adds | Required |
| ----- | ---- | -------- |
| `agents-md` | sandbox `AGENTS.md` baseline (guidance files + per-mixin notes) | every kit |
| `global-opencode-config` | permissive OpenCode config (edit/bash/webfetch allowed — the sandbox is the isolation boundary), dropped into the global config layer, plus the combined provider config (`OPENCODE_CONFIG` merge) | with a model provider (`zeldoc`, `copilot`) |
| `env-guard` | workspace `.env` guard: removes `.env` files (clone mode) or refuses to start (direct mode) | optional |
| `opencode-update` | opencode rolled to the newest npm release at sandbox creation (npm-registry egress) — add the kit line for fresh opencode on every sandbox | opt-in |
| `zeldoc` | Zeldoc.ai model provider (proxy-managed key, config, hosts) | — |
| `copilot` | GitHub Copilot model provider (device-flow sign-in, config fragment, hosts) | — |
| `github-cli` | GitHub CLI (`gh`) + proxy-managed GitHub auth | — |
| `uniform` | Uniform DXP egress: docs, dashboard + Management API, Edge Delivery API (incl. EU + image CDN), proxy-managed `x-api-key` | — |
| `omnium` | Omnium OMS/e-commerce API egress (proxy-managed bearer token) | — |
| `node` | nodejs.org + npm registry egress | — |
| `openapi-ts` | openapi-ts.dev docs egress (openapi-typescript / openapi-fetch) | — |
| `dotnet` | NuGet/Microsoft egress, telemetry opt-out | — |
| `nikcio-openapi-codegen` | openapi-code-generator .NET tool install (C# codegen from OpenAPI 3.x) + openapi.nikcio.com docs egress | needs `dotnet` |
| `python` | PyPI egress for uv/pip | — |
| `go` | Go module proxy + checksum DB egress (`go get`/`go install`, GOTOOLCHAIN downloads) | — |
| `rust` | crates.io + rustup egress for cargo | — |
| `docker-hub` / `gcr` / `ghcr` / `mcr` | registry egress for the in-sandbox Docker engine, per registry | — |
| `browser` | Google Chrome install (software only) | — |
| `playwright` | Playwright + the Chromium headless shell | — |
| `sbx` | the `sbx` CLI inside the sandbox (kit authoring) | — |
| `open-egress` | allows all outbound domains (replaces the deny-by-default baseline) | opt-in |

Full mixin details: [docs/mixins.md](docs/mixins.md).

---

## Developing this repo

Everything below is for developing the template, kit, and mixins.

```text
├── template-node-dotnet/Dockerfile   # → opencode-node-dotnet:v1: .NET SDK, Node via NVM, PNPM, Playwright
├── template-node/Dockerfile          # → opencode-node:v1: Node via NVM, PNPM, Playwright
├── template-python/Dockerfile        # → opencode-python:v1: uv-managed CPython, uv, Node via NVM, PNPM, Playwright
├── template-go/Dockerfile            # → opencode-go:v1: official Go toolchain (GO_VERSION build-arg,
│                                     #   GOTOOLCHAIN=auto for newer toolchains via the module proxy),
│                                     #   Node via NVM, PNPM, Playwright
├── template-rust/Dockerfile          # → opencode-rust:v1: rustup-managed stable toolchain
│                                     #   (clippy + rustfmt + rust-analyzer), cargo, C build toolchain,
│                                     #   Node via NVM, PNPM, Playwright
├── kit-<stack>/                      # sandbox kits (kind: sandbox): one per stack template —
├── kit-node/                         #   same shape as kit/ (image + opencode entrypoint),
├── kit-node-dotnet/                  #   with the stack image from Docker Hub
├── kit-python/
├── kit-go/
├── kit-rust/
├── mixins/
│   ├── agents-md/                    # the sandbox AGENTS.md baseline: guidance files
│   │                                 #   (~/.sandbox-agents.md + ~/.sandbox-agents/*.md) and the
│   │                                 #   startup rebuild of the workspace AGENTS.md from it plus
│   │                                 #   every composed mixin's note (~/.sbx-agents.d/)
│   ├── global-opencode-config/       # owns OPENCODE_CONFIG: permissive OpenCode config
│   │                                 #   (~/.config/opencode/opencode.jsonc) + provider-fragment
│   │                                 #   merge fed by provider mixins' mixins.d/ fragments
│   ├── env-guard/                    # the workspace .env guard (startup command)
│   └── <area>/                       # one single-purpose mixin per area (kind: mixin): network
│                                     #   rules, env vars, credentials, install steps, agent
│   │                                 #   memory note (.sbx-agents.d/<area>.md) — composed
│                                     #   explicitly at launch (--kit flags or kits: list)
├── kit/                              # the base kit (docker/sandbox-templates image), used by
│                                     #   this repo's own sbxenv.yaml
├── examples/*.sbxenv.yaml            # consumer environment examples (one per stack)
├── docs/                             # user guides (see above)
├── agent-guidance/                   # worktrees, versioning, commit conventions
└── opencode-builtin-kit.spec.yaml    # reference snapshot of Docker's built-in opencode kit —
                                      #   upstream of the template image; nothing loads it
```

All five templates extend `docker/sandbox-templates:opencode-docker` and ship
Git (+ git-lfs), Node.js via NVM + PNPM, and Playwright with the Chromium
headless shell.

### Working on the repo

Work in a git worktree branched from `main`
([agent-guidance/worktrees.md](agent-guidance/worktrees.md)) — other agent
sessions share this checkout concurrently.

```bash
sbx kit validate kit-node/             # validate a kit or mixin after edits
sbx env run                   # dev sandbox: kit/ + mixins/ loaded from the working copy
```

Kit changes only apply to new sandboxes: `sbx rm <name>` + `sbx env run`
(or the wizard again).
Releases are cut by release-please from Conventional Commits on `main` —
see [agent-guidance/versioning.md](agent-guidance/versioning.md) and
[agent-guidance/commit-messages.md](agent-guidance/commit-messages.md).
Repo conventions: [AGENTS.md](AGENTS.md).
