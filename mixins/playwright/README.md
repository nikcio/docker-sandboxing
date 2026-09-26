# playwright

Installs **Playwright** and the Chromium **headless shell** (the smallest browser download) for browser automation and end-to-end testing, plus Node via nvm when the template image lacks one.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/playwright&ref=vX.Y.Z   # Playwright + Chromium headless shell
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/playwright <workload> .`

The install runs at **sandbox creation only**. Pair it with the `browser` mixin when Chrome itself is needed too.

## How it works

- **Node**: when `node` is missing, installs it via nvm exactly like the `node` mixin (Node 24 default, symlinks onto `PATH`, nvm in the agent home).
- **Playwright**: `npm install -g playwright`, then `npx playwright install --with-deps chromium-headless-shell` — `--with-deps` pulls Chromium's system libraries via apt. Skipped when the headless shell is already installed.
- Browsers land in `/home/agent/.cache/ms-playwright`, chowned to the `agent` user.
- The sandbox has no display server, so everything runs headless.

## Network domains

| Domain | Why |
| ------ | --- |
| `registry.npmjs.org`, `*.npmjs.org` | npm package install (also covered by the `node` mixin — rules union) |
| `cdn.playwright.dev`, `*.cdn.playwright.dev` | Playwright browser downloads |
| `playwright.azureedge.net` | Browser download CDN |
| `playwright.download.prss.microsoft.com` | Browser download CDN fallback |
| `storage.googleapis.com:443` | Redirect target for Chrome-for-Testing builds (the proxy blocks cross-host redirects otherwise) |
