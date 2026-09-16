# env-guard

Enforces the no-`.env` policy in the workspace: secrets in `.env` files
must never reach the sandbox, where the agent can read them.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/env-guard   # workspace .env guard
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/env-guard <agent> .`

Optional but composed by the [examples](../../examples). No network rules
— capability egress comes from the other mixins.

## How it works

Ships the guard as a startup command (`setup.startup` in
[spec.yaml](spec.yaml)) that runs before the agent starts.

- **Clone mode** (`workspace.clone: true`): `.env` files — including
  `.env.local`-style variants and symlinked `.env` files — are removed
  from the sandbox-local copy, with a warning and a short pause.
- **Direct mode** (`workspace.clone: false`, the workspace tree mounted in
  place): the sandbox refuses to start when a `.env` file exists.
- On the all-clear, a short line reports the result.
