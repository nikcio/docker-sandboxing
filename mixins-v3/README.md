# mixins-v3 — the premade mixins

The premade mixins in this repo live here, in the
[Docker Sandboxes v3 kit format](https://docs.docker.com/ai/sandboxes/customize/author/)
(`schemaVersion: "3"`, `# syntax=docker/sandbox-kit:3`). Each mixin adds
exactly one capability area to the sandbox — a stack toolchain, a tool, a
model provider, network egress for a service — and composes with the others
in the `kits:` list of an `sbxenv.yaml`.

## How a mixin is put together

- **`<name>.yaml`** — the descriptor (`schemaVersion: "3"`, `kind: mixin`):
  metadata, `args` (build arguments for version pinning), `provides`, and
  the `capabilities`:
  - `com.docker.sandbox/network-policy@1` — outbound egress, split into a
    `runtime:` phase (what the tool needs while the sandbox runs) and an
    `install:` phase (only where a hook still downloads at create).
  - `com.docker.sandbox/credential@1` — proxy-managed credentials (same
    `service`/`apiKey` shape as the platform credential model).
  - `com.docker.sandbox/lifecycle@1` — `startup:` and `files:` hooks; hook
    `env` is deny-by-default, so needed variables are declared explicitly.
- **`<name>.dockerfile`** (optional) — the content recipe. Installs run as
  Dockerfile `RUN` at image build and are baked into the mixin layer, so
  every sandbox composed with the mixin starts with the tool present and
  the create-phase network grant stays small.
- **`README.md`** — purpose, usage, build args, and the egress rules with
  the reasoning behind each domain.

Two structural notes for kit authors:

- **Mixin content lives under `/opt`**, not the agent home: a v3 mixin's
  layers are an overlay on an unknown base, so its content must not claim
  `/home/agent` paths the base or the agent user owns. Profile scripts
  under `/etc/profile.d/` put the tools on `PATH` for login shells.
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

Use a local directory in a sandbox (`sbx` builds it at create):

```bash
sbx run <agent> . --kit ./mixins-v3/node
```

Or compose it from this repo by git, pinning a release tag like the
[examples](../examples) do:

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/node&ref=v3.1.2
```

After publishing to a registry, use the image reference instead. Compose
mixins with a v3 workload (e.g. a kit you author on the spec repo's
`opencode` workload example); see
[Kit sets](https://docs.docker.com/ai/sandboxes/customize/author/kit-sets/).

## Mixins

`docker-hub`, `dotnet`, `env-guard`, `gcr`, `ghcr`, `github-cli`, `go`,
`herdr`, `mcr`, `nikcio-openapi-codegen`, `node`, `omnium`, `open-egress`,
`openapi-ts`, `opencode`, `opencode-copilot`, `opencode-zeldoc`,
`playwright`, `python`, `repo-clone`, `rust`, `sbx`, `uniform`, `zsh`.

The full catalog with one-line summaries lives in
[docs/mixins.md](../docs/mixins.md); each `mixins-v3/<name>/README.md` has
the per-mixin details.
