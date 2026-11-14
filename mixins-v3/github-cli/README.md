# github-cli (v3)

GitHub CLI (`gh`) baked into the mixin layer at image build, with gh and
git-over-HTTPS egress. v3 port of [`../github-cli`](../github-cli) — read
that for the full behavior story (including the git-identity setup and why
there is no credential entry here); see [`../README.md`](../README.md) for
what the v3 format changes.

## Differences from the v2 mixin

- `gh` is baked at build; the v2 apt-repo install hook is gone.
- The v2 startup git-identity hook is **not** carried: identity belongs to
  the workload's `git-identity@1` capability (or the github credential's
  own setup) in v3, not to a tool mixin re-deriving it from `GH_TOKEN`.
  Compose with a v3 workload that declares it.
- The apt repo definition ships in the layer, so `apt upgrade` inside the
  sandbox keeps updating gh.

## Build / use

```bash
docker buildx build -f mixins-v3/github-cli/github-cli.yaml -t my/gh-mixin:dev mixins-v3/github-cli/
sbx run <agent> . --kit ./mixins-v3/github-cli   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `ghVersion` | *(empty = latest apt package)* | gh version |

## Runtime egress

`cli.github.com`, `github.com` + wildcard, `*.githubusercontent.com`. No
install-phase grant.
