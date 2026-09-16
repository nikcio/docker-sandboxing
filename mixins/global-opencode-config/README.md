# global-opencode-config

Permissive OpenCode config dropped into the global config layer, plus the
merge step that combines provider config fragments from other mixins into
the single `OPENCODE_CONFIG` file.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/global-opencode-config   # OpenCode config + provider merge
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/global-opencode-config <agent> .`

**Required when composing a model provider** (`zeldoc`, `copilot`) —
their config fragments only merge through it. The merge runs at every
start; no network rules.

## How it works

- **Base config**: ships a permissive `~/.config/opencode/opencode.jsonc`
  (edit/bash/webfetch allowed — the sandbox is the isolation boundary) and
  points `OPENCODE_CONFIG` at the combined file.
- **Fragment merge**: a startup command merges every
  `~/.config/opencode/mixins.d/*.{json,jsonc}` fragment into the combined
  config with `jq`:
  - Objects merge recursively; later fragments win scalar conflicts.
  - `enabled_mixins` / `disabled_mixins` lists are unioned.
  - Comment-bearing `.jsonc` fragments are rejected — fragments must be
    comment-free `.json`.
  - No fragments (or a missing directory) yields the minimal valid config,
    so `OPENCODE_CONFIG` always points at a loadable file.
  - The output is regenerated at every start — edit the owning mixin in
    this repo (or a `mixins.d/` fragment) and restart the sandbox.
