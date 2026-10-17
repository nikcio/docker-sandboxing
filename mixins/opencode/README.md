# opencode

OpenCode itself: installs the CLI (via npm) when the sandbox lacks it, ships the permissive OpenCode config dropped into the global config layer, and merges provider config fragments from other mixins into the single `OPENCODE_CONFIG` file. Also carries the locale env the template images ship none of.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/opencode   # OpenCode CLI + config + provider merge
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/opencode <agent> .`

**Compose it in OpenCode sandboxes that use a model provider** (`opencode-zeldoc`, `opencode-copilot`) — the provider config fragment merges through it. The merge runs at every start.

## How it works

- **Install**: when `opencode` is missing from `PATH`, installs Node via nvm (when npm is missing too) and then `npm install -g opencode-ai@latest` (or a pinned version). A no-op on images that already ship opencode.
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

## Version arguments

| Arg | Default | Description |
| --- | ------- | ----------- |
| `opencodeVersion` | *(empty = latest)* | OpenCode version to install (`1.19.0`) |
| `nodeVersion` | *(empty = latest)* | Node.js version installed via nvm when the base image lacks npm (`24`) |
| `nvmVersion` | *(empty = latest release)* | NVM version to clone (`v0.40.8`) |

Pass values with `--kit-arg opencode.opencodeVersion=1.19.0` (or an `args:` entry on the mixin's `kits:` line in `sbxenv.yaml`) — see [Version arguments](../../docs/mixins.md#version-arguments).

## Network domains

| Domain | Why |
| ------ | --- |
| `registry.npmjs.org`, `*.npmjs.org` | the `opencode-ai` npm package (install) |
| `nodejs.org`, `*.nodejs.org`, `iojs.org` | Node distribution via nvm (only when npm is missing) |
| `github.com`, `*.github.com`, `raw.githubusercontent.com` | nvm itself (repo clone when the install runs) |
| `archive.ubuntu.com`, `security.ubuntu.com`, `ports.ubuntu.com` | apt installs in the install step (ports.ubuntu.com carries arm64 packages) |
| `*.digicert.com`, `*.symcd.com`, `*.symcb.com`, `*.ws.symantec.com`, `*.sectigo.com`, `*.usertrust.com`, `*.globalsign.com` | certificate revocation (OCSP/CRL) + timestamp endpoints hit during HTTPS installs |
