# env-guard (v3)

Enforces the no-`.env` policy in the workspace: secrets in `.env` files
must never reach the sandbox, where the agent can read them.

Declaration-only: no image content, no build args, nothing to build. The
guard is a `com.docker.sandbox/lifecycle@1` startup hook that runs
before the agent starts, on every boot.

## Build / use

Point a sandbox at the mixin directory (`sbx` composes it at create):

```bash
sbx run <agent> . --kit ./mixins-v3/env-guard
```

In a project's `sbxenv.yaml`, add it to the `kits:` list (pin
`&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/env-guard   # workspace .env guard
```

Optional but composed by the examples.

## How it works

- **Clone mode** (`workspace.clone: true`): `.env` files are removed
  from the sandbox-local copy, with a warning and a short pause.
- **Direct mode** (`workspace.clone: false`, the workspace tree mounted
  in place): the sandbox refuses to start when a `.env` file exists —
  deleting host files is not an option.
- On the all-clear, a short line reports the result.

What counts: the literal name `.env` plus the dotenv variants loaders
pick up (`.env.local`, `.env.production`, ...), except
`.env.example`/`.env.sample`/`.env.template`, and nothing under `.git/`.
Symlinks count too, since a committed `.env` symlink to a host-side file
would otherwise slip past a plain file check and be read by the agent.
The clone-mode deletion removes the link only, never the file it points
at. No-op when there is no workspace (`WORKSPACE_DIR` unset).

No network rules; capability egress comes from the other mixins.
