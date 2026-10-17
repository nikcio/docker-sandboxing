# opencode-copilot

GitHub Copilot model provider for OpenCode: the network egress the OAuth device-flow sign-in needs, plus a startup command that enables the `github-copilot` provider in the global opencode config so Copilot models are selectable with `/models`. Runs the agent on your Copilot subscription instead of an API key.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/opencode-copilot   # GitHub Copilot provider
```

Works on its own. The provider config is merged straight into the global `~/.config/opencode/opencode.json(c)` (created if missing) at every start, so it composes with the `opencode` mixin but also works when opencode was installed by any other means. Composing both provider mixins is fine: each only touches its own keys.

Full sign-in walkthrough: [docs/copilot-setup.md](../../docs/copilot-setup.md).

## How it works

- **Sign-in.** Inside the TUI run `/connect`, then complete `github.com/login/device` in your host browser. The token is stored inside the VM at `~/.local/share/opencode/auth.json`. There is no proxy-managed secret (unlike the `opencode-zeldoc` mixin).
- **Provider config.** A startup command merges `"enabled_providers": ["github-copilot"]` into the global `~/.config/opencode/opencode.json(c)` (creating it if missing) at every start. It is idempotent and never touches keys owned by other mixins.

## Network domains

| Domain | Why |
| ------ | --- |
| `github.com:443` | OAuth device flow: device code, token exchange, refresh |
| `api.github.com:443` | Copilot session token mint |
| `*.githubcopilot.com:443` | Copilot API (models + inference), incl. business/enterprise plan endpoints |
