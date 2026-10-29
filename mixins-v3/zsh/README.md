# zsh (v3)

zsh + Oh My Zsh baked into the mixin layer at image build (tree under
`/opt/oh-my-zsh`), with a startup hook making zsh the agent's login shell
and seeding `.zshrc`. v3 port of [`../zsh`](../zsh) — read that for the
full behavior story; see [`../README.md`](../README.md) for what the v3
format changes.

## Build / use

```bash
docker buildx build -f mixins-v3/zsh/zsh.yaml -t my/zsh-mixin:dev mixins-v3/zsh/
sbx run <agent> . --kit ./mixins-v3/zsh   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `zshVersion` | *(empty = latest apt candidate)* | zsh apt package version |
| `ohMyZshRef` | *(empty = master HEAD)* | Oh My Zsh git ref |

## Runtime egress

`github.com` + wildcard, `raw.githubusercontent.com` (Oh My Zsh updates),
`ohmyz.sh` (docs/wiki).
