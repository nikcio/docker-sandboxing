# Mixins

A mixin adds exactly one capability area to the sandbox: network egress
rules, environment settings, install steps, and a note for the agent. The
sandbox network policy is deny-by-default — the union of the composed
mixins' rules is the only outbound traffic. Drop the mixin lines your
project doesn't need from the `kits:` list in your `sbxenv.yaml`.

## Any mixin on any kit

Every toolchain mixin (`node`, `dotnet`, `python`, `go`, `rust`) and the
tool mixins (`browser`, `playwright`, `sbx`, `nikcio-openapi-codegen`)
carry a self-contained check-and-install `setup.install` step: at sandbox
creation it verifies the tool is present and installs it when the kit's
base image lacks it. The built-in Docker template image
(`docker/sandbox-templates:opencode-docker`) ships no stack toolchain, so
the stack mixins do the install. Compose, for example, the `python` mixin
onto the base kit and the sandbox gets a working `python3` + `uv`; no
rebuild needed.

Rules of thumb:

- Install paths and env match the mixin-owned toolchains (same
  `DOTNET_ROOT`, nvm in the agent home, uv-managed CPython,
  `/usr/local/go`, agent-owned rustup).
- Install steps run at **creation only** (kit changes never apply to
  running sandboxes) and need egress: the mixin owning the download host
  (e.g. `node` for npm installs, `dotnet` for the .NET SDK feed) plus the
  Ubuntu apt mirrors (`archive.ubuntu.com`/`security.ubuntu.com`) when the
  step calls `apt-get`. A blocked download fails the install step — check
  `sbx policy log`.
- When the tool is already present the check is a cheap no-op, so keeping
  the install steps in every composition is safe.

## Catalog

| Mixin | Adds |
| ----- | ---- |
| `global-opencode-config` | Permissive OpenCode config (edit/bash/webfetch allowed — the sandbox is the isolation boundary), dropped into the global config layer, plus the combined provider config (`OPENCODE_CONFIG` merge). Required when composing a model provider (`opencode-zeldoc`, `opencode-copilot`) — their fragments only merge through it |
| `env-guard` | Workspace `.env` guard: removes `.env` files (clone mode) or refuses to start (direct mode). Optional — the examples compose it |
| `opencode-update` | Rolls opencode to the newest npm release at sandbox creation (npm-registry egress). Not in the example compositions — add the line for fresh opencode on every sandbox; runs once at creation, no start cost |
| `opencode-zeldoc` | Zeldoc.ai model provider (proxy-managed key, provider config fragment, Zeldoc hosts) |
| `opencode-copilot` | GitHub Copilot model provider (OAuth device-flow sign-in via `/connect`, provider config fragment, GitHub/Copilot API egress — see [copilot-setup.md](copilot-setup.md)) |
| `github-cli` | GitHub CLI (`gh`) install + proxy-managed GitHub auth (see [github-pat.md](github-pat.md)) |
| `uniform` | Uniform DXP egress: docs site, dashboard + Management API (uniform.app), Edge Delivery API (uniform.global, incl. EU + image CDN), proxy-managed `x-api-key` auth (see [uniform-api-key.md](uniform-api-key.md)) |
| `omnium` | Omnium OMS/e-commerce egress: REST API hosts (production/test/dev, each with Swagger), tech docs, proxy-managed `Authorization: Bearer` auth (see [omnium-api-key.md](omnium-api-key.md)) |
| `node` | Node.js toolchain egress: nodejs.org (nvm installs), npm registry (pnpm/npm/npx), pnpm.io docs; pnpm installs gated to versions published ≥24h ago; check-and-install (nvm node + pnpm) when the base image lacks them |
| `openapi-ts` | openapi-ts.dev docs egress for the openapi-typescript + openapi-fetch packages |
| `dotnet` | .NET/NuGet egress + telemetry opt-out; check-and-install (SDK via apt feed / dot.net script) when the base image lacks dotnet |
| `nikcio-openapi-codegen` | Installs the openapi-code-generator .NET global tool (Nikcio.OpenApiCodeGen — the `openapi-codegen` CLI) + openapi.nikcio.com docs egress; needs `dotnet`; ensures the SDK first |
| `python` | Python toolchain egress: PyPI index + package files (uv/pip), astral.sh (uv installer), python.org docs; check-and-install (uv + CPython) when the base image lacks python3 |
| `go` | Go toolchain egress: module proxy + checksum DB (`go get`/`go install`, GOTOOLCHAIN toolchain downloads), dl.google.com (go.dev/dl artifacts), go.dev/golang.org docs; check-and-install (official tarball) when the base image lacks go |
| `rust` | Rust toolchain egress: crates.io index/API + package CDN (cargo), static.rust-lang.org (rustup), sh.rustup.rs (installer), rust-lang.org + docs.rs docs; check-and-install (rustup) when the base image lacks cargo |
| `docker-hub` | Docker Hub registry egress for the in-sandbox Docker engine |
| `gcr` | Google Container Registry egress for the in-sandbox Docker engine |
| `ghcr` | GitHub Container Registry egress for the in-sandbox Docker engine |
| `mcr` | Microsoft Container Registry egress for the in-sandbox Docker engine |
| `browser` | Google Chrome install (dl.google.com egress for the .deb; sites stay gated by the other mixins); skipped when the base image has it |
| `playwright` | Playwright + Chromium headless shell (smallest download); installs node via nvm when the base image lacks one |
| `sbx` | The `sbx` CLI inside the sandbox for kit authoring (validate/inspect/pack); skipped when the base image has it |
| `zsh` | zsh + Oh My Zsh for the agent user (login shell set to zsh); check-and-install when the base image lacks it |
| `open-egress` | Allows all outbound domains (the `**` rule) — replaces the deny-by-default baseline; local deny rules and org policy still take precedence. Opt-in: the stock kit and examples do not compose it |

Network hosts per mixin are listed at the top of each
`mixins/<area>/spec.yaml`, and each mixin ships a
`mixins/<area>/README.md` with usage and details.

## Common sets

| Project | Keep these kit lines |
| ------- | -------------------- |
| Full stack (.NET + Node + Docker + browser tests) | all lines in the example |
| Node only | `global-opencode-config`, `env-guard`, `opencode-update`, `opencode-zeldoc`, `github-cli`, `node` |
| .NET only | `global-opencode-config`, `env-guard`, `opencode-update`, `opencode-zeldoc`, `github-cli`, `dotnet` |
| .NET + OpenAPI codegen (C# models) | `global-opencode-config`, `env-guard`, `opencode-update`, `opencode-zeldoc`, `github-cli`, `dotnet`, `nikcio-openapi-codegen` |
| Python only | `global-opencode-config`, `env-guard`, `opencode-update`, `opencode-zeldoc`, `github-cli`, `python` |
| Go only | `global-opencode-config`, `env-guard`, `opencode-update`, `opencode-zeldoc`, `github-cli`, `go` |
| Rust only | `global-opencode-config`, `env-guard`, `opencode-update`, `opencode-zeldoc`, `github-cli`, `rust` |
| Node + typed API client (openapi-typescript) | `global-opencode-config`, `env-guard`, `opencode-update`, `opencode-zeldoc`, `github-cli`, `node`, `openapi-ts` |
| Node + in-sandbox Docker | `global-opencode-config`, `env-guard`, `opencode-update`, `opencode-zeldoc`, `github-cli`, `node`, `docker-hub` |
| Node frontend with Uniform | `global-opencode-config`, `env-guard`, `opencode-update`, `opencode-zeldoc`, `github-cli`, `node`, `uniform` |
| Browser automation | `global-opencode-config`, `env-guard`, `opencode-update`, `opencode-zeldoc`, `github-cli`, `node`, `browser`, `playwright` |

Notes:

- `global-opencode-config` is the only merge path for model-provider
  config fragments — required when composing a model provider (`opencode-zeldoc`,
  `opencode-copilot`).
  `env-guard` (the no-.env policy) is optional.
- The `playwright` mixin and `sbx` run `apt` at creation — their install
  steps need the Ubuntu apt mirrors. Compose a registry mixin
  (`docker-hub`, `gcr`, `ghcr`, `mcr`) per registry the in-sandbox Docker
  engine pulls from.
- Every set should include at least one model provider (`opencode-zeldoc` and/or
  `opencode-copilot`). Add `opencode-update` to keep opencode current: the built-in
  template image bakes a fixed opencode version, and this mixin rolls it
  to the newest npm release at creation.

## Model providers (opencode-zeldoc / opencode-copilot)

Provider mixins don't fight over one config file: each ships a
pure-JSON fragment to `~/.config/opencode/mixins.d/NN-<provider>.json`
inside the sandbox, and the `global-opencode-config` mixin merges all
fragments into the single config OpenCode loads via `OPENCODE_CONFIG`
at every start. Compose `global-opencode-config` with any provider
mixin — it is required with them (their fragments only merge through
it):

- `enabled_providers` lists are **unioned** — compose `opencode-zeldoc` and
  `opencode-copilot` together and both stay selectable with `/models`.
- Fragments merge in filename order and later fragments win conflicts,
  but **no fragment sets the default model** — the project-level
  `opencode.jsonc` owns it. Commit one in the repo root (see
  [project-kit.md](project-kit.md)), e.g. `"model": "zeldoc/zdev-2"` or
  `"model": "github-copilot/<model-id>"`; without it opencode falls back
  to its own default, and `/models` always works per session.

Swapping providers (or changing any mixin) only applies to **new**
sandboxes — see "Changing mixins" below.

## Changing mixins

Kit changes only apply to **new** sandboxes:

```bash
sbx rm <sandbox-name>
sbx env run
```

Need something specific to your project (a private feed, env vars, an
agent note)? Add an in-project kit — see
[project-kit.md](project-kit.md).
