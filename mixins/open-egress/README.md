# open-egress

Allows all outbound domains (the `**` rule) — replaces the deny-by-default
network baseline for the sandbox.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/open-egress   # allow all outbound domains
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/open-egress <agent> .`

**Opt-in**: the stock kits and examples do not compose it. With it, the
agent can reach any host — prefer listing the specific domains a mixin
owns. Local deny rules and org policy still take precedence over the
allow-all.
