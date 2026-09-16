# agents-md

Replaces the runtime-generated workspace `AGENTS.md` with a sandbox-owned
baseline: the guidance files shipped as this mixin's `files/`
(`~/.sandbox-agents.md` plus `~/.sandbox-agents/*.md` — git auth, network
rules, persistent env, workspace mode) plus a note from every other
composed mixin.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/agents-md   # AGENTS.md baseline
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/agents-md <agent> .`

## How it works

A startup command rebuilds the workspace `AGENTS.md` at every start:

- Strips the runtime-generated content from the workspace file, keeping
  only the runtime-managed Kits section (`<!-- sbx:kits-section -->`).
- Writes the sandbox baseline from `~/.sandbox-agents.md`.
- Appends every note in `~/.sbx-agents.d/*.md` — one per composed mixin.
  Notes containing `sbx:kits-section` markers are skipped with a warning
  (kits-sections are runtime-owned).
- Re-appends the preserved Kits section last.

Without this mixin the workspace `AGENTS.md` is whatever the runtime
generated. Compose it when you want the full sandbox baseline (and every
mixin's agent note) in the file the agent actually reads.
