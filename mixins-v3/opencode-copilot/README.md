# opencode-copilot (v3)

GitHub Copilot model provider for OpenCode: the provider-config merge runs
as a lifecycle startup hook, with the OAuth device-flow egress. v3 port of
[`../opencode-copilot`](../opencode-copilot) — read that for the full
behavior story; see [`../README.md`](../README.md) for what the v3 format
changes.

Declaration-only: no image content, no build args.

## Runtime egress

`github.com` (device flow), `api.github.com` (session token mint),
`api.githubcopilot.com` + wildcard (models + inference).
