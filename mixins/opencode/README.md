# opencode

OpenCode itself: installs OpenCode 2 via the official curl installer when the sandbox lacks it, ships the permissive OpenCode config dropped into the global config layer, and merges provider config fragments from other mixins into the single `OPENCODE_CONFIG` file. Also carries the locale env the template images ship none of.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/opencode   # OpenCode CLI + config + provider merge
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/opencode <agent> .`

**Compose it in OpenCode sandboxes that use a model provider** (`opencode-zeldoc`, `opencode-copilot`) — the provider config fragment merges through it. The merge runs at every start.

## How it works

- **Install**: when `opencode` is missing from `PATH`, runs the official v2 installer (`curl -fsSL https://opencode.ai/v2/install | bash`) and symlinks the binary into `/usr/local/bin`. No Node.js/nvm involved — the installer ships a standalone binary. A no-op on images that already ship opencode.
- **Base config**: ships a permissive `~/.config/opencode/opencode.jsonc` (edit/bash/webfetch allowed — the sandbox is the isolation boundary) and points `OPENCODE_CONFIG` at the combined file.
- **Locale env**: sets `LANG`/`LC_ALL` to `C.UTF-8` — the template images ship no locale env, and without it man-db's profile script prints `manpath: can't set the locale` on every shell start. `C.UTF-8` is always present on glibc, no `locale-gen` needed.
- **Fragment merge**: a startup command merges every `~/.config/opencode/mixins.d/*.{json,jsonc}` fragment into the combined config with `jq`:
  - Objects merge recursively; later fragments win scalar conflicts.
  - Array-valued keys whose name starts with `enabled_` or `disabled_`
    (e.g. `enabled_mixins`, `enabled_providers`) are unioned — that is
    what lets mixins compose (`opencode-zeldoc` + `opencode-copilot` both stay enabled).
  - Comment-bearing `.jsonc` fragments are rejected — fragments must be
    comment-free `.json`.
  - No fragments (or a missing directory) yields the minimal valid config,
    so `OPENCODE_CONFIG` always points at a loadable file.
  - The output is regenerated at every start — edit the owning mixin in
    this repo (or a `mixins.d/` fragment) and restart the sandbox.

## Network domains

| Domain | Why |
| ------ | --- |
| `opencode.ai`, `*.opencode.ai` | the v2 installer script + latest-version metadata (`update.api.opencode.ai`) |
| `registry.npmjs.org`, `*.npmjs.org` | platform tarball download (installer pulls it from the npm registry) |
| `archive.ubuntu.com`, `security.ubuntu.com`, `ports.ubuntu.com` | apt installs in the install step (curl + ca-certificates; ports.ubuntu.com carries arm64 packages) |
| `*.digicert.com`, `*.symcd.com`, `*.symcb.com`, `*.ws.symantec.com`, `*.sectigo.com`, `*.usertrust.com`, `*.globalsign.com` | certificate revocation (OCSP/CRL) + timestamp endpoints hit during HTTPS installs |
