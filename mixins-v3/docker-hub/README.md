# docker-hub (v3)

Network rules so the in-sandbox Docker engine can pull base images and
build containers against Docker Hub.

Declaration-only: no image content, no build args.

## Build / use

```bash
docker buildx build -f mixins-v3/docker-hub/docker-hub.yaml -t my/docker-hub-mixin:dev mixins-v3/docker-hub/
sbx run <agent> . --kit ./mixins-v3/docker-hub   # or the published image reference
```

Or add it to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/docker-hub&ref=v3.1.2   # Docker Hub registry egress
```

## How it works

The sandbox network policy is deny by default, so `docker pull`/`docker
build` fails until the registry hosts are allowed. This mixin grants the
registry hosts at runtime.

## Runtime egress

| Domain | Why |
| ------ | --- |
| `docker.io`, `*.docker.io` | Registry API + image manifests |
| `*.docker.com` | Registry auth/token endpoints |
| `production.cloudflare.docker.com` | Blob/layer downloads (CDN) |
| `production.cloudfront.docker.com` | Blob/layer downloads (CDN fallback) |
