# zeldoc

Zeldoc.ai model provider for OpenCode: proxy-managed API key,
`opencode-zeldoc` plugin config, and network rules for the Zeldoc hosts.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/zeldoc   # Zeldoc.ai provider
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/zeldoc <agent> .`

Requires the `opencode` mixin — the provider config fragment
only merges through it (compose `copilot` alongside; `enabled_providers`
lists are unioned, so both providers stay selectable).

## How it works

- **Credential**: declares `ZELDOC_API_KEY` (required) with
  `proxyManaged: true` — the sandbox only sees a placeholder; the proxy
  injects the real `Authorization: Basic <key>` header on requests to
  `api.zeldoc.ai`. Register the key host-side with
  `sbx secret set zeldoc` (see [docs/zeldoc-api-key.md](../../docs/zeldoc-api-key.md)).
- **Config fragment**: ships a pure-JSON fragment to
  `~/.config/opencode/mixins.d/20-zeldoc.json`, merged into the combined
  `OPENCODE_CONFIG` by the `opencode` mixin's install hook, before the
  agent starts. It enables
  the `zeldoc` provider, installs the
  [`opencode-zeldoc` plugin](https://github.com/martinmose/opencode-zeldoc)
  (the model picker lists exactly the models your key can use, with real
  limits and prices). No manual `provider` block
  with a models list — the plugin is the source of truth, per the
  [Zeldoc guide](https://docs.zeldoc.ai/connect-opencode).

## Network domains

| Domain | Why |
| ------ | --- |
| `api.zeldoc.ai:443` | Model API + plugin model discovery (auth injected by the proxy) |
| `zeldoc.ai:443` | Product site |
| `docs.zeldoc.ai:443` | Docs (webfetch) |
| `registry.npmjs.org:443` | Plugin install (`opencode-zeldoc` at start) |
