# Mixins

A mixin adds exactly one capability area to the sandbox: network egress
rules, environment settings, install steps, and a note for the agent. The
sandbox network policy is deny-by-default — the union of the composed
mixins' rules is the only outbound traffic. Drop the mixin lines your
project doesn't need from the `kits:` list in your `.sbxenv.yaml`.

## Catalog

| Mixin | Adds |
| ----- | ---- |
| `opencode-runtime` | Egress the agent itself needs: updates, model lists (models.dev), npm-hosted plugins |
| `zeldoc` | Zeldoc.ai model provider (proxy-managed key, provider config, Zeldoc hosts) |
| `git` | Git hosting egress (HTTPS + SSH), proxy-managed GitHub auth, worktree workflow for the agent |
| `node` | Node.js toolchain egress: nodejs.org (nvm installs), npm registry, pnpm.io docs; pnpm installs gated to versions published ≥24h ago |
| `openapi-ts` | openapi-ts.dev docs egress for the openapi-typescript + openapi-fetch packages |
| `dotnet` | .NET/NuGet egress + telemetry opt-out |
| `python` | Python toolchain egress: PyPI index + package files (uv/pip), astral.sh (uv installer), python.org docs |
| `go` | Go toolchain egress: module proxy + checksum DB (`go get`/`go install`, GOTOOLCHAIN toolchain downloads), dl.google.com (go.dev/dl artifacts), go.dev/golang.org docs |
| `rust` | Rust toolchain egress: crates.io index/API + package CDN (cargo), static.rust-lang.org (rustup), sh.rustup.rs (installer), rust-lang.org + docs.rs docs |
| `docker` | Registry egress for the Docker engine inside the sandbox |
| `apt` | Ubuntu/Microsoft package mirrors for `sudo apt-get` |
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
| Node only | `opencode-runtime`, `zeldoc`, `git`, `node` |
| .NET only | `opencode-runtime`, `zeldoc`, `git`, `dotnet` |
| Python only | `opencode-runtime`, `zeldoc`, `git`, `python` |
| Go only | `opencode-runtime`, `zeldoc`, `git`, `go` |
| Rust only | `opencode-runtime`, `zeldoc`, `git`, `rust` |
| Node + typed API client (openapi-typescript) | `opencode-runtime`, `zeldoc`, `git`, `node`, `openapi-ts` |
| Node + in-sandbox Docker | `opencode-runtime`, `zeldoc`, `git`, `node`, `docker` |
| Browser automation | `opencode-runtime`, `zeldoc`, `git`, `node`, `apt`, `browser`, `playwright*` |

Notes:

- The `playwright*` mixins and `sbx` run `apt` at creation — compose the
  `apt` mixin with them.
- Every set should include `zeldoc` (the model provider) and
  `opencode-runtime` (the agent's own egress).

## Changing mixins

Kit changes only apply to **new** sandboxes:

```bash
sbx rm <sandbox-name>
sbx env run
```

Need something specific to your project (a private feed, env vars, an
agent note)? Add an in-project kit — see
[project-kit.md](project-kit.md).
