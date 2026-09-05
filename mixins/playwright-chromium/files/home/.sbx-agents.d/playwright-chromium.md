## Playwright (Chromium)

- Playwright is installed globally (npm) — run it as `npx playwright ...`
- Preinstalled browser: full Chromium (headless use only — the sandbox
  has no display server)
- More browsers: `npx playwright install firefox|webkit` (needs egress
  to cdn.playwright.dev; OS libraries via `--with-deps`)
- OS libraries were installed with `--with-deps`; those hosts belong to
  the `apt` mixin
