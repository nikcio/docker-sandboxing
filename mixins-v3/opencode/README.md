# opencode (v3)

OpenCode 2 baked into the mixin layer at image build, the permissive
opencode config written as a lifecycle file, and the mixins-fragment
config merge at startup. v3 port of [`../opencode`](../opencode) — read
that for the full behavior story; see [`../README.md`](../README.md) for
what the v3 format changes.

## Differences from the v2 mixin

- The CLI is baked at build under `/opt/opencode-home` (on PATH via a
  `/usr/local/bin` symlink); the create-time curl install is gone.
- The permissive config ships as a `lifecycle@1` `files:` entry written
  into the agent home at start (`overwrite: false`, so the agent's own
  config survives a recreate).
- OpenCode's data/cache/state dirs stay at their XDG defaults under the
  agent home (the v2 behavior): the baked `/opt/opencode-home` tree is
  root-owned, so pointing the XDG environment at it made opencode fail
  with `EACCES` on its first write.
- `OPENCODE_CONFIG` is not exported as image environment (the v2 mixin
  pointed it at the merged config): the workload's entrypoint owns the
  launch, so compose this mixin with a workload that reads the merged
  config, or extend it in your own kit.

## Build / use

```bash
docker buildx build -f mixins-v3/opencode/opencode.yaml -t my/opencode-mixin:dev mixins-v3/opencode/
sbx run <agent> . --kit ./mixins-v3/opencode   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `opencodeVersion` | *(empty = latest)* | OpenCode version |

## Runtime egress

`opencode.ai` + wildcard, `registry.npmjs.org` + wildcard (plugins/MCP
installs at start). No install-phase grant.
