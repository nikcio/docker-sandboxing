# docker-sandboxing

A [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/) **template** and
**kit** that give [OpenCode](https://opencode.ai) a full-stack dev environment:

- **Template** (`template/Dockerfile` → `opencode-node-dotnet:v1`)
  - .NET SDK (LTS, `dotnet`)
  - Node.js LTS via [NVM](https://github.com/nvm-sh/nvm) (`nvm`, `node`)
  - [PNPM](https://pnpm.io) (`pnpm`)
  - Git (+ git-lfs) and common CLI utilities
  - Extends the built-in `docker/sandbox-templates:opencode-docker` image
- **Kit** (`kit/`, `kind: sandbox`, `extends: opencode`)
  - Points at the template via `sandbox.image`
  - Wires **Zeldoc.ai** (`zdev`) in as the model provider via a proxy-managed
    API key — the key never enters the sandbox VM
  - Declares the network allowlist the agent needs (the sandbox runs with a
    deny-by-default network policy)
  - Injects a base `AGENTS.md` that mandates the git-worktree workflow

## Repo layout

```text
├── AGENTS.md                                  # base agent instructions (worktrees + repo rules)
├── template/
│   └── Dockerfile                             # sandbox template image
├── kit/
│   ├── spec.yaml                              # kit spec (schemaVersion "2")
│   └── files/home/.config/opencode/zeldoc.jsonc
│                                              # Zeldoc provider config (OPENCODE_CONFIG)
├── scripts/
│   ├── bootstrap.ps1                          # host setup (Windows)
│   └── bootstrap.sh                           # host setup (Linux/macOS/Git Bash)
└── examples/
    └── opencode-node-dotnet.sbxenv.yaml       # project .sbxenv.yaml example
```

## Prerequisites

- [Docker Desktop](https://docs.docker.com/desktop/) with the `sbx` CLI
  installed and signed in (`sbx login`), version 0.39.0+
- A [Zeldoc.ai](https://zeldoc.ai) API key
  ([setup guide](https://docs.zeldoc.ai/connect-opencode))

## Quick start

Run the bootstrap script — it flips the host setting, builds/loads the
template, registers your Zeldoc key, and validates the kit:

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
| Build + load template | `docker build` → `docker image save` → `sbx template load` | bakes .NET/Node/PNPM/Git into the image; no per-sandbox installs |
| *(or push)* | `PUSH_REGISTRY=docker.io/myorg ./scripts/bootstrap.sh` | share the template; then update `sandbox.image` in `kit/spec.yaml` |
| Register Zeldoc key | `sbx secret set zeldoc` (+ pre-creates the credential binding) | proxy substitutes the real key on `api.zeldoc.ai` requests; the sandbox only sees a placeholder |
| Validate kit | `sbx kit validate kit/` | catches spec errors early |

Then launch a sandbox for any project:

```bash
sbx run --kit /path/to/docker-sandboxing/kit opencode-node-dotnet /path/to/project
```

## How the Zeldoc setup works

Per the [Zeldoc connect guide](https://docs.zeldoc.ai/connect-opencode), the
provider is declared in its own config file instead of the sandbox-managed
`~/.config/opencode/opencode.json`:

1. `kit/files/.../zeldoc.jsonc` defines the `zeldoc` provider
   (`api.zeldoc.ai/v1`, model `zdev`), sets `model: zeldoc/zdev`, disables
   sharing and the default `opencode` provider, and denies `websearch`.
2. The kit sets `OPENCODE_CONFIG=/home/agent/.config/opencode/zeldoc.jsonc`.
   OpenCode merges this between the global and project config layers, so
   sandbox-managed wiring (e.g. the MCP gateway) keeps working.
3. The kit's `credentials` block declares the `zeldoc` service
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

## Network allowlist

The sandbox network policy is deny-by-default; the kit's
`permissions.network.allow` is the only egress. Domains covered:

| Purpose | Hosts |
| ------- | ----- |
| Zeldoc.ai | `api.zeldoc.ai`, `zeldoc.ai`, `docs.zeldoc.ai` |
| OpenCode runtime | `opencode.ai`, `models.dev`, `console.anomaly.co`, `*.anomaly.co` |
| npm / pnpm / npx | `registry.npmjs.org`, `*.npmjs.org`, `npmjs.com` |
| Git hosting | `github.com`, `*.github.com`, `*.githubusercontent.com`, `gitlab.com` (bare hosts → git over SSH works) |
| .NET / NuGet | `nuget.org`, `*.nuget.org`, `*.microsoft.com`, `dot.net`, `*.dot.net`, `*.azureedge.net` |
| Node / NVM | `nodejs.org`, `*.nodejs.org` |
| In-sandbox Docker | `docker.io`, `*.docker.io`, `*.docker.com`, `production.cloudflare.docker.com`, `ghcr.io` |
| apt | `archive.ubuntu.com`, `security.ubuntu.com`, `packages.microsoft.com`, `*.launchpadcontent.net` |

Telemetry is explicitly denied (`*.applicationinsights.azure.com`); deny rules
win over allow rules.

**If a download fails inside the sandbox**, check `sbx policy log` for the
blocked host, add it to `kit/spec.yaml`, and recreate the sandbox — kit
changes never apply to running sandboxes:

```bash
sbx rm <sandbox-name>
sbx run --kit ./kit opencode-node-dotnet <project>
```

## Using it from a project (`.sbxenv.yaml`)

See `examples/opencode-node-dotnet.sbxenv.yaml`. Copy it into a directory
*next to* (not inside) your project, adjust the workspace path, then:

```bash
sbx env run
```

## Iterating

- Kit spec changes → `sbx kit validate kit/`, then recreate the sandbox.
  While iterating on mixin-limited fields, `sbx kit add <sandbox> ./kit`
  restarts an existing sandbox with the new kit set.
- Template changes → re-run the bootstrap script (build + save + load), then
  recreate the sandbox.
- `sbx template ls` lists loaded templates; `sbx template rm` removes one.

## Publishing

- **Kit**: `sbx kit pack kit/ -o opencode-node-dotnet.zip`, `sbx kit push kit/ <oci-ref>`,
  or serve it from a Git repository
  (`sbx run --kit "git+https://host/repo.git#dir=kit" …`). If you load kits
  from a remote source, allow it first:
  `sbx settings set kit.allowedSources '["docker.io/","github.com/<org>/"]'`.
- **Template**: build with `-PushRegistry` / `PUSH_REGISTRY` and update
  `sandbox.image` in `kit/spec.yaml` to the full registry reference (sbx does
  not resolve the Docker Hub domain automatically). Consumers of other
  registries need `sbx secret set --registry <host>` credentials.

## Security notes

- The Zeldoc API key is registered host-side only; sandboxes see a
  placeholder and the proxy rewrites the auth header.
- Never commit secrets, `dist/`, `*.tar`, `*.zip`, or `local.sbxenv.yaml`
  (see `.gitignore`).
- Kit install commands run with root privileges inside the sandbox — this kit
  intentionally has none; all tooling is baked into the template image.
