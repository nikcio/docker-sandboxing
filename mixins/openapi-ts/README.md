# openapi-ts

Docs egress for the [openapi-typescript](https://openapi-ts.dev) and
[openapi-fetch](https://openapi-ts.dev) packages, so the agent can look up
usage and reference material while generating typed API clients.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/openapi-ts   # openapi-ts docs
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/openapi-ts <agent> .`

Pair it with the `node` mixin — package installs themselves flow through
the npm registry rules that mixin owns.

## Network domains

| Domain | Why |
| ------ | --- |
| `openapi-ts.dev:443`, `*.openapi-ts.dev:443` | Docs site for openapi-typescript / openapi-fetch |
