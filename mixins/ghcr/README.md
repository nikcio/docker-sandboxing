# ghcr

Network rules so the in-sandbox Docker engine can pull base images and build containers against GitHub Container Registry.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/ghcr&ref=vX.Y.Z   # GitHub Container Registry egress
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/ghcr <workload> .`

The sandbox network policy is deny-by-default, so `docker pull`/`build` fails until the registry hosts are allowed. Private images additionally need registry auth: the sandbox does not see your GitHub token (the proxy injects it only for github.com hosts), so authenticate the in-sandbox docker daemon via `sbx registries` or a read:packages PAT stored separately.

## Network domains

| Domain | Why |
| ------ | --- |
| `ghcr.io:443`, `*.ghcr.io:443` | Registry API, manifests, and blob/layer downloads |
