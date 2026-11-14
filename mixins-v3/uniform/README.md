# uniform (v3)

Uniform DXP egress + proxy-managed API key
(`com.docker.sandbox/credential@1`, `service: uniform`). v3 port of
[`../uniform`](../uniform) — read that for the full behavior story; see
[`../README.md`](../README.md) for what the v3 format changes.

Declaration-only: no image content, no build args. Credential is optional
and registered host-side with `sbx secret set uniform` — see
[docs/uniform-api-key.md](../../docs/uniform-api-key.md).

## Runtime egress

`uniform.app` + wildcard (dashboard, Management API, docs, per-team
subdomains), `uniform.global` + wildcard (Edge Delivery API + image CDN,
US/EU).
