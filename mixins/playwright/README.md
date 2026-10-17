# playwright

Installs **Playwright** and the Chromium **headless shell** (the smallest browser download) for browser automation and end-to-end testing, plus Node via nvm when the base image lacks one.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/playwright   # Playwright + Chromium headless shell
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/playwright <agent> .`

The install runs at **sandbox creation only**.

## How it works

- **Node**: when `node` is missing, installs it via nvm exactly like the `node` mixin (latest Node default, symlinks onto `PATH`, nvm in the agent home).
- **Playwright**: `npm install -g playwright`, then `npx playwright install --with-deps chromium-headless-shell` — `--with-deps` pulls Chromium's system libraries via apt. Skipped when the headless shell is already installed.
- Browsers land in `/home/agent/.cache/ms-playwright`, chowned to the `agent` user.
- The sandbox has no display server, so everything runs headless.

## Version arguments

| Arg | Default | Description |
| --- | ------- | ----------- |
| `playwrightVersion` | *(empty = latest)* | Playwright version to install in `MAJOR[.MINOR[.PATCH]]` form |
| `nodeVersion` | *(empty = latest)* | Node.js version installed via nvm when the base image lacks node — a major or a full version |
| `nvmVersion` | *(empty = latest release)* | NVM version to clone in the upstream `vMAJOR.MINOR.PATCH` form |

Pass values with `--kit-arg playwright.playwrightVersion=<version>` (or an `args:` entry on the mixin's `kits:` line in `sbxenv.yaml`) — see [Version arguments](../../docs/mixins.md#version-arguments).

## Network domains

| Domain | Why |
| ------ | --- |
| `registry.npmjs.org`, `*.npmjs.org` | npm package install (also covered by the `node` mixin — rules union) |
| `cdn.playwright.dev`, `*.cdn.playwright.dev` | Playwright browser downloads |
| `playwright.azureedge.net` | Browser download CDN |
| `playwright.download.prss.microsoft.com` | Browser download CDN fallback |
| `storage.googleapis.com:443` | Redirect target for Chrome-for-Testing builds (the proxy blocks cross-host redirects otherwise) |
