# zsh

Installs **zsh** with [Oh My Zsh](https://github.com/ohmyzsh/ohmyzsh) for
the `agent` user and makes zsh the login shell — interactive shells start
in zsh with the stock Oh My Zsh config.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/zsh   # zsh + Oh My Zsh
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/zsh <agent> .`

The install runs at **sandbox creation only** and needs egress — the
domains in the table below plus the Ubuntu apt mirrors
(`archive.ubuntu.com`/`security.ubuntu.com`) for its `apt-get` calls. On
template images that already ship zsh + Oh My Zsh the install is a cheap
no-op.

## How it works

- **Check-and-install**: when zsh or Oh My Zsh are missing, installs the
  `zsh` package via apt, clones Oh My Zsh (shallow) into the agent home,
  seeds `~/.zshrc` from the Oh My Zsh template (never overwrites an
  existing one), and points the `agent` user's login shell at
  `/usr/bin/zsh`.
- The agent's `~/.zshrc` is agent-owned — the agent is free to customize
  plugins/themes afterwards.
- Scripts and `exec` calls still run through bash; only interactive/login
  shells pick up zsh.

## Network domains

| Domain | Why |
| ------ | --- |
| `github.com`, `*.github.com`, `raw.githubusercontent.com` | Oh My Zsh repo clone + updates |
| `ohmyz.sh:443` | Oh My Zsh docs/wiki (referenced by the framework) |
