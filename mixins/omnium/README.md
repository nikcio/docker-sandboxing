# omnium

Omnium OMS/e-commerce API egress for the sandbox: the REST API hosts
(production/test/dev, each with a Swagger UI under `/documentation`),
tech docs, and proxy-managed `Authorization: Bearer` auth.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/omnium&ref=vX.Y.Z   # Omnium API
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/omnium <workload> .`

The sandbox network policy is deny-by-default, so API calls fail until the
Omnium hosts are allowed.

## How it works

- **Credential**: declares `OMNIUM_API_KEY` with `proxyManaged: true` —
  the sandbox only sees a placeholder; the proxy injects the real
  `Authorization: Bearer <token>` header on requests to the Omnium API
  hosts. The token is an API access token (JWT) for the environment you
  target, minted host-side by POSTing the API user's `ClientId` +
  `ClientSecret` to `/api/token` on that environment's API host — create
  API users in the Omnium GUI under Configuration > Authorization > API
  Users. Full walkthrough: [docs/omnium-api-key.md](../../docs/omnium-api-key.md).
- **Docs and status** are reachable without a token, for research.

## Network domains

| Domain | Why |
| ------ | --- |
| `api.omnium.no:443` | Production REST API |
| `apitest.omnium.no:443` | Test REST API |
| `apidev.omnium.no:443` | Dev REST API (shares data with test) |
| `docs.omnium.no:443` | Tech docs |
| `help.omnium.no:443` | User docs |
| `status.omnium.no:443` | System status |
