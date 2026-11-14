# python (v3)

uv-managed Python interpreter + the uv tool baked into the mixin layer at
image build, with PyPI egress.

## Build / use

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
`&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/python   # python + uv
```

For local development, point `--kit` at the directory instead —
`sbx run <agent> . --kit ./mixins-v3/python` — or build the image yourself:

```bash
docker buildx build -f mixins-v3/python/python.yaml -t my/python-mixin:dev mixins-v3/python/
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `pythonVersion` | *(empty = latest stable)* | CPython version for uv to install |
| `uvVersion` | *(empty = latest)* | uv version |

## How it works

- `uv`/`uvx` are installed to `/usr/local/bin` via the astral.sh installer
  (pinned with `uvVersion` if set), then the requested CPython (latest
  stable by default) is installed into `/opt/uv/python` and `python3`/
  `python` are symlinked onto `/usr/local/bin`. The interpreter lives
  under `/opt`, not the agent home: a mixin's layers are an overlay on an
  unknown base, so its content must not claim `/home/agent` paths.
- A `/etc/profile.d/` script exports `UV_PYTHON_INSTALL_DIR=/opt/uv/python`
  and `UV_LINK_MODE=copy`, so login shells use the same interpreter store
  the baked interpreter came from.
- The versions resolve "latest" at build, so the image is a snapshot:
  rebuild (or pin via the build args) to update it.

## Runtime egress

`pypi.org` + wildcard, `files.pythonhosted.org` + wildcard (PyPI index and
package CDN for uv/pip installs), `astral.sh` + `releases.astral.sh`
(`uv self update`; the short link 301-redirects to
`releases.astral.sh`, which is also the installer's artifact host on
rebuilds), `python.org` + wildcard (docs, FTP downloads, devguide
lifecycles, PEPs for agent lookups). No install-phase grant.
