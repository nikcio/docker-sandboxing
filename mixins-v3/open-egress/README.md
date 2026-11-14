# open-egress (v3)

Allows all outbound domains (the `**` rule, replacing the deny-by-default
network baseline) in both the install and runtime phases.

Declaration-only: no image content, no build args. Opt-in.

## Build / use

```bash
docker buildx build -f mixins-v3/open-egress/open-egress.yaml -t my/open-egress-mixin:dev mixins-v3/open-egress/
sbx run <agent> . --kit ./mixins-v3/open-egress   # or the published image reference
```

Or add it to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/open-egress&ref=v3.1.2   # allow all outbound domains
```

## How it works

The sandbox network policy is deny by default. With this mixin, the agent
can reach any host. Prefer listing the specific domains a mixin owns;
local deny rules and org policy still take precedence over the allow-all.

The install phase is opened too: with all egress open at runtime, an
install-phase gap would be the only thing that could still fail a
sandbox create.
