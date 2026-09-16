# opencode-update

Rolls opencode to the newest npm release at sandbox creation — the
template images bake a fixed opencode version, and this mixin updates it.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/opencode-update   # opencode auto-update
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/opencode-update <agent> .`

Not in the stock kits — opt in by adding the line. Runs once at creation,
no start cost.

## How it works

- Skipped when `opencode` is not on `PATH`, npm is unavailable, or the
  installed opencode is not npm-managed (a different install layout is
  left alone).
- Otherwise runs `npm install -g --prefix <opencode-prefix> opencode-ai@latest`
  as the `agent` user, in-place, and prints the new version.
- Compose the `node` mixin when the template image lacks npm — the update
  step needs it.

## Network domains

| Domain | Why |
| ------ | --- |
| `registry.npmjs.org:443` | The `opencode-ai` npm package download |
