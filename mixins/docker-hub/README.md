# docker-hub

> **Deprecated.** This kit uses the deprecated v2 kit format. Use the v3 mixin in [`mixins-v3/docker-hub/`](../../mixins-v3/docker-hub/) instead — same purpose and behavior. To migrate an existing setup, see the [v2 kit migration guide](../../docs/migrating-from-v2-kits.md).

Network rules so the in-sandbox Docker engine can pull base images and build containers against Docker Hub.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/docker-hub   # Docker Hub registry egress
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/docker-hub <agent> .`

The sandbox network policy is deny by default, so `docker pull`/`build` fails until the registry hosts are allowed.

## Network domains

| Domain | Why |
| ------ | --- |
| `docker.io`, `*.docker.io` | Registry API + image manifests |
| `*.docker.com` | Registry auth/token endpoints |
| `production.cloudflare.docker.com` | Blob/layer downloads (CDN) |
| `production.cloudfront.docker.com` | Blob/layer downloads (CDN fallback) |
