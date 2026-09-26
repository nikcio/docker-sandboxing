# uniform

Uniform DXP egress for the sandbox: docs, dashboard + Management API,
Edge Delivery API (incl. EU and the image CDN), and proxy-managed
`x-api-key` auth.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/uniform&ref=vX.Y.Z   # Uniform DXP
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/uniform <workload> .`

The sandbox network policy is deny-by-default, so API calls fail until the
Uniform hosts are allowed.

## How it works

- **Credential**: declares `UNIFORM_API_KEY` with `proxyManaged: true` —
  the sandbox only sees a placeholder; the proxy injects the real
  `x-api-key` header on requests to the Uniform hosts. The key is a
  service-account key or personal access token for the Management API and
  Edge Delivery API — create it in Uniform under Security > Service
Accounts. Full walkthrough:
[docs/uniform-api-key.md](../../docs/uniform-api-key.md).

## Network domains

| Domain | Why |
| ------ | --- |
| `uniform.app`, `*.uniform.app` | Dashboard, Management API, docs (docs.uniform.app), per-team app subdomains — US and EU |
| `uniform.global`, `*.uniform.global` | Edge Delivery API + image CDN (`img.uniform.global` / `img.eu.uniform.global`), US and EU |
