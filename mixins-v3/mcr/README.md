# mcr (v3)

Network rules so the in-sandbox Docker engine can pull base images and
build containers against Microsoft Container Registry.

Declaration-only: no image content, no build args.

## Build / use

```bash
docker buildx build -f mixins-v3/mcr/mcr.yaml -t my/mcr-mixin:dev mixins-v3/mcr/
sbx run <agent> . --kit ./mixins-v3/mcr   # or the published image reference
```

Or add it to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/mcr&ref=v3.1.2   # Microsoft Container Registry egress
```

## How it works

The sandbox network policy is deny by default, so `docker pull`/`docker
build` fails until the registry hosts are allowed. This mixin grants the
registry hosts at runtime.

## Runtime egress

| Domain | Why |
| ------ | --- |
| `mcr.microsoft.com`, `*.mcr.microsoft.com` | Registry API, manifests, and blob/layer downloads |
| `*.data.mcr.microsoft.com` | Blob/layer download endpoints |
