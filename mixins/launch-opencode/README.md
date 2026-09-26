# launch-opencode

Launches the OpenCode agent at sandbox startup. Compose it with the
`opencode` mixin: the install hook writes the agent shim
(`/usr/local/bin/agent-launch` — `exec opencode`) and the workload's
bash entrypoint execs it; without this mixin the workload launches a
plain shell.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/launch-opencode&ref=vX.Y.Z # launch opencode at startup
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/launch-opencode <workload> .`

Requires the `opencode` mixin — the shim execs the `opencode` binary,
which that mixin installs (without it the sandbox stops immediately
with "opencode: not found").

## How it works

- **Install hook**: writes `/usr/local/bin/agent-launch`
  (`#!/bin/sh` + `exec opencode`, root-owned, mode 755) once, at
  creation, before the workload entrypoint runs.
- **Startup hook**: re-creates the shim on later boots if it went
  missing (best-effort, root) — container restarts preserve the
  filesystem, so this is a safety net only.
- **Entrypoint contract**: the workload kits' entrypoint execs the shim
  when it exists and falls back to plain bash otherwise — the same
  workload image serves shell-only and OpenCode compositions.
