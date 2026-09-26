# mcr

Network rules so the in-sandbox Docker engine can pull base images and build containers against Microsoft Container Registry.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/mcr&ref=vX.Y.Z   # Microsoft Container Registry egress
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/mcr <workload> .`

The sandbox network policy is deny-by-default, so `docker pull`/`build` fails until the registry hosts are allowed.

## Network domains

| Domain | Why |
| ------ | --- |
| `mcr.microsoft.com`, `*.mcr.microsoft.com` | Registry API, manifests, and blob/layer downloads |
| `*.data.mcr.microsoft.com` | Blob/layer download endpoints |
