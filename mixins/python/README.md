# python

Python toolchain egress for the sandbox: a uv-managed CPython install (when the base image lacks one), the `uv` tool for package and venv management, PyPI access, and docs hosts.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/python   # python + uv
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/python <agent> .`

The install runs at sandbox creation only and needs egress. It uses the domains in the table below plus the Ubuntu apt mirrors (`archive.ubuntu.com`/`security.ubuntu.com`) for its `apt-get` calls. On base images that already ship Python the install is a no-op.

## How it works

- **Check-and-install.** When `python3` (with working pip) is missing, installs `uv` to `/usr/local/bin`, then installs the latest stable CPython (or a pinned version via the `pythonVersion` arg) as the `agent` user into the agent home, and symlinks `python3`/`python` onto `PATH`, the same uv-managed layout as the base images.
- `UV_PYTHON_INSTALL_DIR` and `UV_LINK_MODE=copy` are exported via `/etc/sandbox-persistent.sh` so login shells use the same interpreter store.
- `python3-venv` is installed as a best-effort for venv workflows.

## Version arguments

| Arg | Default | Description |
| --- | ------- | ----------- |
| `pythonVersion` | *(empty = latest stable)* | CPython version for uv to install — a major-minor (`MAJOR.MINOR`) or a full (`MAJOR.MINOR.PATCH`) |
| `uvVersion` | *(empty = latest)* | uv version to install in `MAJOR[.MINOR[.PATCH]]` form |

Pass values with `--kit-arg python.pythonVersion=<version>` (or an `args:` entry on the mixin's `kits:` line in `sbxenv.yaml`) — see [Version arguments](../../docs/mixins.md#version-arguments).

## Network domains

| Domain | Why |
| ------ | --- |
| `pypi.org`, `*.pypi.org` | PyPI index and queries (uv/pip installs) |
| `files.pythonhosted.org`, `*.pythonhosted.org` | Package files on the PyPI CDN |
| `astral.sh:443`, `releases.astral.sh:443` | uv installer (the short link redirects to releases.astral.sh) |
| `python.org`, `*.python.org` | Docs, FTP downloads, devguide lifecycles, PEPs |
