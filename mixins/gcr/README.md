# gcr

> **Deprecated.** This kit uses the deprecated v2 kit format. Use the v3 mixin in [`mixins-v3/gcr/`](../../mixins-v3/gcr/) instead — same purpose and behavior. To migrate an existing setup, see the [v2 kit migration guide](../../docs/migrating-from-v2-kits.md).

Network rules so the in-sandbox Docker engine can pull base images and build containers against Google Container Registry.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/gcr   # Google Container Registry egress
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/gcr <agent> .`

The sandbox network policy is deny by default, so `docker pull`/`build` fails until the registry hosts are allowed.

## Network domains

| Domain | Why |
| ------ | --- |
| `gcr.io:443`, `*.gcr.io:443` | Registry API, manifests, and blob/layer downloads |
