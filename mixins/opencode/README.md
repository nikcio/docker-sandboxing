# opencode

OpenCode itself: installs OpenCode 2 via the official curl installer when the sandbox lacks it, ships the permissive OpenCode config dropped into the global config layer, and merges `mixins.d/` config fragments (from third-party project kits) into the single `OPENCODE_CONFIG` file. Also carries the locale env the template images ship none of.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/opencode   # OpenCode CLI + config + provider merge
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/opencode <agent> .`

Model provider mixins (`opencode-zeldoc`, `opencode-copilot`) work without it — they merge their own config into the global config file. Composing it alongside them is still fine and is the common setup (it ships the permissive base config and the locale env).

## How it works

- **Install**: when `opencode` is missing from `PATH`, runs the official v2 installer (`curl -fsSL https://opencode.ai/v2/install | env HOME=/home/agent bash`, or a pinned version via `--version`) and symlinks the binary into `/usr/local/bin`. `HOME` is pinned to the agent user so the binary lands where the symlink points (install steps run as root). No Node.js/nvm involved — the installer ships a standalone binary. A no-op on images that already ship opencode.
- **Base config**: ships a permissive comment-free `~/.config/opencode/opencode.json` (edit/bash/webfetch allowed — the sandbox is the isolation boundary; comment-free so the provider mixins' jq merges can parse it) and points `OPENCODE_CONFIG` at the combined file.
- **Locale env**: sets `LANG`/`LC_ALL` to `C.UTF-8` — the template images ship no locale env, and without it man-db's profile script prints `manpath: can't set the locale` on every shell start. `C.UTF-8` is always present on glibc, no `locale-gen` needed.
- **Fragment merge**: a startup command merges every `~/.config/opencode/mixins.d/*.{json,jsonc}` fragment into the combined config with `jq`:
  - Objects merge recursively; later fragments win scalar conflicts.
  - Array-valued keys whose name starts with `enabled_` or `disabled_`
    (e.g. `enabled_mixins`, `enabled_providers`) are unioned — that is
    what lets third-party kit fragments compose without clobbering each other.
  - Comment-bearing `.jsonc` fragments are rejected — fragments must be
    comment-free `.json`.
  - No fragments (or a missing directory) yields the minimal valid config,
    so `OPENCODE_CONFIG` always points at a loadable file.
  - The output is regenerated at every start — edit the owning mixin in
    this repo (or a `mixins.d/` fragment) and restart the sandbox.

## Version arguments

| Arg | Default | Description |
| --- | ------- | ----------- |
| `opencodeVersion` | *(empty = latest)* | OpenCode version to install, with or without the leading `v` |

Pass values with `--kit-arg opencode.opencodeVersion=<version>` (or an `args:` entry on the mixin's `kits:` line in `sbxenv.yaml`) — see [Version arguments](../../docs/mixins.md#version-arguments).

## Network domains

| Domain | Why |
| ------ | --- |
| `opencode.ai`, `*.opencode.ai` | the v2 installer script + latest-version metadata (`update.api.opencode.ai`) |
| `registry.npmjs.org`, `*.npmjs.org` | platform tarball download (installer pulls it from the npm registry) |
| `archive.ubuntu.com`, `security.ubuntu.com`, `ports.ubuntu.com` | apt installs in the install step (curl + ca-certificates; ports.ubuntu.com carries arm64 packages) |
| `*.digicert.com`, `*.symcd.com`, `*.symcb.com`, `*.ws.symantec.com`, `*.sectigo.com`, `*.usertrust.com`, `*.globalsign.com` | certificate revocation (OCSP/CRL) + timestamp endpoints hit during HTTPS installs |
