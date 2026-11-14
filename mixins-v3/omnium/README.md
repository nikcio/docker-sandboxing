# omnium (v3)

Omnium OMS/e-commerce API egress for the sandbox: the REST API hosts
(production/test/dev, each with a Swagger UI under `/documentation`),
tech docs, and proxy-managed `Authorization: Bearer` auth
(`com.docker.sandbox/credential@1`, `service: omnium`).

Declaration-only: no image content, no build args. The credential is
optional and minted host-side — see
[docs/omnium-api-key.md](../../docs/omnium-api-key.md).

## Build / use

```bash
docker buildx build -f mixins-v3/omnium/omnium.yaml -t my/omnium-mixin:dev mixins-v3/omnium/
sbx run <agent> . --kit ./mixins-v3/omnium   # or the published image reference
```

Or add it to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/omnium&ref=v3.1.2   # Omnium API
```

## How it works

The sandbox network policy is deny by default, so API calls fail until
the Omnium hosts are allowed. This mixin grants them at runtime.

- **Credential.** Declares `OMNIUM_API_KEY` with `proxyManaged: true`.
  The sandbox only sees a placeholder; the proxy injects the real
  `Authorization: Bearer <token>` header on requests to the Omnium API
  hosts. The token is an API access token (JWT) for the environment you
  target, minted host-side by POSTing the API user's `ClientId` +
  `ClientSecret` to `/api/token` on that environment's API host. Create
  API users in the Omnium GUI under Configuration > Authorization >
  API Users. Full walkthrough:
  [docs/omnium-api-key.md](../../docs/omnium-api-key.md).
- **Docs and status** (`docs.omnium.no`, `help.omnium.no`,
  `status.omnium.no`) are reachable without a token, for research.

## Runtime egress

| Domain | Why |
| ------ | --- |
| `api.omnium.no:443` | Production REST API |
| `apitest.omnium.no:443` | Test REST API |
| `apidev.omnium.no:443` | Dev REST API (shares data with test) |
| `docs.omnium.no:443` | Tech docs |
| `help.omnium.no:443` | User docs |
| `status.omnium.no:443` | System status |
