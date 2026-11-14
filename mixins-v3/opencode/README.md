# opencode (v3)

OpenCode 2 baked into the mixin layer at image build, the permissive
opencode config written as a lifecycle file, the `mixins.d/` config
fragment merge at startup, and the locale env the template images ship
none of.

## Build / use

```bash
docker buildx build -f mixins-v3/opencode/opencode.yaml -t my/opencode-mixin:dev mixins-v3/opencode/
sbx run <agent> . --kit ./mixins-v3/opencode   # or the published image reference
```

In a project's `sbxenv.yaml`, add the mixin to the `kits:` list (pin
`&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/opencode   # OpenCode CLI + config + provider merge
```

Model provider mixins (`opencode-zeldoc`, `opencode-copilot`) work without
it, because they merge their own config into the global config file.
Composing it alongside them is still fine and is the common setup (it
ships the permissive base config and the locale env).

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `opencodeVersion` | *(empty = latest)* | OpenCode version, with or without the leading `v` |

Args resolve when the image builds (empty = latest upstream release);
pass them with an `args:` entry on the mixin's `kits:` line.

## How it works

- **Install.** The official curl installer runs at image build with
  `HOME=/opt/opencode-home`, so the binary lands under `/opt` (a mixin's
  layers are an overlay on an unknown base, so its content must not
  claim `/home/agent` paths the base or the agent user owns); a symlink
  in `/usr/local/bin` puts `opencode` on PATH. No Node.js/nvm involved:
  the installer ships a standalone binary.
- **Base config.** A permissive comment-free
  `~/.config/opencode/opencode.json` (edit/bash/webfetch allowed, because
  the sandbox is the isolation boundary) is written at startup as a
  `lifecycle@1` `files:` entry with `overwrite: false` — the agent's own
  config, or a local override, is never clobbered on a recreate.
- **Locale env.** The image sets `LANG`/`LC_ALL` to `C.UTF-8`. The
  template images ship no locale env, and without it man-db's profile
  script prints `manpath: can't set the locale` on every shell start.
  `C.UTF-8` is always present on glibc, no `locale-gen` needed.
- **Fragment merge.** A startup command merges every
  `~/.config/opencode/mixins.d/*.json` fragment into the combined config
  with `jq`:
  - Objects merge recursively; later fragments win scalar conflicts.
  - Array-valued keys whose name starts with `enabled_` or `disabled_`
    (e.g. `enabled_mixins`, `enabled_providers`) are unioned. That is
    what lets third-party kit fragments compose without clobbering each
    other.
  - Comment-bearing `.jsonc` fragments are rejected; fragments must be
    comment-free `.json`.
  - No fragments (or a missing directory) yields the minimal valid
    config, so the combined file is always loadable.
  - The output is regenerated at every start. Edit the owning mixin (or
    a `mixins.d/` fragment) and restart the sandbox.

## Notes

- OpenCode's data/cache/state dirs stay at their XDG defaults under the
  agent home: the baked `/opt/opencode-home` tree is root-owned, so
  pointing the XDG environment at it would make opencode fail with
  `EACCES` on its first write.
- `OPENCODE_CONFIG` is not exported as image environment: the workload's
  entrypoint owns the launch, so compose this mixin with a workload that
  reads the merged config, or extend it in your own kit.

## Runtime egress

`opencode.ai` + wildcard (installer script + latest-version metadata;
kept for agent-driven installs/updates), `registry.npmjs.org` + wildcard
(platform tarball; plugins/MCP installs at start). No install-phase
grant: the install is baked at build.
