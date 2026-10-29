# playwright (v3)

Playwright + the Chromium headless shell + the nvm Node it needs, baked
into the mixin layer at image build. v3 port of
[`../playwright`](../playwright) — read that for the full behavior story;
see [`../README.md`](../README.md) for what the v3 format changes.

## Build / use

```bash
docker buildx build -f mixins-v3/playwright/playwright.yaml -t my/playwright-mixin:dev mixins-v3/playwright/
sbx run <agent> . --kit ./mixins-v3/playwright   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `playwrightVersion` | *(empty = latest)* | Playwright version |
| `nodeVersion` | *(empty = latest)* | Node version for the nvm install |
| `nvmVersion` | *(empty = latest release)* | NVM ref to clone |

## Runtime egress

`registry.npmjs.org` + wildcard (package installs), `cdn.playwright.dev` +
wildcard, `playwright.azureedge.net`,
`playwright.download.prss.microsoft.com`, `storage.googleapis.com`
(browser downloads, including the Chrome-for-Testing redirect target).
No install-phase grant.
