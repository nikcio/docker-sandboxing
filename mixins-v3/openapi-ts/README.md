# openapi-ts (v3)

Docs egress for the [openapi-typescript](https://openapi-ts.dev) and
[openapi-fetch](https://openapi-ts.dev) packages, so the agent can look
up usage and reference material while generating typed API clients.

Declaration-only: no image content, no build args.

## Build / use

```bash
docker buildx build -f mixins-v3/openapi-ts/openapi-ts.yaml -t my/openapi-ts-mixin:dev mixins-v3/openapi-ts/
sbx run <agent> . --kit ./mixins-v3/openapi-ts   # or the published image reference
```

Or add it to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/openapi-ts&ref=v3.1.2   # openapi-ts docs
```

Pair it with the `node` mixin. Package installs themselves flow through
the npm registry rules that mixin owns.

## Runtime egress

| Domain | Why |
| ------ | --- |
| `openapi-ts.dev:443`, `*.openapi-ts.dev:443` | Docs site for openapi-typescript / openapi-fetch |
