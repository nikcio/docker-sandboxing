# zsh (v3)

zsh + [Oh My Zsh](https://github.com/ohmyzsh/ohmyzsh) baked into the
mixin layer at image build, with a startup hook making zsh the agent's
login shell and copying the Oh My Zsh tree + a seeded `.zshrc` into the
agent home — so interactive shells start in zsh with the stock Oh My Zsh
config.

## Build / use

```bash
docker buildx build -f mixins-v3/zsh/zsh.yaml -t my/zsh-mixin:dev mixins-v3/zsh/
sbx run <agent> . --kit ./mixins-v3/zsh   # or the published image reference
```

In a project's `sbxenv.yaml`, add the mixin to the `kits:` list (pin
`&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/zsh   # zsh + Oh My Zsh
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `zshVersion` | *(empty = latest apt candidate)* | zsh apt package version — the full version string including the distro revision (codename-specific) |
| `ohMyZshRef` | *(empty = master HEAD)* | Oh My Zsh git ref to clone (`master`, a tag, …) |

Args resolve when the image builds; pass them with an `args:` entry on
the mixin's `kits:` line.

## How it works

- **Baked install.** At image build, the `zsh` package is installed via
  apt and Oh My Zsh is cloned (shallow) under `/opt/oh-my-zsh` (a
  mixin's layers are an overlay on an unknown base, so its content must
  not claim `/home/agent` paths the base or the agent user owns). The
  whole zsh file tree ships, not just the binary: zsh loads its modules
  and package function files from its system paths — a binary-only layer
  starts but dies with `failed to load module zsh/parameter` on first
  use.
- **Startup hook.** Runs on every boot, idempotent, so a base whose
  login shell or `.zshrc` changed underneath (or a fresh volume) gets
  the same setup re-applied:
  - Points the `agent` user's login shell at `/usr/bin/zsh` by editing
    `/etc/passwd` directly — `chsh` goes through PAM, which fails on
    bases where the PAM stack can't authenticate the passwordless agent
    user.
  - Copies the Oh My Zsh tree into `~/.oh-my-zsh` (agent-owned). It must
    live in the agent home: the stock `.zshrc` template points `$ZSH`
    there, and a root-owned tree would break compinit's cache and omz
    updates.
  - Seeds `~/.zshrc` from the Oh My Zsh template (never overwrites an
    existing one).
- The agent's `~/.zshrc` is agent-owned; the agent is free to customize
  plugins/themes afterwards.
- Scripts and `exec` calls still run through bash; only interactive/
  login shells pick up zsh.

## Runtime egress

`github.com` + wildcard and `raw.githubusercontent.com` (Oh My Zsh repo
— framework updates), `ohmyz.sh` (docs/wiki referenced by the
framework). No install-phase grant: the install is baked at build.
