# sbx (v3)

The Docker Sandboxes CLI (`sbx`/`docker-sbx`) baked into the mixin layer
at image build. v3 port of [`../sbx`](../sbx) — read that for the full
behavior story; see [`../README.md`](../README.md) for what the v3 format
changes.

## Build / use

```bash
docker buildx build -f mixins-v3/sbx/sbx.yaml -t my/sbx-mixin:dev mixins-v3/sbx/
sbx run <agent> . --kit ./mixins-v3/sbx   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `sbxVersion` | *(empty = latest apt candidate)* | docker-sbx apt package version |

## Runtime egress

`download.docker.com` (apt repo baked at build; kept for in-sandbox
upgrades). No install-phase grant.
