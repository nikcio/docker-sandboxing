## Browser (Google Chrome)

- Google Chrome is installed (`google-chrome` / `google-chrome-stable`)
- Headless checks: `google-chrome --headless=new --dump-dom <url>` or
  `--screenshot=out.png <url>`
- The browser works headless; there is no display server in the sandbox
- Network egress is still governed by the sandbox policy — sites not
  allowed by the composed mixins return 403 (check `sbx policy log` on
  the host)
