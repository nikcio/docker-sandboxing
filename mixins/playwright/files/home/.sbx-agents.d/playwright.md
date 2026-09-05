## Playwright

- Playwright is installed globally (npm) — run it as `npx playwright ...`
- Preinstalled browsers: Chromium **headless shell only** (headless use:
  `npx playwright screenshot <url> out.png`, scripts, scraping)
- More browsers: `npx playwright install chromium|firefox|webkit`
  (needs egress to cdn.playwright.dev; OS libraries via `--with-deps`)
- OS libraries were installed with `--with-deps`; those hosts belong to
  the `apt` mixin
