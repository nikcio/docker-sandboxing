## Browser (Google Chrome)

- Google Chrome is installed (`google-chrome` / `google-chrome-stable`)
- Headless checks: `google-chrome --headless=new --no-sandbox --dump-dom <url>`
  or `--no-sandbox --screenshot=out.png <url>`
- Always pass `--no-sandbox`: Chrome's own sandbox needs user namespaces
  the sandbox VM does not allow — without the flag Chrome aborts with a
  zygote crash (exit 134)
- The browser works headless; there is no display server in the sandbox
