# playwright (v3)

Playwright + the Chromium headless shell + the nvm Node it needs, baked
into the mixin layer at image build.

## Build / use

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
`&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/playwright   # Playwright + Chromium headless shell
```

For local development, point `--kit` at the directory instead —
`sbx run <agent> . --kit ./mixins-v3/playwright` — or build the image
yourself:

```bash
docker buildx build -f mixins-v3/playwright/playwright.yaml -t my/playwright-mixin:dev mixins-v3/playwright/
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `playwrightVersion` | *(empty = latest)* | Playwright version |
| `nodeVersion` | *(empty = latest)* | Node version for the nvm install |
| `nvmVersion` | *(empty = latest release)* | NVM ref to clone |

## How it works

- Node comes via nvm, same layout as the `node` mixin: nvm tree under
  `/opt/nvm`, `node`/`npm`/`npx`/`corepack` symlinked onto
  `/usr/local/bin`, `NVM_DIR` exported via a `/etc/profile.d/` script.
- Playwright is installed globally via npm (pinned with
  `playwrightVersion` if set), then `npx playwright install --with-deps
  chromium-headless-shell` bakes the Chromium headless shell — the
  smallest browser download — with its system libraries into
  `/opt/ms-playwright`. `PLAYWRIGHT_BROWSERS_PATH=/opt/ms-playwright`
  ships in a profile script, so Playwright resolves the baked browsers.
  Additional browsers can be installed at runtime with
  `npx playwright install` using the egress below.
- The sandbox has no display server, so everything runs headless.
- The versions resolve "latest" at build, so the image is a snapshot:
  rebuild (or pin via the build args) to update it.

## Runtime egress

`registry.npmjs.org` + wildcard (npm package installs/updates),
`cdn.playwright.dev` + wildcard, `playwright.azureedge.net`,
`playwright.download.prss.microsoft.com` (browser downloads),
`storage.googleapis.com`. The CDN 302-redirects Chrome-for-Testing builds
to `storage.googleapis.com`, so the redirect target must be allowed too —
the sandbox proxy blocks cross-host redirects otherwise. No install-phase
grant.
