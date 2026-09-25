# env-guard

Enforces the no-`.env` policy in the workspace: secrets in `.env` files
must never reach the sandbox, where the agent can read them.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - docker.io/nikcio/sbx-mixin-env-guard:vX.Y.Z # workspace .env guard
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/env-guard <workload> .`

Optional but composed by the [examples](../../examples). No network rules
— capability egress comes from the other mixins.

## How it works

The guard script ships in this mixin's image
(`/opt/sandbox/env-guard/env-guard.sh`) and runs as a lifecycle install
hook — once, at sandbox creation, after the workspace is mounted and
before the agent starts. That is the whole gating story: no entrypoint
shim, no race with the agent.

- **Clone mode** (`workspace.clone: true`): `.env` files — including
  `.env.local`-style variants and symlinked `.env` files — are removed
  from the sandbox-local copy, with a warning and a short pause.
- **Direct mode** (`workspace.clone: false`, the workspace tree mounted in
  place): the hook exits nonzero, which fails sandbox creation — delete
  the `.env` file(s) on the host and recreate.
- On the all-clear, a short line reports the result.
- No workspace (`WORKSPACE_DIR` unset): the check is skipped.

Symlinks count as files (deleting a symlink removes the link, never its
target), and `.env.example` / `.env.sample` / `.env.template` are allowed.
