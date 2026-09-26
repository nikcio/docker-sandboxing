# docker-hub

Network rules so the in-sandbox Docker engine can pull base images and build containers against Docker Hub.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/docker-hub&ref=vX.Y.Z   # Docker Hub registry egress
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/docker-hub <workload> .`

The sandbox network policy is deny-by-default, so `docker pull`/`build` fails until the registry hosts are allowed.

## Network domains

| Domain | Why |
| ------ | --- |
| `docker.io`, `*.docker.io` | Registry API + image manifests |
| `*.docker.com` | Docker web/UI hosts (`hub.docker.com`) |
| `production.cloudflare.docker.com` | Blob/layer downloads (CDN) |
| `production.cloudfront.docker.com` | Blob/layer downloads (CDN fallback) |
