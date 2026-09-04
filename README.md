# docker-sandboxing

A [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/) **template** and
composable **kits** that give [OpenCode](https://opencode.ai) a full-stack dev
environment:

- **Template** (`template/Dockerfile` → `opencode-node-dotnet:v1`)
  - .NET SDK (LTS, `dotnet`)
  - Node.js LTS via [NVM](https://github.com/nvm-sh/nvm) (`nvm`, `node`)
  - [PNPM](https://pnpm.io) (`pnpm`)
  - Git (+ git-lfs) and common CLI utilities
  - Extends the built-in `docker/sandbox-templates:opencode-docker` image
- **Sandbox kit** (`kit/`, `kind: sandbox`, `extends: opencode`) — thin agent
  definition only: points at the template via `sandbox.image` and sets the
  entrypoint. No rules of its own.
- **Mixins** (`mixins/<area>/`, `kind: mixin`) — each defines the rules for
  exactly one area and stacks via `--kit` or a `.sbxenv.yaml`:

  | Mixin | Area | Provides |
  | ----- | ---- | -------- |
  | `zeldoc` | model provider | Zeldoc.ai credential (proxy-managed key), provider config (`OPENCODE_CONFIG`), Zeldoc hosts |
  | `git` | git workflow | git hosting egress (HTTPS + SSH) + the mandatory worktree workflow in agent memory |
  | `node` | Node toolchain | nodejs.org + npm registry egress, nvm/pnpm notes |
  | `dotnet` | .NET toolchain | NuGet/Microsoft egress, telemetry opt-out, telemetry deny |
  | `docker` | containers | registry egress for the Docker engine inside the sandbox |
  | `opencode-runtime` | agent runtime | opencode.ai/models.dev/Zen egress, npm-hosted plugins |
  | `apt` | OS packages | Ubuntu/Microsoft package mirrors for `sudo apt-get` |

  Drop the mixins you don't need — e.g. a pure Node project skips `dotnet`,
  `docker`, and `apt`.

  Mixin memory notes (`agentInstructions`) are written to `kits-memory/<mixin>.md`
  next to the main `AGENTS.md` (with an index) when the agent launches.

## Repo layout

```text
├── AGENTS.md                                  # base agent instructions (worktrees + repo rules)
├── template/
│   └── Dockerfile                             # sandbox template image
├── kit/
│   └── spec.yaml                              # thin sandbox kit (template + entrypoint only)
├── mixins/
│   ├── zeldoc/                                # provider credential + config + network
│   │   ├── spec.yaml
│   │   └── files/home/.config/opencode/zeldoc.jsonc
│   ├── git/                                   # git egress + worktree workflow memory
│   ├── node/                                  # node/npm egress
│   ├── dotnet/                                # nuget/microsoft egress + telemetry deny
│   ├── docker/                                # registry egress
│   ├── opencode-runtime/                      # agent runtime egress
│   └── apt/                                   # package mirror egress
├── scripts/
│   ├── bootstrap.ps1                          # host setup (Windows)
│   ├── bootstrap.sh                           # host setup (Linux/macOS/Git Bash)
│   ├── new-sandbox.ps1                        # configurable `sbx-new` launcher (Windows)
│   └── new-sandbox.sh                         # configurable `sbx-new` launcher (bash)
└── examples/
    └── opencode-node-dotnet.sbxenv.yaml       # project .sbxenv.yaml example (composition)
```

## Prerequisites

- [Docker Desktop](https://docs.docker.com/desktop/) with the `sbx` CLI
  installed and signed in (`sbx login`), version 0.39.0+
- This repository published at `github.com/nikcio/docker-sandboxing` (kits
  are fetched from there by default; the repo must be **public** for
  `git+https`, or use `git+ssh` / `-Source local` from a clone)
- A [Zeldoc.ai](https://zeldoc.ai) API key
  ([setup guide](https://docs.zeldoc.ai/connect-opencode))

## Quick start

Run the bootstrap script — it flips the host settings, builds/loads the
template, registers your Zeldoc key, registers the `sbx-new` shell function,
and validates the kit and every mixin:

PowerShell:

```powershell
$env:ZELDOC_API_KEY = "your-zeldoc-key"
./scripts/bootstrap.ps1
```

Bash:

```bash
ZELDOC_API_KEY=your-zeldoc-key ./scripts/bootstrap.sh
```

What it does:

| Step | Command | Why |
| ---- | ------- | --- |
| Allow clipboard image paste | `sbx settings set clipboard.imagePaste true` | lets the sandboxed agent read images you paste (host-side setting) |
| Allow the kit source | merges `github.com/nikcio/` into `kit.allowedSources` | kits/mixins are fetched from this GitHub repo (list is merged, never overwritten) |
| Build + load template | `docker build` → `docker image save` → `sbx template load` | bakes .NET/Node/PNPM/Git into the image; no per-sandbox installs |
| *(or push)* | `PUSH_REGISTRY=docker.io/myorg ./scripts/bootstrap.sh` | share the template; then update `sandbox.image` in `kit/spec.yaml` |
| Register Zeldoc key | `sbx secret set zeldoc` (+ pre-creates the credential binding) | proxy substitutes the real key on `api.zeldoc.ai` requests; the sandbox only sees a placeholder |
| Register `sbx-new` | appends a function to your PowerShell profile / `~/.bashrc` | configurable alias for creating sandboxes (skip: `-SkipAlias` / `SKIP_ALIAS=1`) |
| Validate kits | `sbx kit validate kit/` + every `mixins/<area>/` | catches spec errors early |

Then launch a sandbox for any project (from a **new** shell so `sbx-new` is
loaded):

```bash
sbx-new /path/to/project
```

## The `sbx-new` launcher

`sbx-new` (registered by the bootstrap script; wraps `scripts/new-sandbox.*`)
creates the sandbox with a configurable mixin stack and attaches. If a
sandbox with the same name already exists it re-attaches instead (kits only
apply at creation).

```bash
sbx-new <workspace>                      # full profile, kits from GitHub
sbx-new --profile node <workspace>       # node-only mixin set
sbx-new --mixins zeldoc,git,node <ws>    # explicit mixin list
sbx-new --source local <workspace>       # use the local clone instead of GitHub
sbx-new --detach <workspace>             # create without attaching
sbx-new --list-profiles                  # show profiles
```

Profiles: `full` (default: all mixins), `node`, `dotnet`, `node-docker`,
`none`. Persist your own defaults via environment variables:

| Variable | Default | Meaning |
| -------- | ------- | ------- |
| `SBX_SANDBOX_PROFILE` | `full` | preset mixin set |
| `SBX_SANDBOX_MIXINS` | from profile | comma-separated override |
| `SBX_SANDBOX_SOURCE` | `git` | `git` (fetch from GitHub) or `local` (use the clone) |
| `SBX_SANDBOX_REPO` | `nikcio/docker-sandboxing` | GitHub repo to fetch kits from |
| `SBX_SANDBOX_REF` | unset | pin kits to a branch/tag/commit |
| `SBX_SANDBOX_REPO_DIR` | repo root | local repo dir for `--source local` |

Equivalent flags (`-Profile`, `-Mixins`, `-Source`, `-Repo`, `-Ref`,
`-RepoDir`, `-Name`, `-Detach`) win over the environment.

Under the hood it composes:

```bash
sbx run \
  --kit "git+https://github.com/nikcio/docker-sandboxing.git#dir=kit" \
  --kit "git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/opencode-runtime" \
  --kit "git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/zeldoc" \
  ... opencode-node-dotnet <workspace>
```

Each mixin is optional; drop the lines for areas a project doesn't need.

## How the Zeldoc setup works

Per the [Zeldoc connect guide](https://docs.zeldoc.ai/connect-opencode), the
provider is declared in its own config file instead of the sandbox-managed
`~/.config/opencode/opencode.json`:

1. `mixins/zeldoc/files/.../zeldoc.jsonc` defines the `zeldoc` provider
   (`api.zeldoc.ai/v1`, model `zdev`), sets `model: zeldoc/zdev`, disables
   sharing and the default `opencode` provider, and denies `websearch`.
2. The mixin sets `OPENCODE_CONFIG=/home/agent/.config/opencode/zeldoc.jsonc`.
   OpenCode merges this between the global and project config layers, so
   sandbox-managed wiring (e.g. the MCP gateway) keeps working.
3. The mixin's `credentials` block declares the `zeldoc` service
   (`ZELDOC_API_KEY`, proxy-managed) and injects it as `Authorization: Bearer …`
   on `api.zeldoc.ai` requests. The value inside the sandbox is a placeholder;
   the real key lives in the host secret store (`sbx secret set zeldoc`).
   Because this is a third-party v2 kit, the credential also needs a one-time
   **binding approval** — `sbx` prompts for it on the first interactive
   `sbx run`, or the bootstrap script pre-creates it in
   `%APPDATA%\sbx\credentials.yaml` (Linux/macOS: `~/.config/sbx/credentials.yaml`).

If ZDev ever reports "encountered an error", the model's context limit likely
changed — update `"limit".context` in `zeldoc.jsonc` per the
[Zeldoc guide](https://docs.zeldoc.ai/connect-opencode) (currently `1000000`).

> Note: because the sandbox kit extends the built-in `opencode` agent, it
> inherits the builtin provider credentials (anthropic, github, openai, …).
> Third-party kits don't carry builtin provenance, so `sbx` notes at creation
> that those credentials aren't injected until you approve bindings for them.
> That's harmless here — Zeldoc is the configured provider — but if you want
> another provider too, approve its binding (interactively on first run, or
> in `credentials.yaml`) and store its key with `sbx secret set <service>`.

## Network allowlist

The sandbox network policy is deny-by-default; the union of every composed
mixin's `permissions.network.allow` is the only egress. Domains per mixin:

| Mixin | Hosts |
| ----- | ----- |
| `zeldoc` | `api.zeldoc.ai`, `zeldoc.ai`, `docs.zeldoc.ai` |
| `opencode-runtime` | `opencode.ai`, `models.dev`, `console.anomaly.co`, `*.anomaly.co`, `registry.npmjs.org` (plugins) |
| `node` | `nodejs.org`, `*.nodejs.org`, `registry.npmjs.org`, `*.npmjs.org`, `npmjs.com` |
| `git` | `github.com`, `*.github.com`, `*.githubusercontent.com`, `gitlab.com` (bare hosts → git over SSH works) |
| `dotnet` | `nuget.org`, `*.nuget.org`, `*.microsoft.com`, `dot.net`, `*.dot.net`, `*.azureedge.net` |
| `docker` | `docker.io`, `*.docker.io`, `*.docker.com`, `production.cloudflare.docker.com`, `ghcr.io` |
| `apt` | `archive.ubuntu.com`, `security.ubuntu.com`, `packages.microsoft.com`, `*.launchpadcontent.net` |

`dotnet` also denies `*.applicationinsights.azure.com`; deny rules win over
allow rules. A sandbox composed without a mixin simply has no egress for that
area.

**If a download fails inside the sandbox**, check `sbx policy log` for the
blocked host, add it to the owning mixin's `spec.yaml`, and recreate the
sandbox — kit changes never apply to running sandboxes:

```bash
sbx rm <sandbox-name>
sbx-new <project>
```

## Using it from a project (`.sbxenv.yaml`)

See `examples/opencode-node-dotnet.sbxenv.yaml`. Copy it into a directory
*next to* (not inside) your project, adjust the workspace path, then:

```bash
sbx env run
```

## Iterating

- Mixin/sandbox kit spec changes → `sbx kit validate <dir>`, push, then
  recreate the sandbox (`sbx rm <name>` + `sbx-new`). While iterating from a
  clone, use `sbx-new --source local` to pick up uncommitted changes without
  pushing. While iterating on mixin-limited fields (`environment.variables`,
  `setup.install`, `permissions.network.allow`), `sbx kit add <sandbox> <dir>`
  restarts an existing sandbox with the new kit set.
- Template changes → re-run the bootstrap script (build + save + load), then
  recreate the sandbox.
- `sbx template ls` lists loaded templates; `sbx template rm` removes one.

## Publishing

- **Push this repo** to `github.com/nikcio/docker-sandboxing` — that's where
  `sbx-new` fetches kits from by default. For reproducible launches, pin a
  tag: `SBX_SANDBOX_REF=v1.0.0 sbx-new <workspace>` (or pass `--ref`).
- **OCI alternative**: pack/push each directory (`sbx kit pack kit/ -o …`,
  `sbx kit push mixins/zeldoc/ <oci-ref>`, …). Any non-Docker-Hub kit source
  must be allow-listed:
  `sbx settings set kit.allowedSources '["docker.io/","github.com/nikcio/"]'`.
- **Template**: build with `-PushRegistry` / `PUSH_REGISTRY` and update
  `sandbox.image` in `kit/spec.yaml` to the full registry reference (sbx does
  not resolve the Docker Hub domain automatically). Consumers of other
  registries need `sbx secret set --registry <host>` credentials.

## Security notes

- The Zeldoc API key is registered host-side only; sandboxes see a
  placeholder and the proxy rewrites the auth header.
- Never commit secrets, `dist/`, `*.tar`, `*.zip`, or `local.sbxenv.yaml`
  (see `.gitignore`).
- Kit install commands run with root privileges inside the sandbox — these
  kits intentionally have none; all tooling is baked into the template image.
