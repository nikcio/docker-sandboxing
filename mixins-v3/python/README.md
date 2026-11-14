# python (v3)

uv-managed Python interpreter + the uv tool baked into the mixin layer at
image build, with PyPI egress. v3 port of [`../python`](../python) — read
that for the full behavior story; see [`../README.md`](../README.md) for
what the v3 format changes.

## Build / use

```bash
docker buildx build -f mixins-v3/python/python.yaml -t my/python-mixin:dev mixins-v3/python/
sbx run <agent> . --kit ./mixins-v3/python   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `pythonVersion` | *(empty = latest stable)* | CPython version for uv to install |
| `uvVersion` | *(empty = latest)* | uv version |

## Runtime egress

`pypi.org` + wildcard, `files.pythonhosted.org` + wildcard, `astral.sh` +
`releases.astral.sh` (uv self-update), `python.org` + wildcard (docs).
No install-phase grant.
