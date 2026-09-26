# docker-sandboxing

Run [OpenCode](https://opencode.ai) in a sandboxed [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/) VM inside your own repo: copy the example for your stack into your project, run one command, and OpenCode starts in a VM with your toolchain preinstalled.

The sandbox is the isolation boundary: outbound network is deny-by-default and API keys are injected by a proxy — the sandbox only ever sees placeholders.

## Use it in your project

### 1. Copy the example for your stack

Copy the example matching your stack into your project's root and rename it `sbxenv.yaml`. Commit it so teammates get the same sandbox.

| Your stack | Copy this example |
| ---------- | ----------------- |
| .NET | [`examples/opencode-dotnet.sbxenv.yaml`](examples/opencode-dotnet.sbxenv.yaml) |
| Node.js | [`examples/opencode-node.sbxenv.yaml`](examples/opencode-node.sbxenv.yaml) |
| Python + uv | [`examples/opencode-python.sbxenv.yaml`](examples/opencode-python.sbxenv.yaml) |
| Go | [`examples/opencode-go.sbxenv.yaml`](examples/opencode-go.sbxenv.yaml) |
| Rust + cargo | [`examples/opencode-rust.sbxenv.yaml`](examples/opencode-rust.sbxenv.yaml) |

### 2. Adjust the config to your project

| Setting | What to do |
| ------- | ---------- |
| `name:` | Set it to your project's name. |
| `agent:` | Keep the workload image (or point it at your own). |
| `kits:` | Drop the lines your project doesn't need — the mixin reference below describes what each adds. |
| `workspace.path: .` | Leave as is — it targets your repo. |

### 3. Run it

From your project root:

```bash
sbx env run
```

OpenCode starts automatically. When you quit it, the sandbox exits — rerun `sbx env run` whenever you want it back.

Full walkthrough (prerequisites, first run, daily use): [docs/getting-started.md](docs/getting-started.md).

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

| Mixin | Adds | Notes |
| ----- | ---- | ------ |
| `opencode` | the OpenCode agent (newest npm release at creation), permissive OpenCode config, and the provider-config merge (`OPENCODE_CONFIG`) | pairs with `launch-opencode` |
| `env-guard` | workspace `.env` guard: removes `.env` files (clone mode) or fails creation (direct mode), via one install-hook command in its descriptor | optional |
| `opencode-zeldoc` | Zeldoc.ai model provider (proxy-managed key, config, hosts) | — |
| `opencode-copilot` | GitHub Copilot model provider (device-flow sign-in, config fragment, hosts) | — |
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

Everything below is for developing the kits and mixins.

```text
├── kit-<stack>/                      # shell workload kits (kind: workload, v3): one per stack —
├── kit-node/                         #   <kit>.yaml descriptor + <kit>.dockerfile. Each Dockerfile
├── kit-dotnet/                       #   builds the whole workload on the shell base image
├── kit-python/                       #   (docker/sandbox-templates:shell): stack toolchain, git-lfs,
├── kit-go/                           #   Node via NVM + PNPM, Playwright; launches bash by default
├── kit-rust/                         #   (compose launch-opencode to run OpenCode); published as
│                                     #   sbx-kit-<stack>
├── mixins/                           # one single-purpose mixin per area (kind: mixin, v3):
│   │                                 #   <area>.yaml descriptor (capabilities: network policy,
│   │                                 #   credentials, lifecycle, agent-context) + optional
│   │                                 #   <area>.dockerfile for shipped config — consumed as git
│   │                                 #   references (kits: list or --kit), not published as images
│   ├── opencode/                     # the OpenCode agent: newest npm release at creation,
│   │                                 #   permissive config, provider-fragment merge
│   ├── launch-opencode/              # launches OpenCode at sandbox startup (the workloads'
│   │                                 #   bash entrypoint execs its agent shim)
│   ├── env-guard/                    # the workspace .env guard (one install-hook command)
│   └── <area>/                       # stack toolchains, providers, registries, tools
├── examples/*.sbxenv.yaml            # consumer environment examples (one per stack)
├── docs/                             # user guides (see above)
└── agent-guidance/                   # worktrees, versioning, commit conventions
```

Every workload builds on `docker/sandbox-templates:shell` (agent user, workspace, persistent-shell env, tini) and adds its stack in its own `kit-*/kit-*.dockerfile`: Git (+ git-lfs), Node.js via NVM + PNPM, and Playwright with the Chromium headless shell. The workloads are shell kits — they launch bash; compose the `opencode` mixin (the agent, newest npm release by default — pin it with its `version` arg) and the `launch-opencode` mixin (startup launch) to run OpenCode on any of them.

Kits are [v3 kit descriptors](https://github.com/docker/sandbox-kit-spec) (`# syntax=docker/sandbox-kit:3`), requiring sbx v0.45+. The workload defines the image and launch; mixins declare network/credential/lifecycle capabilities and ship files through their own image layers.

### Working on the repo

Work in a git worktree branched from `main` ([agent-guidance/worktrees.md](agent-guidance/worktrees.md)) — other agent sessions share this checkout concurrently.

```bash
docker buildx build kit-node/ --file kit-node/kit-node.yaml   # build = validate (strict descriptor decode)
docker buildx build mixins/dotnet/ --file mixins/dotnet/dotnet.yaml # (declaration-only mixins have no Dockerfile)
sbx env run                   # dev sandbox: built-in shell agent + mixins/ from the working copy
```

Kit changes only apply to new sandboxes: `sbx rm <name>` + `sbx env run` (or the wizard again). Releases are cut by release-please from Conventional Commits on `main` — see [agent-guidance/versioning.md](agent-guidance/versioning.md) and [agent-guidance/commit-messages.md](agent-guidance/commit-messages.md). Repo conventions: [AGENTS.md](AGENTS.md).
