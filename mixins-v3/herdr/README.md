# herdr (v3)

The [Herdr](https://herdr.dev) CLI (agent runtime, keeps coding agents
running and resumes them from any machine) baked into the mixin layer at
image build. v3 port of [`../herdr`](../herdr) — read that for the full
behavior story; see [`../README.md`](../README.md) for what the v3 format
changes.

## Build / use

```bash
docker buildx build -f mixins-v3/herdr/herdr.yaml -t my/herdr-mixin:dev mixins-v3/herdr/
sbx run <agent> . --kit ./mixins-v3/herdr   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `herdrVersion` | *(empty = latest)* | Herdr version (`v` prefix optional) |

## Runtime egress

`herdr.dev` (installer + release manifest), `github.com` + wildcard,
`objects.githubusercontent.com` (release binaries for `herdr update`).
No install-phase grant.
