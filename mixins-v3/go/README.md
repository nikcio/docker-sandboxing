# go (v3)

Go toolchain baked into the mixin layer at image build, with module-proxy
egress. v3 port of [`../go`](../go) — read that for the full behavior
story; see [`../README.md`](../README.md) for what the v3 format changes.

## Build / use

```bash
docker buildx build -f mixins-v3/go/go.yaml -t my/go-mixin:dev mixins-v3/go/
sbx run <agent> . --kit ./mixins-v3/go   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `goVersion` | *(empty = latest)* | Go version (`goMAJOR` or `goMAJOR.MINOR[.PATCH]`) |

## Runtime egress

`proxy.golang.org`, `sum.golang.org`, `go.dev` + wildcard, `golang.org` +
wildcard, `dl.google.com`. No install-phase grant.
