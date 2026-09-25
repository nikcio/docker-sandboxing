# Mixins

A mixin adds exactly one capability area to the sandbox: network egress
rules, credential requests, install steps, files, and a note for the
agent. The sandbox network policy is deny-by-default — the union of the
composed mixins' rules is the only outbound traffic. Drop the mixin lines
your project doesn't need from the `kits:` list in your `sbxenv.yaml`.

Kits here are v3 descriptors (`mixins/<area>/<area>.yaml` plus a
`<area>.dockerfile` where the mixin ships files), published as plain OCI
images on Docker Hub (`docker.io/nikcio/sbx-mixin-<area>`). See
[Version compatibility](https://docs.docker.com/ai/sandboxes/#version-compatibility):
a v3 workload requires v3 mixins, and sbx v0.45+.

## Any mixin on any workload

Stack mixins don't have to match the workload's baked toolchain. Every
toolchain mixin (`node`, `dotnet`, `python`, `go`, `rust`) and the tool
mixins (`browser`, `playwright`, `sbx`, `nikcio-openapi-codegen`) carry a
self-contained check-and-install lifecycle `install` hook: at sandbox
creation it verifies the tool is present and installs it when the
workload image lacks it. Compose, for example, the `python` mixin onto
the Go workload and the sandbox gets a working `python3` + `uv`; no
rebuild needed.

Rules of thumb:

- Install paths and env match the workload images (same `DOTNET_ROOT`,
  nvm in the agent home, uv-managed CPython, `/usr/local/go`,
  agent-owned rustup) — a later image rebuild converges to the same
  layout.
- Install hooks run at **creation only** (kit changes never apply to
  running sandboxes) and need egress: the mixin declares it in the
  `install` phase of its network policy (the runtime closes install
  grants before the agent starts). A blocked download fails the install
  hook — check `sbx policy log`.
- On a workload that already has the tool the check is a cheap no-op, so
  keeping the install hooks in every composition is safe.

## Catalog

| Mixin | Adds |
| ----- | ---- |
| `opencode` | The OpenCode agent (newest npm release by default; pin it with the mixin's `version` arg), permissive OpenCode config (edit/bash/webfetch allowed — the sandbox is the isolation boundary), and the combined provider config (`OPENCODE_CONFIG` merge). Required in every OpenCode sandbox, and required when composing a model provider (`opencode-zeldoc`, `opencode-copilot`) — their fragments only merge through it |
| `env-guard` | Workspace `.env` guard: removes `.env` files (clone mode) or fails sandbox creation (direct mode). Self-contained — the script ships in the mixin's own image and runs as a lifecycle install hook. Optional — the examples compose it |
| `opencode-zeldoc` | Zeldoc.ai model provider (proxy-managed key, provider config fragment, Zeldoc hosts) |
| `opencode-copilot` | GitHub Copilot model provider (OAuth device-flow sign-in via `/connect`, provider config fragment, GitHub/Copilot API egress — see [copilot-setup.md](copilot-setup.md)) |
| `github-cli` | GitHub CLI (`gh`) install + proxy-managed GitHub auth (see [github-pat.md](github-pat.md)) |
| `uniform` | Uniform DXP egress: docs site, dashboard + Management API (uniform.app), Edge Delivery API (uniform.global, incl. EU + image CDN), proxy-managed `x-api-key` auth (see [uniform-api-key.md](uniform-api-key.md)) |
| `omnium` | Omnium OMS/e-commerce egress: REST API hosts (production/test/dev, each with Swagger), tech docs, proxy-managed `Authorization: Bearer` auth (see [omnium-api-key.md](omnium-api-key.md)) |
| `node` | Node.js toolchain egress: nodejs.org (nvm installs), npm registry (pnpm/npm/npx), pnpm.io docs; pnpm installs gated to versions published ≥24h ago; check-and-install (nvm node + pnpm) for workloads without them |
| `openapi-ts` | openapi-ts.dev docs egress for the openapi-typescript + openapi-fetch packages |
| `dotnet` | .NET/NuGet egress + telemetry opt-out; check-and-install (SDK via apt feed / dot.net script) for workloads without dotnet |
| `nikcio-openapi-codegen` | Installs the openapi-code-generator .NET global tool (Nikcio.OpenApiCodeGen — the `openapi-codegen` CLI) + openapi.nikcio.com docs egress; requires the `dotnet` mixin's `dotnet@10.0` capability, ensures the SDK itself otherwise |
| `python` | Python toolchain egress: PyPI index + package files (uv/pip), python.org docs; check-and-install (uv + CPython) for workloads without python3 |
| `go` | Go toolchain egress: module proxy + checksum DB (`go get`/`go install`, GOTOOLCHAIN toolchain downloads), dl.google.com (go.dev/dl artifacts), go.dev/golang.org docs; check-and-install (official tarball) for workloads without go |
| `rust` | Rust toolchain egress: crates.io index/API + package CDN (cargo), static.rust-lang.org (rustup), rust-lang.org + docs.rs docs; check-and-install (rustup) for workloads without cargo |
| `docker-hub` | Docker Hub registry egress for the in-sandbox Docker engine |
| `gcr` | Google Container Registry egress for the in-sandbox Docker engine |
| `ghcr` | GitHub Container Registry egress for the in-sandbox Docker engine |
| `mcr` | Microsoft Container Registry egress for the in-sandbox Docker engine |
| `browser` | Google Chrome install (dl.google.com egress for the .deb; sites stay gated by the other mixins); skipped when the workload has it |
| `playwright` | Playwright + Chromium headless shell (smallest download); installs node via nvm when the workload lacks one |
| `sbx` | The `sbx` CLI inside the sandbox for kit authoring; skipped when the workload has it |
| `open-egress` | Allows all outbound domains (the `**` rule) — replaces the deny-by-default baseline; local deny rules and org policy still take precedence. Opt-in: the stock kits and examples do not compose it |

Network hosts per mixin are listed in the mixin's descriptor
(`mixins/<area>/<area>.yaml`), and each mixin ships a
`mixins/<area>/README.md` with usage and details.

## Common sets

| Project | Keep these kit lines |
| ------- | -------------------- |
| Full stack (.NET + Node + Docker + browser tests) | all lines in the example |
| Node only | `opencode`, `env-guard`, `opencode-zeldoc`, `github-cli` |
| .NET only | `opencode`, `env-guard`, `zeldoc`, `github-cli`, `dotnet` |
| .NET + OpenAPI codegen (C# models) | `opencode`, `env-guard`, `zeldoc`, `github-cli`, `dotnet`, `nikcio-openapi-codegen` |
| Python only | `opencode`, `env-guard`, `zeldoc`, `github-cli`, `python` |
| Go only | `opencode`, `env-guard`, `zeldoc`, `github-cli`, `go` |
| Rust only | `opencode`, `env-guard`, `zeldoc`, `github-cli`, `rust` |
| Node + typed API client (openapi-typescript) | `opencode`, `env-guard`, `zeldoc`, `github-cli`, `node`, `openapi-ts` |
| Node + in-sandbox Docker | `opencode`, `env-guard`, `zeldoc`, `github-cli`, `node`, `docker-hub` |
| Node frontend with Uniform | `opencode`, `env-guard`, `zeldoc`, `github-cli`, `node`, `uniform` |
| Browser automation | `opencode`, `env-guard`, `zeldoc`, `github-cli`, `node`, `browser`, `playwright` |

Notes:

- `opencode` (the agent + config + provider merge) is required in every
  OpenCode sandbox and with every model provider (`opencode-zeldoc`, `opencode-copilot`) —
  their config fragments only merge through it. `env-guard` (the
  no-.env policy) is optional.
- The `playwright` and `sbx` mixins run `apt` at creation — their
  install hooks declare the Ubuntu apt mirrors in their install-phase
  policy. Compose a registry mixin (`docker-hub`, `gcr`, `ghcr`, `mcr`)
  per registry the in-sandbox Docker engine pulls from.
- `opencode-zeldoc` and/or `opencode-copilot` give the agent a model provider. The
  `opencode` mixin installs the newest opencode at every creation by
  default (arg `version: latest`); pin it with a `version` value to
  freeze the agent release.

## Version overrides

Every installed tool's version is a kit argument, overridable per
environment (`--kit-arg` in `sbxenv.yaml`'s `kits:` entries or at
`sbx run`) or per build (`--build-arg` for workload kit images):

- **Workload images** (stack toolchains baked at build): the `kit-*/`
descriptors declare args like `node_version`, `go_version`,
`python_version`, `rust_version`, `dotnet_channel`, `playwright_version`
— build the kit with `--build-arg <arg>=<value>` to change them.
- **Creation-time installs** (mixin install hooks): `opencode`'s
`version` (default `latest`), `node`'s `nvm_version`/`node_version`/
`pnpm_version`, `python`'s `python_version`, `rust`'s `rust_version`,
`dotnet`'s `channel`, `browser`'s `channel`, `nikcio-openapi-codegen`'s
`tool_version` — override with `--kit-arg`:

```yaml
kits:
  - source: docker.io/nikcio/sbx-mixin-opencode:vX.Y.Z
    args:
      version: "1.18.32"
```

Each arg carries a `default` (the documented behavior) and a `pattern`
validating overrides; a failing value is rejected at sandbox creation.

## Model providers (opencode-zeldoc / opencode-copilot)

Provider mixins don't fight over one config file: each ships a
pure-JSON fragment to `~/.config/opencode/mixins.d/NN-<provider>.json`
inside the sandbox (via its image layer), and the `opencode` mixin
merges all fragments into the single config OpenCode loads via
`OPENCODE_CONFIG` — an install hook, before the agent starts. Compose
`opencode` with any provider mixin — it is required with them (their
fragments only merge through it):

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
