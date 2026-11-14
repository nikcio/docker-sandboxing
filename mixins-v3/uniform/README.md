# uniform (v3)

Uniform DXP egress for the sandbox: docs, dashboard + Management API,
Edge Delivery API (incl. EU and the image CDN), and proxy-managed
`x-api-key` auth (`com.docker.sandbox/credential@1`, `service: uniform`).

Declaration-only: no image content, no build args. The credential is
optional and registered host-side with `sbx secret set uniform` — see
[docs/uniform-api-key.md](../../docs/uniform-api-key.md).

## Build / use

```bash
docker buildx build -f mixins-v3/uniform/uniform.yaml -t my/uniform-mixin:dev mixins-v3/uniform/
sbx run <agent> . --kit ./mixins-v3/uniform   # or the published image reference
```

Or add it to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/uniform&ref=v3.1.2   # Uniform DXP
```

## How it works

The sandbox network policy is deny by default, so API calls fail until
the Uniform hosts are allowed. This mixin grants them at runtime.

- **Credential.** Declares `UNIFORM_API_KEY` with `proxyManaged: true`.
  The sandbox only sees a placeholder; the proxy injects the real
  `x-api-key` header on requests to the Uniform hosts. The key is a
  service-account key or personal access token for the Management API
  and Edge Delivery API. Create it in Uniform under Security > Service
  Accounts. Full walkthrough:
  [docs/uniform-api-key.md](../../docs/uniform-api-key.md).

## Runtime egress

| Domain | Why |
| ------ | --- |
| `uniform.app`, `*.uniform.app` | Dashboard, Management API, docs (docs.uniform.app), per-team app subdomains, US and EU |
| `uniform.global`, `*.uniform.global` | Edge Delivery API + image CDN (`img.uniform.global` / `img.eu.uniform.global`), US and EU |

## Notes

- The credential's inject domains (`eu.uniform.app`,
  `eu.uniform.global`) must appear literally in the allow list: the
  wildcards cover runtime egress, but an inject domain must be in the
  allow list exactly.
