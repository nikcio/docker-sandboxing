# Mixins

A mixin adds exactly one capability area to the sandbox: network egress
rules, environment settings, install steps, and a note for the agent. The
sandbox network policy is deny-by-default — the union of the composed
mixins' rules is the only outbound traffic. Drop the mixin lines your
project doesn't need from the `kits:` list in your `.sbxenv.yaml`.

## Catalog

| Mixin | Adds |
| ----- | ---- |
| `base` | The AGENTS.md logic + MCP gateway every kit needs: agent guidance files, the shared `AGENTS.md` base, the AGENTS.md rebuild hook, MCP gateway registration (startup hook + backstop). Required by every kit — keep this line |
| `opencode-config` | Permissive OpenCode config (edit/bash/webfetch allowed — the sandbox is the isolation boundary), dropped into the global config layer. Required by every kit — keep this line |
| `opencode-entrypoint` | The entrypoint runtime: startup banner, mixin hook runner, opencode autostart, `.env` guard, login shell on exit. Required by every kit — keep this line |
| `opencode-runtime` | Egress the agent itself needs: updates, model lists (models.dev), npm-hosted plugins |
| `zeldoc` | Zeldoc.ai model provider (proxy-managed key, provider config fragment, Zeldoc hosts) |
| `copilot` | GitHub Copilot model provider (OAuth device-flow sign-in via `/connect`, provider config fragment, GitHub/Copilot API egress — see [copilot-setup.md](copilot-setup.md)) |
| `git` | Git hosting egress (HTTPS + SSH), proxy-managed GitHub auth, worktree workflow for the agent |
| `uniform` | Uniform DXP egress: docs site, dashboard + Management API (uniform.app), Edge Delivery API (uniform.global, incl. EU + image CDN), proxy-managed `x-api-key` auth (see [uniform-api-key.md](uniform-api-key.md)) |
| `omnium` | Omnium OMS/e-commerce egress: REST API hosts (production/test/dev, each with Swagger), tech docs, proxy-managed `Authorization: Bearer` auth (see [omnium-api-key.md](omnium-api-key.md)) |
| `node` | Node.js toolchain egress: nodejs.org (nvm installs), npm registry, pnpm.io docs; pnpm installs gated to versions published ≥24h ago |
| `openapi-ts` | openapi-ts.dev docs egress for the openapi-typescript + openapi-fetch packages |
| `dotnet` | .NET/NuGet egress + telemetry opt-out |
| `python` | Python toolchain egress: PyPI index + package files (uv/pip), astral.sh (uv installer), python.org docs |
| `go` | Go toolchain egress: module proxy + checksum DB (`go get`/`go install`, GOTOOLCHAIN toolchain downloads), dl.google.com (go.dev/dl artifacts), go.dev/golang.org docs |
| `rust` | Rust toolchain egress: crates.io index/API + package CDN (cargo), static.rust-lang.org (rustup), sh.rustup.rs (installer), rust-lang.org + docs.rs docs |
| `docker` | Registry egress for the Docker engine inside the sandbox |
| `apt` | Ubuntu/Microsoft package mirrors for `sudo apt-get` + background package-cache update at start |
| `browser` | Google Chrome install (no network rules — sites stay gated by the other mixins) |
| `playwright` | Playwright + Chromium headless shell (smallest download) |
| `playwright-chromium` | Playwright + full Chromium |
| `playwright-all` | Playwright + Chromium, Firefox, WebKit (~1 GB+ download) |
| `sbx` | The `sbx` CLI inside the sandbox for kit authoring (validate/inspect/pack) |

Network hosts per mixin are listed at the top of each
`mixins/<area>/spec.yaml`.

## Common sets

| Project | Keep these kit lines |
| ------- | -------------------- |
| Full stack (.NET + Node + Docker + browser tests) | all lines in the example |
| Node only | `base`, `opencode-config`, `opencode-entrypoint`, `opencode-runtime`, `zeldoc`, `git`, `node` |
| .NET only | `base`, `opencode-config`, `opencode-entrypoint`, `opencode-runtime`, `zeldoc`, `git`, `dotnet` |
| Python only | `base`, `opencode-config`, `opencode-entrypoint`, `opencode-runtime`, `zeldoc`, `git`, `python` |
| Go only | `base`, `opencode-config`, `opencode-entrypoint`, `opencode-runtime`, `zeldoc`, `git`, `go` |
| Rust only | `base`, `opencode-config`, `opencode-entrypoint`, `opencode-runtime`, `zeldoc`, `git`, `rust` |
| Node + typed API client (openapi-typescript) | `base`, `opencode-config`, `opencode-entrypoint`, `opencode-runtime`, `zeldoc`, `git`, `node`, `openapi-ts` |
| Node + in-sandbox Docker | `base`, `opencode-config`, `opencode-entrypoint`, `opencode-runtime`, `zeldoc`, `git`, `node`, `docker` |
| Node frontend with Uniform | `base`, `opencode-config`, `opencode-entrypoint`, `opencode-runtime`, `zeldoc`, `git`, `node`, `uniform` |
| Browser automation | `base`, `opencode-config`, `opencode-entrypoint`, `opencode-runtime`, `zeldoc`, `git`, `node`, `apt`, `browser`, `playwright*` |

Notes:

- Three mixins are required by every kit: `base` (agent guidance,
  AGENTS.md rebuild, MCP gateway), `opencode-config` (permissive OpenCode
  config), and `opencode-entrypoint` (entrypoint runtime). Keep all three
  in every set.
- The `playwright*` mixins and `sbx` run `apt` at creation — compose the
  `apt` mixin with them.
- Every set should include at least one model provider (`zeldoc` and/or
  `copilot`) and `opencode-runtime` (the agent's own egress).

## Model providers (zeldoc / copilot)

Provider mixins don't fight over one config file: each ships a
pure-JSON fragment to `~/.config/opencode/providers.d/NN-<provider>.json`
inside the sandbox, and the `opencode-config` mixin merges all
fragments into the single config OpenCode loads via `OPENCODE_CONFIG`
at every start:

- `enabled_providers` lists are **unioned** — compose `zeldoc` and
  `copilot` together and both stay selectable with `/models`.
- Fragments merge in filename order and later fragments win conflicts —
  the `20-` zeldoc fragment sorts after the `10-` copilot one, so the
  **default model stays `zeldoc/zdev-2`** when both are composed. Point
  `model` at a Copilot model in a project-level `opencode.jsonc` (see
  [project-kit.md](project-kit.md)) to flip the default.
- Composing only one provider keeps that provider's default model.

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
