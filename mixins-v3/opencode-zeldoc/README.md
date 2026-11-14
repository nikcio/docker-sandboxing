# opencode-zeldoc (v3)

Zeldoc.ai model provider for OpenCode: proxy-managed API key
(`com.docker.sandbox/credential@1`, `service: zeldoc`), the provider +
plugin config merge and the default model-variant seed as lifecycle
startup hooks, and the Zeldoc egress.

Declaration-only: no image content, no build args, nothing to build. The
credential is required and registered host-side with
`sbx secret set zeldoc` — see
[docs/zeldoc-api-key.md](../../docs/zeldoc-api-key.md).

## Build / use

Point a sandbox at the mixin directory (`sbx` composes it at create):

```bash
sbx run <agent> . --kit ./mixins-v3/opencode-zeldoc
```

In a project's `sbxenv.yaml`, add it to the `kits:` list (pin
`&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/opencode-zeldoc   # Zeldoc.ai provider
```

Works on its own: the provider config is merged straight into the global
`~/.config/opencode/opencode.json(c)` (created if missing) at every
start, so it composes with the `opencode` mixin but also works when
opencode was installed by any other means. Composing both provider
mixins is fine: each only touches its own keys.

## How it works

- **Credential.** Declares `ZELDOC_API_KEY` (required) with
  `proxyManaged: true`. The sandbox only sees a placeholder; the proxy
  injects the real `Authorization: Basic <key>` header on requests to
  `api.zeldoc.ai`.
- **Provider config.** A startup hook merges
  `"enabled_providers": ["zeldoc"]` and `"plugin": ["opencode-zeldoc"]`
  into the global `~/.config/opencode/opencode.json(c)` (creating it if
  missing) at every start. It is idempotent and never touches keys owned
  by other mixins. It installs the
  [`opencode-zeldoc` plugin](https://github.com/martinmose/opencode-zeldoc)
  (the model picker lists exactly the models your key can use, with real
  limits and prices). No manual `provider` block with a models list; the
  plugin is the source of truth, per the
  [Zeldoc guide](https://docs.zeldoc.ai/connect-opencode).
- **Default variant.** A startup hook seeds the OpenCode model state
  file (`~/.local/state/opencode/model.json`) with
  `variant: {"zeldoc/zdev": "high"}` so the first TUI launch starts on
  the `high` variant. It only fills the key in when missing; a variant
  you have already picked (cycling persists to this file) is never
  overridden.

## Runtime egress

| Domain | Why |
| ------ | --- |
| `api.zeldoc.ai` | Model API + plugin model discovery (auth injected by the proxy) |
| `zeldoc.ai` | Product site |
| `docs.zeldoc.ai` | Docs (webfetch) |
| `registry.npmjs.org` | Plugin install (`opencode-zeldoc` at start) |
