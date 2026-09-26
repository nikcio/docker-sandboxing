# python

Python toolchain egress for the sandbox: a uv-managed CPython install (when the template image lacks one), the `uv` tool for package and venv management, PyPI access, and docs hosts.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/python&ref=vX.Y.Z   # python + uv
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/python <workload> .`

The install runs at **sandbox creation only** and needs egress — the domains in the table below plus the Ubuntu apt mirrors (`archive.ubuntu.com`/`security.ubuntu.com`) for its `apt-get` calls. On template images that already ship Python the install is a cheap no-op.

## How it works

- **Check-and-install**: when `python3` (with working pip) is missing, installs `uv` to `/usr/local/bin`, then `uv python install 3.12` as the `agent` user into the agent home, and symlinks `python3`/`python` onto `PATH` — the same uv-managed layout as the template images.
- `UV_PYTHON_INSTALL_DIR` and `UV_LINK_MODE=copy` are exported via `/etc/sandbox-persistent.sh` so login shells use the same interpreter store.
- `python3-venv` is installed as a best-effort for venv workflows.

## Network domains

| Domain | Why |
| ------ | --- |
| `pypi.org`, `*.pypi.org` | PyPI index and queries (uv/pip installs) |
| `files.pythonhosted.org`, `*.pythonhosted.org` | Package files on the PyPI CDN |
| `astral.sh:443`, `releases.astral.sh:443` | uv installer (the short link redirects to releases.astral.sh) |
| `python.org`, `*.python.org` | Docs, FTP downloads, devguide lifecycles, PEPs |
