# global-opencode-config

Permissive OpenCode config dropped into the global config layer, the merge
step that combines provider config fragments from other mixins into the
single `OPENCODE_CONFIG` file, and the sandbox `AGENTS.md` baseline
(`~/.sandbox-agents.md` plus `~/.sandbox-agents/*.md` — git auth, network
rules, persistent env, workspace mode) rebuilt into the workspace
`AGENTS.md` together with a note from every other composed mixin.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/global-opencode-config   # OpenCode config + provider merge + AGENTS.md baseline
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/global-opencode-config <agent> .`

**Required when composing a model provider** (`zeldoc`, `copilot`) —
their config fragments only merge through it. Required by every kit for
the sandbox `AGENTS.md` baseline. The merge and the baseline rebuild run
at every start; no network rules.

## How it works

- **Base config**: ships a permissive `~/.config/opencode/opencode.jsonc`
  (edit/bash/webfetch allowed — the sandbox is the isolation boundary) and
  points `OPENCODE_CONFIG` at the combined file.
- **Fragment merge**: a startup command merges every
  `~/.config/opencode/mixins.d/*.{json,jsonc}` fragment into the combined
  config with `jq`:
  - Objects merge recursively; later fragments win scalar conflicts.
  - Array-valued keys whose name starts with `enabled_` or `disabled_`
    (e.g. `enabled_mixins`, `enabled_providers`) are unioned — that is
    what lets mixins compose (`zeldoc` + `copilot` both stay enabled).
  - Comment-bearing `.jsonc` fragments are rejected — fragments must be
    comment-free `.json`.
  - No fragments (or a missing directory) yields the minimal valid config,
    so `OPENCODE_CONFIG` always points at a loadable file.
  - The output is regenerated at every start — edit the owning mixin in
    this repo (or a `mixins.d/` fragment) and restart the sandbox.
- **AGENTS.md baseline**: ships the sandbox baseline as `~/.sandbox-agents.md`
  (environment facts plus an index of guidance files in `~/.sandbox-agents/`).
  A startup command rebuilds the workspace `AGENTS.md` at every start:
  - Strips the runtime-generated content from the workspace file, keeping
    only the runtime-managed Kits section (`<!-- sbx:kits-section -->`).
  - Writes the sandbox baseline from `~/.sandbox-agents.md`.
  - Appends every note in `~/.sbx-agents.d/*.md` — one per composed mixin.
    Notes containing `sbx:kits-section` markers are skipped with a warning
    (kits-sections are runtime-owned).
  - Re-appends the preserved Kits section last.

  Without the rebuild the workspace `AGENTS.md` is whatever the runtime
  generated. Every kit composes this mixin, so the file the agent actually
  reads gets the full sandbox baseline (and every mixin's agent note).
