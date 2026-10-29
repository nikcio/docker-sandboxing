# omnium (v3)

Omnium API egress + proxy-managed bearer token
(`com.docker.sandbox/credential@1`, `service: omnium`). v3 port of
[`../omnium`](../omnium) — read that for the full behavior story; see
[`../README.md`](../README.md) for what the v3 format changes.

Declaration-only: no image content, no build args. Credential is optional
and minted host-side — see [docs/omnium-api-key.md](../../docs/omnium-api-key.md).

## Runtime egress

`api.omnium.no`, `apitest.omnium.no`, `apidev.omnium.no` (REST API),
`docs.omnium.no`, `help.omnium.no`, `status.omnium.no` (docs/status).
