# browser

Installs **Google Chrome** (stable) plus the color emoji font inside the
sandbox for headless browsing: page checks, screenshots, DOM dumps, and
browser automation alongside Playwright or an agent script.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/browser&ref=vX.Y.Z   # Google Chrome
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/browser <workload> .`

The install runs at **sandbox creation only**. On
template images that already ship Chrome the install is a no-op.

## How it works

- Downloads the official `.deb` from `dl.google.com` and installs it with
  apt, then removes the added apt source again to keep the sandbox's apt
  state clean.
- Installs `fonts-noto-color-emoji` so emoji render in screenshots.
- Skipped when `google-chrome` is already on `PATH`.

Inside the sandbox:

```console
google-chrome --headless=new --no-sandbox --dump-dom <url>
google-chrome --headless=new --no-sandbox --screenshot=out.png <url>
```

Always pass `--no-sandbox`: Chrome's own sandbox needs user namespaces the
sandbox VM does not allow — without the flag Chrome aborts with a zygote
crash (exit 134). There is no display server in the sandbox, so browsing
is headless only.

## Network domains

| Domain | Why |
| ------ | --- |
| `dl.google.com:443` | The Chrome `.deb` download |

Sites the browser visits stay gated by the other mixins' rules.
