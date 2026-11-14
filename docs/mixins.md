# Mixins

The premade mixins in this repo each add exactly one capability area to the sandbox: network egress rules, environment settings, tools, and a note for the agent. The sandbox network policy is deny by default, and the union of the composed mixins' rules is the only outbound traffic. Drop the mixin lines your project doesn't need from the `kits:` list in your `sbxenv.yaml`.

## Toolchains and tools

Every toolchain mixin (`node`, `dotnet`, `python`, `go`, `rust`) and the tool mixins (`playwright`, `sbx`, `nikcio-openapi-codegen`, `opencode`) bake their tools into the mixin image when it is built, so every sandbox starts with the toolchain present — no install step runs at sandbox creation. The built-in Docker template image (`docker/sandbox-templates:opencode-docker`, which the built-in `opencode` agent runs on) ships no stack toolchain, so the stack mixins do the install. Compose, for example, the `python` mixin and the sandbox gets a working `python3` + `uv`.

Rules of thumb:

- Toolchains land under `/opt` with profile scripts putting them on `PATH` for login shells; paths and env match the mixin-owned toolchains (`DOTNET_ROOT`, uv-managed CPython, `/usr/local/go`, rustup).
- Installs run at image build only. At runtime each mixin grants exactly the egress its tools need (download hosts, docs hosts); the union of the composed mixins' `runtime:` rules is the sandbox's outbound policy.
- When the agent hits a blocked host at runtime, check `sbx policy log` and add the host to the owning mixin (see [troubleshooting.md](troubleshooting.md)).

## Version arguments

Every mixin that installs software declares its version arguments in its `<name>.yaml` (`args:` block) and defaults to the latest version (upstream release, or the distro's latest apt package for apt installs) when no value is supplied. The values are passed as build arguments when the mixin image is built. Pin a version in `sbxenv.yaml` with an `args:` entry on the mixin's `kits:` item:

```yaml
kits:
  - source: git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/node&ref=v3.1.2
    args:
      pnpmVersion: "10"
```

Leave the argument out (or empty) and the install resolves the latest release when the image is built. Which arguments a mixin takes is listed in its `mixins-v3/<mixin>/<mixin>.yaml` (`args:` block) and README. Version changes only apply to **new** sandboxes.

## Catalog

| Mixin | Adds |
| ----- | ---- |
| `opencode` | OpenCode itself: the OpenCode CLI baked into the mixin image at build (`opencodeVersion` arg, empty = latest), permissive OpenCode config (edit/bash/webfetch allowed, because the sandbox is the isolation boundary) dropped into the global config layer, the combined provider config (`OPENCODE_CONFIG` merge), and the locale env. Provider mixins (`opencode-zeldoc`, `opencode-copilot`) also work without it |
| `env-guard` | Workspace `.env` guard: removes `.env` files (clone mode) or refuses to start (direct mode). Optional, but the examples compose it |
| `opencode-zeldoc` | Zeldoc.ai model provider (proxy-managed key, provider config merged into the global opencode config, Zeldoc hosts) |
| `opencode-copilot` | GitHub Copilot model provider (OAuth device-flow sign-in via `/connect`, provider config merged into the global opencode config, GitHub/Copilot API egress; see [copilot-setup.md](copilot-setup.md)) |
| `github-cli` | GitHub CLI (`gh`) install + git-identity setup; GitHub auth rides the built-in agent's proxy-managed `github` credential (see [github-pat.md](github-pat.md)) |
| `repo-clone` | Clones extra GitHub repositories (from the `repositories` kit argument) into the workspace at creation, via the built-in proxy-managed `github` credential; repos the token cannot access are skipped with a warning (see the mixin's README) |
| `uniform` | Uniform DXP egress: docs site, dashboard + Management API (uniform.app), Edge Delivery API (uniform.global, incl. EU + image CDN), proxy-managed `x-api-key` auth (see [uniform-api-key.md](uniform-api-key.md)) |
| `omnium` | Omnium OMS/e-commerce egress: REST API hosts (production/test/dev, each with Swagger), tech docs, proxy-managed `Authorization: Bearer` auth (see [omnium-api-key.md](omnium-api-key.md)) |
| `node` | Node.js toolchain (nvm-managed Node + pnpm, baked into the mixin image at build) + npm ecosystem egress: nodejs.org (nvm), npm registry (pnpm/npm/npx), pnpm.io docs; pnpm installs gated to versions published ≥24h ago |
| `openapi-ts` | openapi-ts.dev docs egress for the openapi-typescript + openapi-fetch packages |
| `dotnet` | .NET/NuGet egress + telemetry opt-out; the .NET SDK (official install script) baked into the mixin image at build |
| `nikcio-openapi-codegen` | Installs the openapi-code-generator .NET global tool (`Nikcio.OpenApiCodeGen`, the `openapi-codegen` CLI) + openapi.nikcio.com docs egress; needs `dotnet`; ensures the SDK first |
| `python` | Python toolchain (uv + CPython, baked into the mixin image at build) + egress: PyPI index + package files (uv/pip), astral.sh (uv installer), python.org docs |
| `go` | Go toolchain (official toolchain, baked into the mixin image at build) + egress: module proxy + checksum DB (`go get`/`go install`, GOTOOLCHAIN toolchain downloads), dl.google.com (go.dev/dl artifacts), go.dev/golang.org docs |
| `rust` | Rust toolchain (rustup-managed, baked into the mixin image at build) + egress: crates.io index/API + package CDN (cargo), static.rust-lang.org (rustup), sh.rustup.rs (installer), rust-lang.org + docs.rs docs |
| `docker-hub` | Docker Hub registry egress for the in-sandbox Docker engine |
| `gcr` | Google Container Registry egress for the in-sandbox Docker engine |
| `ghcr` | GitHub Container Registry egress for the in-sandbox Docker engine |
| `mcr` | Microsoft Container Registry egress for the in-sandbox Docker engine |
| `playwright` | Playwright + the Chromium headless shell (smallest download), baked into the mixin image at build |
| `sbx` | The `sbx` CLI inside the sandbox for kit authoring (validate/inspect/pack), baked into the mixin image at build |
| `zsh` | zsh + Oh My Zsh for the agent user (login shell set to zsh), baked into the mixin image at build |
| `herdr` | The [Herdr](https://herdr.dev) CLI, the agent runtime: a background server that owns the agents' terminals, so agents keep working when you detach and can be resumed from any machine; baked into the mixin image at build |
| `open-egress` | Allows all outbound domains (the `**` rule) and replaces the deny by default baseline; local deny rules and org policy still take precedence. Opt-in: the examples do not compose it |

Network hosts per mixin are listed in each `mixins-v3/<area>/<area>.yaml` (the `network-policy` capability), and each mixin ships a `mixins-v3/<area>/README.md` with usage and details.

## Common sets

| Project | Keep these kit lines |
| ------- | -------------------- |
| Full stack (.NET + Node + Docker + browser tests) | all lines in the example |
| Node only | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli`, `node` |
| .NET only | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli`, `dotnet` |
| .NET + OpenAPI codegen (C# models) | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli`, `dotnet`, `nikcio-openapi-codegen` |
| Python only | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli`, `python` |
| Go only | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli`, `go` |
| Rust only | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli`, `rust` |
| Node + typed API client (openapi-typescript) | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli`, `node`, `openapi-ts` |
| Node + in-sandbox Docker | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli`, `node`, `docker-hub` |
| Node frontend with Uniform | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli`, `node`, `uniform` |
| Browser automation | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli`, `node`, `playwright` |

Notes:

- Model providers (`opencode-zeldoc`, `opencode-copilot`) manage their own config, so the `opencode` mixin is not required for them. `env-guard` (the no-.env policy) is optional.
- Compose a registry mixin (`docker-hub`, `gcr`, `ghcr`, `mcr`) per registry the in-sandbox Docker engine pulls from.
- Every set should include at least one model provider (`opencode-zeldoc` and/or `opencode-copilot`).

## Model providers (opencode-zeldoc / opencode-copilot)

Provider mixins manage their own config: each runs a startup command that merges its keys (`enabled_providers`, and for Zeldoc the `plugin` list) straight into the global `~/.config/opencode/opencode.json(c)`, creating the file if it does not exist. That means they work standalone (opencode installed by any means), no `opencode` mixin required:

- `enabled_providers` entries are unioned. Compose `opencode-zeldoc` and `opencode-copilot` together and both stay selectable with `/models` (each startup command only touches its own keys).
- No provider mixin sets the default model; the project-level `opencode.jsonc` owns it. Commit one in the repo root (see [project-kit.md](project-kit.md)), e.g. `"model": "zeldoc/zdev"` or `"model": "github-copilot/<model-id>"`; without it opencode falls back to its own default, and `/models` always works per session.

Swapping providers (or changing any mixin) only applies to new sandboxes; see "Changing mixins" below.

## Changing mixins

Kit changes only apply to new sandboxes:

```bash
sbx rm <sandbox-name>
sbx env run
```

Need something specific to your project (a private feed, env vars, an agent note)? Add an in-project kit; see [project-kit.md](project-kit.md).
