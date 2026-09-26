# opencode-copilot

GitHub Copilot model provider for OpenCode: the network egress the OAuth
device-flow sign-in needs, plus a provider config fragment so Copilot
models are selectable with `/models`. Runs the agent on your Copilot
subscription instead of an API key.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/opencode-copilot&ref=vX.Y.Z # GitHub Copilot provider
```

Requires the `opencode` mixin — the provider config fragment
only merges through it.

Full sign-in walkthrough: [docs/copilot-setup.md](../../docs/copilot-setup.md).

## How it works

- **Sign-in**: inside the TUI run `/connect`, then complete
  `github.com/login/device` in your host browser. The token is stored
  inside the VM at `~/.local/share/opencode/auth.json` — there is no
  proxy-managed secret (unlike the `opencode-zeldoc` mixin).
- **Config fragment**: ships a pure-JSON fragment to
  `~/.config/opencode/mixins.d/10-copilot.json`, merged into the combined
  `OPENCODE_CONFIG` by the `opencode` mixin's install hook, before the
  agent starts.

## Network domains

| Domain | Why |
| ------ | --- |
| `github.com:443` | OAuth device flow: device code, token exchange, refresh |
| `api.github.com:443` | Copilot session token mint |
| `*.githubcopilot.com:443` | Copilot API (models + inference), incl. business/enterprise plan endpoints |
