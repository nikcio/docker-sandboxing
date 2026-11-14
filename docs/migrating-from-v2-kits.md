# Migrating kits to the v3 kit format

The v2 kit format (`schemaVersion: "2"`, the `mixins/` tree) is deprecated. The v3 kit format (`schemaVersion: "3"`, the `mixins-v3/` tree) replaces it: same mixins, same purpose and behavior, with installs baked into the mixin image instead of running as create-time hooks. The deprecated tree keeps working during the migration window but no longer receives new features — new mixins and changes land in `mixins-v3/` only.

Every mixin in `mixins/` has a counterpart of the same name in `mixins-v3/`: `docker-hub`, `dotnet`, `env-guard`, `gcr`, `ghcr`, `github-cli`, `go`, `herdr`, `mcr`, `nikcio-openapi-codegen`, `node`, `omnium`, `open-egress`, `openapi-ts`, `opencode`, `opencode-copilot`, `opencode-zeldoc`, `playwright`, `python`, `repo-clone`, `rust`, `sbx`, `uniform`, `zsh`.

## What changes between the formats

| v2 (`mixins/`, `schemaVersion: "2"`) | v3 (`mixins-v3/`, `schemaVersion: "3"`) |
| ------------------------------------ | --------------------------------------- |
| `spec.yaml` descriptor | `<name>.yaml` descriptor (`# syntax=docker/sandbox-kit:3`) + optional `<name>.dockerfile` content recipe |
| `setup.install` steps run as hooks at sandbox creation (network + apt needed on every recreate) | installs run as Dockerfile `RUN` at image build, baked into the mixin layer; every sandbox starts with the toolchain present |
| `permissions.network.allow` | `com.docker.sandbox/network-policy@1` capability (`runtime:` phase; `install:` only where a hook still needs egress) |
| `credentials:` block | `com.docker.sandbox/credential@1` capability (same `service`/`apiKey` shape) |
| `setup.startup` / `setup.files` | `com.docker.sandbox/lifecycle@1` capability (`startup:` / `files:`; hook `env` is deny-by-default — needed variables are declared) |
| `environment.variables` | image `ENV` in the dockerfile (or a profile script for PATH-style additions) |
| version pinning resolved at sandbox creation | version args are build arguments, resolved when the mixin image is built |

Two structural notes for kit authors:

- **Mixin content lives under `/opt`**, not the agent home: a v3 mixin's layers are an overlay on an unknown base, so its content must not claim `/home/agent` paths the base or the agent user owns. Profile scripts under `/etc/profile.d/` put the tools on `PATH` for login shells.
- **`provides` is unversioned** for installs that resolve "latest" at build: the exact version isn't knowable ahead of the build, so the descriptor states the primary tool name and a `version:` fallback (an unversioned provide only matches unconstrained requires).

## Migrate an environment file

Update each kit line to point at the v3 tree — the mixin name and the `&ref=` pin stay the same:

```yaml
# before
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/node&ref=v3.1.2
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/env-guard&ref=v3.1.2

# after
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/node&ref=v3.1.2
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/env-guard&ref=v3.1.2
```

- **Migrate every line at once.** A sandbox composes either all v2 kits or all v3 kits — the formats cannot be mixed in one sandbox.
- **Version arguments move into `args:` entries.** `args:` entries in `sbxenv.yaml` keep working; they are passed to the image build as build arguments now. If you pinned versions with `--kit-arg`, move the values into the `args:` entry on the mixin's `kits:` line. Pinning a version (or leaving it empty for latest) resolves when the mixin image is built instead of at sandbox creation.
- **Recreate the sandbox.** Kit changes only apply to new sandboxes: `sbx rm <name>` + `sbx env run`. The first create after migrating builds the mixin layers; later creates reuse them.

Behavior you will notice after migrating:

- The sandbox always starts with the toolchain present — the tools are in the mixin image, not installed at create.
- Create-time installs (and their apt-mirror/download egress inside the sandbox) are gone; at runtime a mixin grants exactly the egress its tools need.
- Local directory kits work the same: `sbx run <agent> . --kit ./mixins-v3/<name>` builds the kit at create.

## Migrate an in-project kit

Rename the descriptor and map each block to the matching capability:

| v2 `spec.yaml` block | v3 `<name>.yaml` block |
| -------------------- | ---------------------- |
| `schemaVersion: "2"` | `schemaVersion: "3"` + `# syntax=docker/sandbox-kit:3` |
| `permissions.network.allow` | `capabilities:` → `com.docker.sandbox/network-policy@1` (`runtime.allow`) |
| `environment.variables` | `dockerfile:` with image `ENV` entries |
| `setup.startup` / `setup.files` | `capabilities:` → `com.docker.sandbox/lifecycle@1` (`startup:` / `files:`) |
| `credentials:` | `capabilities:` → `com.docker.sandbox/credential@1` |

A typical project mixin only carries network rules, so the migration is small:

```yaml
# before: sandbox-kit/spec.yaml
schemaVersion: "2"
kind: mixin
name: my-project
version: 0.1.0
permissions:
  network:
    allow:
      - npm.mycompany.com:443
```

```yaml
# after: sandbox-kit/my-project.yaml
# syntax=docker/sandbox-kit:3
schemaVersion: "3"
kind: mixin
displayName: My project
version: 0.1.0

capabilities:
  - type: com.docker.sandbox/network-policy@1
    config:
      runtime:
        allow:
          - npm.mycompany.com:443
```

The `kits:` entry keeps pointing at the directory (`- ./sandbox-kit`); only the files inside change. Validate the migrated kit with a build — `docker buildx build -f sandbox-kit/my-project.yaml -t sandbox-kit:check sandbox-kit/` — then recreate the sandbox. A full walkthrough of the v3 in-project kit shape: [project-kit.md](project-kit.md).

## Where to go from here

- Mixin catalog and usage: [mixins.md](mixins.md)
- Per-mixin details: each `mixins-v3/<name>/README.md`
- Building and composing v3 kits locally: [mixins-v3 README](../mixins-v3/README.md)
