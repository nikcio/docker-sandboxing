# docker-sandboxing

A collection of premade mixins for [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/): reusable building blocks that add stack toolchains, tools, network egress, and model providers to a sandboxed VM in your own repo. Copy the example for your stack into your project, run one command, and your coding agent starts in a VM with the toolchain installed at creation.

The sandbox is the isolation boundary. Outbound network is deny by default, and a proxy injects API keys, so the sandbox only ever sees placeholders.

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
| `kits:` | Drop the lines your project doesn't need. See the mixin reference below. |
| `workspace.path: .` | Leave as is, it targets your repo. |

### 3. Run it

From your project root:

```bash
sbx env run
```

OpenCode starts automatically. When you quit it, the sandbox exits. Rerun `sbx env run` whenever you want it back.

Full walkthrough (prerequisites, first run, daily use): [docs/getting-started.md](docs/getting-started.md).

## Prefer a GUI?

[Sandy](https://github.com/Nikcio-labs/sandy) is a desktop app for Docker Sandboxes with guided `sbx` setup and an always-refreshing local sandbox dashboard. It lives in its own repo now: [github.com/Nikcio-labs/sandy](https://github.com/Nikcio-labs/sandy).

## Guides

| Guide | Covers |
| ----- | ------ |
| [Getting started](docs/getting-started.md) | prerequisites, the steps, first run, daily use |
| [Sandy (desktop app)](https://github.com/Nikcio-labs/sandy) | guided setup, local sandbox dashboard, and details inspector (separate repo) |
| [Set your Zeldoc API key](docs/zeldoc-api-key.md) | get, register, and approve the model provider key |
| [Use GitHub Copilot](docs/copilot-setup.md) | run the agent on your Copilot subscription (or both providers) |
| [Create a GitHub PAT](docs/github-pat.md) | correct permissions and scope, store it per sandbox, rotate it |
| [Set your Uniform API key](docs/uniform-api-key.md) | create a Uniform service-account key, register it with `sbx secret set uniform` |
| [Set your Omnium API token](docs/omnium-api-key.md) | create an Omnium API user, mint and refresh the bearer token |
| [Mixins](docs/mixins.md) | what each mixin adds, common sets, changing them |
| [Project-specific config](docs/project-kit.md) | an in-project kit: project feeds, env vars, files, agent notes |
| [Local overrides](docs/local-overrides.md) | personal settings in a gitignored `local.sbxenv.yaml`, merged over the team's `sbxenv.yaml` |
| [Agent skills](docs/agent-skills.md) | share your host's global agent skills with the sandboxed agent (`sbx skills import`) |
| [Troubleshooting](docs/troubleshooting.md) | blocked downloads, git auth, the .env guard, stale changes |
| [GitHub repo setup](docs/repo-setup.md) | maintainer setup: release App, secrets, branch protection |
| [Branch policies for AI agents](docs/agent-branch-protection.md) | protect your repo when an autonomous agent works in it: PRs, required checks, no force pushes, release tags, secret scanning |

## Mixin reference

| Mixin | Adds | When to compose |
| ----- | ---- | -------- |
| `opencode` | OpenCode itself: installs OpenCode 2 via the official curl installer when the sandbox lacks it, permissive OpenCode config (edit/bash/webfetch allowed, because the sandbox is the isolation boundary) dropped into the global config layer, the combined provider config (`OPENCODE_CONFIG` merge), and the locale env (`LANG`/`LC_ALL`) | optional for model providers, `opencode-zeldoc`/`opencode-copilot` manage their own config |
| `env-guard` | workspace `.env` guard: removes `.env` files (clone mode) or refuses to start (direct mode) | optional |
| `opencode-zeldoc` | Zeldoc.ai model provider (proxy-managed key, config, hosts) | none |
| `opencode-copilot` | GitHub Copilot model provider (device-flow sign-in, provider config merged into the global opencode config, hosts) | none |
| `github-cli` | GitHub CLI (`gh`) + git identity; auth rides the built-in agent's proxy-managed `github` credential | none |
| `repo-clone` | clones extra GitHub repos (from the `repositories` kit argument) into the workspace at creation, via the built-in `github` credential; inaccessible repos are skipped | needs `github-cli` |
| `uniform` | Uniform DXP egress: docs, dashboard + Management API, Edge Delivery API (incl. EU + image CDN), proxy-managed `x-api-key` | none |
| `omnium` | Omnium OMS/e-commerce API egress (proxy-managed bearer token) | none |
| `node` | nodejs.org + npm registry egress | none |
| `openapi-ts` | openapi-ts.dev docs egress (openapi-typescript / openapi-fetch) | none |
| `dotnet` | NuGet/Microsoft egress, telemetry opt-out | none |
| `nikcio-openapi-codegen` | openapi-code-generator .NET tool install (C# codegen from OpenAPI 3.x) + openapi.nikcio.com docs egress | needs `dotnet` |
| `python` | PyPI egress for uv/pip | none |
| `go` | Go module proxy + checksum DB egress (`go get`/`go install`, GOTOOLCHAIN downloads) | none |
| `rust` | crates.io + rustup egress for cargo | none |
| `docker-hub` / `gcr` / `ghcr` / `mcr` | registry egress for the in-sandbox Docker engine, per registry | none |
| `playwright` | Playwright + the Chromium headless shell | none |
| `sbx` | the `sbx` CLI inside the sandbox (kit authoring) | none |
| `zsh` | zsh + Oh My Zsh for the agent user (login shell set to zsh) | none |
| `herdr` | the [Herdr](https://herdr.dev) CLI, an agent runtime; agents keep working when detached, resume from any machine | none |
| `open-egress` | allows all outbound domains (replaces the deny by default baseline) | opt-in |

Full mixin details: [docs/mixins.md](docs/mixins.md).

---

## Developing this repo

Everything below is for developing the mixins in this repo.

```text
├── mixins/
│   ├── opencode/                     # OpenCode itself: CLI install (curl, v2) + owns
│   │                                 #   OPENCODE_CONFIG: permissive OpenCode config
│   │                                 #   (~/.config/opencode/opencode.json) + mixins.d/
│   │                                 #   fragment merge (third-party kit fragments;
│   │                                 #   provider mixins merge their config into the
│   │                                 #   global config themselves, so they work standalone)
│   ├── env-guard/                    # the workspace .env guard (startup command)
│   └── <area>/                       # one single-purpose premade mixin per area (kind: mixin):
│   │                                 #   network rules, env vars, credentials, install steps,
│   │                                 #   composed explicitly at launch (--kit flags or kits: list)
├── examples/*.sbxenv.yaml            # consumer environment examples (one per stack),
│                                     #   composing this repo's mixins
├── docs/                             # user guides (see above)
├── agent-guidance/                   # worktrees, versioning, commit conventions
├── sbxenv.yaml                       # this repo's dev sandbox: local paths into the working copy
└── opencode.jsonc                    # opencode settings for working in this repo
```

The sandboxes run Docker's built-in agents (e.g. `agent: opencode`, on Docker's
[`docker/sandbox-templates:opencode-docker`](https://docs.docker.com/ai/sandboxes/)
image). No images are published from this repo, and there are no kits. Each
mixin owns its own install steps, locale env, and network rules; stack
toolchains (Node, .NET, Python, Go, Rust) install at sandbox creation via the
`mixins/<stack>` mixins.

### Working on the repo

Work in a git worktree branched from `main` ([agent-guidance/worktrees.md](agent-guidance/worktrees.md)); other agent sessions share this checkout concurrently.

```bash
sbx kit pack mixins/opencode -o /tmp/x.zip   # validate a mixin after edits (works offline; `sbx kit validate` requires being signed in to Docker)
sbx env run                                  # dev sandbox: mixins/ loaded from the working copy
```

Mixin changes only apply to new sandboxes: `sbx rm <name>` + `sbx env run`. Releases are cut by release-please from Conventional Commits on `main`; see [agent-guidance/versioning.md](agent-guidance/versioning.md) and [agent-guidance/commit-messages.md](agent-guidance/commit-messages.md). Repo conventions: [AGENTS.md](AGENTS.md).
