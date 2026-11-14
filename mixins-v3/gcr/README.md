# gcr (v3)

Network rules so the in-sandbox Docker engine can pull base images and
build containers against Google Container Registry.

Declaration-only: no image content, no build args.

## Build / use

```bash
docker buildx build -f mixins-v3/gcr/gcr.yaml -t my/gcr-mixin:dev mixins-v3/gcr/
sbx run <agent> . --kit ./mixins-v3/gcr   # or the published image reference
```

Or add it to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/gcr&ref=v3.1.2   # Google Container Registry egress
```

## How it works

The sandbox network policy is deny by default, so `docker pull`/`docker
build` fails until the registry hosts are allowed. This mixin grants the
registry hosts at runtime.

## Runtime egress

| Domain | Why |
| ------ | --- |
| `gcr.io:443`, `*.gcr.io:443` | Registry API, manifests, and blob/layer downloads |
