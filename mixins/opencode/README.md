# opencode

Installs the [OpenCode](https://opencode.ai) agent on the sandbox's shell
workloads, ships the permissive OpenCode config, and merges provider
mixins' config fragments into the combined `OPENCODE_CONFIG` before the
agent starts. Replaces the previous `opencode-update` and
`global-opencode-config` mixins.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - docker.io/nikcio/sbx-mixin-opencode:vX.Y.Z # opencode + config + provider merge
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/opencode <workload> .`

**Required when composing a model provider** (`zeldoc`, `copilot`) —
their config fragments only merge through it.

## How it works

- **Agent install**: an install hook installs the opencode release from
  the mixin's `version` arg into the agent's npm prefix — `latest` (the
  default) rolls to the newest npm release at every sandbox creation; a
  pinned version (e.g. `1.18.32`) is installed once and skipped on
  later creations when already present. The shell workload image ships
  no agent; on a node-based workload this refreshes the baked version.
- **Base config**: writes the permissive
  `~/.config/opencode/opencode.jsonc` (edit/bash/webfetch allowed — the
  sandbox is the isolation boundary) with `overwrite: false`, so a config
  from an earlier boot is kept.
- **Fragment merge**: an install hook (the only phase guaranteed to
  finish before the agent starts) runs
  `/opt/sandbox/opencode/merge-mixins.sh`, merging every
  `~/.config/opencode/mixins.d/*.json` fragment into the combined config
  with `jq`:
  - Objects merge recursively; later fragments win scalar conflicts.
  - Array-valued keys whose name starts with `enabled_` or `disabled_`
    (e.g. `enabled_mixins`, `enabled_providers`) are unioned — that is
    what lets mixins compose (`zeldoc` + `copilot` both stay enabled).
  - Comment-bearing `.jsonc` fragments are rejected — fragments must be
    comment-free `.json`.
  - No fragments (or a missing directory) yields the minimal valid
    config, so `OPENCODE_CONFIG` always points at a loadable file.
  - To re-merge after dropping a new fragment into a running sandbox:
    `/opt/sandbox/opencode/merge-mixins.sh`.
- **Environment**: `OPENCODE_CONFIG` points at the combined file
  (mixin image `ENV`, composed additively at assembly).

No network rules beyond the npm registry (agent install + plugin fetches).
