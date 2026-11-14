# mixins-v3 — the same mixins in the v3 kit format

This tree mirrors every `mixins/<name>` mixin in the
[Docker Sandboxes v3 kit format](https://docs.docker.com/ai/sandboxes/customize/author/)
(`schemaVersion: "3"`, `# syntax=docker/sandbox-kit:3`). The v2 tree
(`mixins/`) remains the primary one: built-in agent shortcuts (`claude`,
`codex`, …) and the existing examples consume v2, and v2 and v3 cannot be
mixed in one sandbox.

Each mixin keeps its v2 sibling's purpose, arguments, network domains, and
behavior. See the sibling's `../<name>/README.md` for the full story; this
section covers only what changes in v3.

## What changes from v2

| v2 (`mixins/`) | v3 (`mixins-v3/`) |
| -------------- | ----------------- |
| `spec.yaml` descriptor (`schemaVersion: "2"`) | `<name>.yaml` descriptor (`schemaVersion: "3"`) + optional `<name>.dockerfile` content recipe |
| `setup.install` steps run as hooks at sandbox creation (network + apt every recreate) | installs run as Dockerfile `RUN` at image build, baked into the mixin layer; every sandbox starts with the toolchain present |
| `permissions.network.allow` (v2 grammar) | `com.docker.sandbox/network-policy@1` capability (`runtime:` phase; `install:` only where a hook still needs egress) |
| `credentials:` block | `com.docker.sandbox/credential@1` capability (same `service`/`apiKey` shape) |
| `setup.startup` / `setup.files` | `com.docker.sandbox/lifecycle@1` capability (`startup:` / `files:`; hook `env` is deny-by-default — needed variables are declared) |
| `environment.variables` | image `ENV` (or a profile script for PATH-style additions) |
| version pinning at create | version args are `buildArg`s, resolved at image build |

Two structural notes:

- **Mixin content lives under `/opt`**, not the agent home: a v3 mixin's
  layers are an overlay on an unknown base, so its content must not claim
  `/home/agent` paths the base or the agent user owns. Profile scripts
  under `/etc/profile.d/` put the tools on PATH for login shells.
- **`provides` is unversioned** for installs that resolve "latest" at
  build: the exact version isn't knowable ahead of the build, so the
  descriptor states the primary tool name and a `version:` fallback
  (an unversioned provide only matches unconstrained requires).

## Build and use

Build a mixin (validated by the kit frontend; capabilities only apply when
the sandbox composes it):

```bash
docker buildx build -f mixins-v3/node/node.yaml -t my/node-mixin:dev mixins-v3/node/
```

Use a local directory in a v3 sandbox (`sbx` builds it at create):

```bash
sbx run <agent> . --kit ./mixins-v3/node
```

After publishing to a registry, use the image reference instead. Compose
mixins with a v3 workload (e.g. a kit you author on the spec repo's
`opencode` workload example); see
[Kit sets](https://docs.docker.com/ai/sandboxes/customize/author/kit-sets/).

## Mixins

Same set as the v2 tree: `docker-hub`, `dotnet`, `env-guard`, `gcr`,
`ghcr`, `github-cli`, `go`, `herdr`, `mcr`, `nikcio-openapi-codegen`,
`node`, `omnium`, `open-egress`, `openapi-ts`, `opencode`,
`opencode-copilot`, `opencode-zeldoc`, `playwright`, `python`, `repo-clone`,
`rust`, `sbx`, `uniform`, `zsh`.
