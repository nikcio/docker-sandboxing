# opencode-copilot (v3)

GitHub Copilot model provider for OpenCode: a lifecycle startup hook that
enables the `github-copilot` provider in the global opencode config, plus
the egress the OAuth device-flow sign-in needs. Runs the agent on your
Copilot subscription instead of an API key.

Declaration-only: no image content, no build args, nothing to build.

## Build / use

Point a sandbox at the mixin directory (`sbx` composes it at create):

```bash
sbx run <agent> . --kit ./mixins-v3/opencode-copilot
```

In a project's `sbxenv.yaml`, add it to the `kits:` list (pin
`&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/opencode-copilot   # GitHub Copilot provider
```

Works on its own: the provider config is merged straight into the global
`~/.config/opencode/opencode.json(c)` (created if missing) at every
start, so it composes with the `opencode` mixin but also works when
opencode was installed by any other means. Composing both provider
mixins is fine: each only touches its own keys.

Full sign-in walkthrough: [docs/copilot-setup.md](../../docs/copilot-setup.md).

## How it works

- **Sign-in.** Inside the TUI run `/connect`, then complete
  `github.com/login/device` in your host browser. The token is stored
  inside the VM at `~/.local/share/opencode/auth.json`. There is no
  proxy-managed secret (unlike the `opencode-zeldoc` mixin).
- **Provider config.** The startup hook merges
  `"enabled_providers": ["github-copilot"]` into the global
  `~/.config/opencode/opencode.json(c)` (creating it if missing) at every
  start. It is idempotent and never touches keys owned by other mixins.

## Runtime egress

| Domain | Why |
| ------ | --- |
| `github.com` | OAuth device flow: device code, token exchange, refresh |
| `api.github.com` | Copilot session token mint |
| `api.githubcopilot.com` + wildcard | Copilot API (models + inference), incl. business/enterprise plan endpoints |
