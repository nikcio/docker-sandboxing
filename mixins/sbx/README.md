# sbx

Installs the **Docker Sandboxes CLI** (`docker-sbx`, the `sbx` command) inside the sandbox for kit authoring: `sbx validate`, `sbx inspect`, `sbx pack`.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/sbx   # sbx CLI inside the sandbox
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/sbx <agent> .`

The install runs at **sandbox creation only**. Skipped when the base image already ships the CLI.

## How it works

- Adds Docker's apt repo and keyring, then installs the `docker-sbx` package from it, using an isolated apt list directory (the same pattern as the other mixins) so the sandbox's apt state stays clean.
- The Docker apt repo stays configured so `apt upgrade` keeps the CLI current.

## Version arguments

| Arg | Default | Description |
| --- | ------- | ----------- |
| `sbxVersion` | *(empty = latest apt candidate)* | `docker-sbx` apt package version — use the full version including the `~ubuntu...` suffix; apt rejects prefixes with `=` |

Pass values with `--kit-arg sbx.sbxVersion=<version>` (or an `args:` entry on the mixin's `kits:` line in `sbxenv.yaml`) — see [Version arguments](../../docs/mixins.md#version-arguments).

## Network domains

| Domain | Why |
| ------ | --- |
| `download.docker.com:443` | Docker apt repo: the `docker-sbx` package + keyring, and upgrades |
