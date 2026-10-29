# opencode-zeldoc (v3)

Zeldoc.ai model provider for OpenCode: proxy-managed API key
(`com.docker.sandbox/credential@1`, `service: zeldoc`), the provider +
plugin config merge and model-state seed as lifecycle startup hooks, and
the Zeldoc egress. v3 port of [`../opencode-zeldoc`](../opencode-zeldoc) —
read that for the full behavior story; see [`../README.md`](../README.md)
for what the v3 format changes.

Declaration-only: no image content, no build args. The credential is
optional (skip when unbound) and registered host-side with
`sbx secret set zeldoc` — see [docs/zeldoc-api-key.md](../../docs/zeldoc-api-key.md).

## Runtime egress

`api.zeldoc.ai`, `zeldoc.ai`, `docs.zeldoc.ai`, `registry.npmjs.org`
(plugin install at start).
