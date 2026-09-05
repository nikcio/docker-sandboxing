# docker-sandboxing

A [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/) **template** and
composable **kits** that give [OpenCode](https://opencode.ai) a full-stack dev
environment:

- **Template** (`template/Dockerfile` → `opencode-node-dotnet:v1`)
  - .NET SDK (LTS, `dotnet`)
  - Node.js LTS via [NVM](https://github.com/nvm-sh/nvm) (`nvm`, `node`)
  - [PNPM](https://pnpm.io) (`pnpm`)
  - Git (+ git-lfs), GitHub CLI (`gh`), and common CLI utilities
  - `o` — PATH shim that (re)launches opencode from any shell
  - Extends the built-in `docker/sandbox-templates:opencode-docker` image
- **Sandbox kit** (`kit/`, `kind: sandbox`, `extends: opencode`) — thin agent
  definition: points at the template via `sandbox.image`, sets the entrypoint
  (a login shell that prints a startup banner, auto-starts opencode, and
  drops into the shell when opencode quits — exiting that shell ends the
  session), and drops a **permissive OpenCode config** (edit/bash/webfetch
  allowed — the sandbox is the isolation boundary). No network rules of its
  own.
- **Mixins** (`mixins/<area>/`, `kind: mixin`) — each defines the rules for
  exactly one area and stacks via `--kit` or a `.sbxenv.yaml`:

  | Mixin | Area | Provides |
  | ----- | ---- | -------- |
  | `zeldoc` | model provider | Zeldoc.ai credential (proxy-managed key), provider config (`OPENCODE_CONFIG`), Zeldoc hosts |
  | `git` | git workflow | git hosting egress (HTTPS + SSH), proxy-managed GitHub auth for `gh`/git-over-HTTPS (scoped PAT), the mandatory worktree workflow + gh guidance in agent memory |
  | `node` | Node toolchain | nodejs.org + npm registry egress, nvm/pnpm notes |
  | `dotnet` | .NET toolchain | NuGet/Microsoft egress, telemetry opt-out, telemetry deny |
  | `docker` | containers | registry egress for the Docker engine inside the sandbox |
  | `opencode-runtime` | agent runtime | opencode.ai/models.dev egress, npm-hosted plugins |
  | `apt` | OS packages | Ubuntu/Microsoft package mirrors for `sudo apt-get` |
  | `browser` | browser software | Google Chrome install + agent notes (no network rules — the composed mixins own all egress) |
  | `playwright` | browser automation | Playwright (npm global) + Chromium headless shell, Playwright CDN egress |
  | `playwright-chromium` | browser automation | Playwright + the full Chromium build |
  | `playwright-all` | browser automation | Playwright + Chromium, Firefox, and WebKit (~1 GB+ download) |
  | `sbx` | sandbox tooling | `docker-sbx` CLI from Docker's apt repo — kit authoring inside the sandbox (`sbx kit validate/inspect/pack`); not in the `full` profile |

  Drop the mixins you don't need — e.g. a pure Node project skips `dotnet`,
  `docker`, and `apt`.

  The `playwright*` mixins and `sbx` run apt at creation (OS libraries /
  Docker's repo) — compose the `apt` mixin with them (the `full` and
  `browser` profiles already do).

  Mixin memory notes are static files (`mixins/<area>/files/home/.sbx-agents.d/<area>.md`)
  that the kit entrypoint appends directly to the sandbox `AGENTS.md` when the
  agent launches.

## Repo layout

```text
├── AGENTS.md                                  # base agent instructions (worktrees + repo rules)
├── template/
│   └── Dockerfile                             # sandbox template image
├── kit/
│   ├── spec.yaml                              # thin sandbox kit (template + entrypoint)
│   └── files/home/.config/opencode/opencode.jsonc
│                                              # permissive OpenCode config (global layer)
├── mixins/
│   ├── zeldoc/                                # provider credential + config + network
│   │   ├── spec.yaml
│   │   └── files/home/.config/opencode/zeldoc.jsonc
│   ├── git/                                   # git egress + gh auth + worktree memory
│   ├── node/                                  # node/npm egress
│   ├── dotnet/                                # nuget/microsoft egress + telemetry deny
│   ├── docker/                                # registry egress
│   ├── opencode-runtime/                      # agent runtime egress
│   ├── apt/                                   # package mirror egress
│   ├── browser/                               # Google Chrome install (software only)
│   ├── playwright/                            # Playwright + Chromium headless shell
│   ├── playwright-chromium/                   # Playwright + full Chromium
│   ├── playwright-all/                        # Playwright + Chromium/Firefox/WebKit
│   └── sbx/                                   # sbx CLI install (kit authoring in-sandbox)
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
  are fetched from there by default). The repo can be public or private —
  you just need access to it
- A [Zeldoc.ai](https://zeldoc.ai) API key
  ([setup guide](https://docs.zeldoc.ai/connect-opencode))
- Optional: a fine-grained [GitHub personal access token](#github-cli--a-scoped-personal-access-token)
  so the agent can use `gh` and push over HTTPS (public repos and SSH agent
  forwarding work without one)

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
| Register Zeldoc key | `sbx secret set zeldoc` (+ pre-creates the credential binding) — skipped if already stored; an env var always re-registers | proxy substitutes the real key on `api.zeldoc.ai` requests; the sandbox only sees a placeholder |
| Register GitHub token *(optional)* | `sbx secret set github` (prompted, or `GITHUB_PAT`; empty input skips) — skipped if already stored; an env var always re-registers. Pre-creates the credential binding | proxy substitutes the real token on GitHub requests; the sandbox only sees a placeholder |
| Register `sbx-new` | appends a function to your PowerShell profile / `~/.bashrc` | configurable alias for creating sandboxes (skip: `-SkipAlias` / `SKIP_ALIAS=1`) |
| Validate kits | `sbx kit validate kit/` + every `mixins/<area>/` | catches spec errors early |

Then launch a sandbox for any project (from a **new** shell so `sbx-new` is
loaded):

```bash
sbx-new
```

## The sandbox shell

Attaching lands you in opencode directly — a startup banner explains the
flow. Quitting opencode does **not** end the session: you drop into the
sandbox's login shell (with the startup banner above it), where you can run
git, builds, `dotnet`/`pnpm`, etc. Relaunch opencode anytime with `o`; exit
the shell when you're done with the sandbox.

## The `sbx-new` launcher

`sbx-new` (registered by the bootstrap script; wraps `scripts/new-sandbox.*`)
is a **wizard**: run it with no arguments and it guides you through

1. **Workspace** — project directory (created if it doesn't exist yet)
2. **Mixin profile** — `full`, `node`, `dotnet`, `node-docker`, `browser`,
   `none`, or `custom` (pick individual mixins; each is shown with a description)
3. **Kit source** — fetch from `github.com/nikcio/docker-sandboxing`
   (recommended, optionally pinned to a branch/tag) or use the local clone
4. **Sandbox name** — default `opencode-node-dotnet-<workspace>`; if the name
   already exists you can attach to it, rename, or cancel
5. **Launch mode** — create & attach, or create only
6. **Summary** — the full resolved configuration before anything runs

Flags are optional — they skip the wizard for scripted use:

```bash
sbx-new <workspace>                      # full profile, kits from GitHub
sbx-new --profile node <workspace>       # node-only mixin set
sbx-new --mixins zeldoc,git,node <ws>    # explicit mixin list
sbx-new --source local <workspace>       # use the local clone instead of GitHub
sbx-new --detach <workspace>             # create without attaching
sbx-new --yes                            # no prompts, pure defaults/env
sbx-new --list-profiles                  # show profiles
```

Profiles: `full` (default: all mixins), `node`, `dotnet`, `node-docker`,
`browser`, `none`. Persist your own defaults via environment variables:

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
3. The base kit drops a permissive `opencode.jsonc` into the global config
   layer (`~/.config/opencode`) — the sandbox-managed `opencode.json` (MCP
   gateway) is a separate file and both are merged. Config layering, lowest
   to highest: managed `opencode.json` → kit's permissive `opencode.jsonc`
   → zeldoc's `OPENCODE_CONFIG` → project config.
4. The mixin's `credentials` block declares the `zeldoc` service
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

## GitHub CLI + a scoped personal access token

The template ships [`gh`](https://cli.github.com), and the `git` mixin wires
its authentication through the sandbox's proxy-managed credential flow: the
sandbox boots with `GH_TOKEN=proxy-managed` (a placeholder), and the
host-side proxy swaps in the real token on requests to
`api.github.com`, `github.com`, `uploads.github.com`, and
`raw.githubusercontent.com`. The token itself never enters the sandbox VM —
`gh auth status` inside the sandbox only ever sees the placeholder, which is
expected.

Because the token is what the agent acts with, keep its blast radius small:
use a **fine-grained PAT scoped to just the repositories the agent should
touch**, not your host `gh` login.

1. On GitHub: **Settings → Developer settings → Personal access tokens →
   Fine-grained tokens → Generate new token**.
2. Resource owner: you (or the org that owns the repos). Expiration per your
   policy (e.g. 90 days).
3. **Repository access → Only select repositories** — pick the repositories
   the agent works on. This is the main safety lever: everything not listed
   is invisible to the agent.
4. Repository permissions — start from this minimum:
   - **Metadata: Read** (mandatory, set automatically)
   - **Contents: Read and write** — clone, commit, push, tags
   - **Pull requests: Read and write**
   - **Issues: Read and write** *(optional)*
   - **Actions: Read** *(optional — view CI status)*
5. Deliberately leave out anything the agent doesn't need:
   - **Workflows: leave unset** — without it the agent cannot push changes
     to `.github/workflows/`, i.e. cannot alter your CI.
   - No **Administration**, no **Secrets**, no org-wide permissions.

Register it host-side (the bootstrap script does this too — prompted, or set
`GITHUB_PAT`; empty input skips it):

```bash
sbx secret set github
# non-interactive:
printf '%s\n' "github_pat_..." | sbx secret set github
```

Third-party v2 kits also need a one-time **credential binding approval** for
`github` (same mechanism as `zeldoc`): approve it interactively on the first
`sbx run`, or pre-create it in `%APPDATA%\sbx\credentials.yaml`
(Linux/macOS: `~/.config/sbx/credentials.yaml`) — the bootstrap script
writes it:

```yaml
bindings:
  github:
    apiKey:
      domains: [api.github.com, github.com, uploads.github.com, raw.githubusercontent.com]
```

Notes:

- Without a stored token the `git` mixin still works for **public**
  repositories and for **git over SSH** (the sandbox forwards your host SSH
  agent; private keys stay on the host).
- If you'd rather reuse your host `gh` login instead of a scoped PAT:
  `sbx secret set github --command 'gh auth token'` — but that token carries
  whatever scopes your CLI has (`repo`, `workflow`, `read:org`, …). The
  scoped PAT is the recommended path.
- Rotate or revoke the PAT from GitHub anytime (`sbx secret rm github` to
  forget it host-side). Recreate a running sandbox after changing a global
  secret for it to take effect.

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
| `opencode-runtime` | `opencode.ai`, `models.dev`, `registry.npmjs.org` (plugins) |
| `node` | `nodejs.org`, `*.nodejs.org`, `registry.npmjs.org`, `*.npmjs.org`, `npmjs.com` |
| `git` | `github.com`, `*.github.com`, `*.githubusercontent.com`, `gitlab.com` (bare hosts → git over SSH works) |
| `dotnet` | `nuget.org`, `*.nuget.org`, `*.microsoft.com`, `dot.net`, `*.dot.net`, `*.azureedge.net`, `*.digicert.com`, `*.symcd.com`, `*.symcb.com`, `*.ws.symantec.com` (CA OCSP/CRL + timestamp checks) |
| `docker` | `docker.io`, `*.docker.io`, `*.docker.com`, `production.cloudflare.docker.com`, `ghcr.io` |
| `apt` | `archive.ubuntu.com`, `security.ubuntu.com`, `packages.microsoft.com`, `*.launchpadcontent.net` |
| `browser` | *(none — software only; every site stays gated by the composed mixins)* |
| `playwright` / `playwright-chromium` / `playwright-all` | `registry.npmjs.org`, `*.npmjs.org`, `cdn.playwright.dev`, `*.cdn.playwright.dev`, `playwright.azureedge.net` |
| `sbx` | `download.docker.com` |

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
  `sbx-new` fetches kits from by default. The repo can be public or
  private; you just need access to it. For reproducible launches, pin a
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
- The GitHub token is the same deal (`GH_TOKEN` holds a placeholder in the
  sandbox; the proxy injects the real token only on the hosts the `git`
  mixin declares). Use a fine-grained PAT scoped to the repos the agent
  should reach — see
  [GitHub CLI + a scoped personal access token](#github-cli--a-scoped-personal-access-token).
- Never commit secrets, `dist/`, `*.tar`, `*.zip`, or `local.sbxenv.yaml`
  (see `.gitignore`).
- Kit install commands run with root privileges inside the sandbox — these
  kits intentionally have none; all tooling is baked into the template image.
