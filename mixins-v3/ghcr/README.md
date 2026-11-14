# ghcr (v3)

Network rules so the in-sandbox Docker engine can pull base images and
build containers against GitHub Container Registry.

Declaration-only: no image content, no build args.

## Build / use

```bash
docker buildx build -f mixins-v3/ghcr/ghcr.yaml -t my/ghcr-mixin:dev mixins-v3/ghcr/
sbx run <agent> . --kit ./mixins-v3/ghcr   # or the published image reference
```

Or add it to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/ghcr&ref=v3.1.2   # GitHub Container Registry egress
```

## How it works

The sandbox network policy is deny by default, so `docker pull`/`docker
build` fails until the registry hosts are allowed. This mixin grants the
registry hosts at runtime. Private images additionally need registry
auth: run `docker login ghcr.io` inside the sandbox with a token that can
read them.

## Runtime egress

| Domain | Why |
| ------ | --- |
| `ghcr.io:443`, `*.ghcr.io:443` | Registry API, manifests, and blob/layer downloads |
