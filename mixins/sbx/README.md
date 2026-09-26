# sbx

Installs the **Docker Sandboxes CLI** (`docker-sbx`, the `sbx` command) inside the sandbox for kit authoring: `sbx validate`, `sbx inspect`, `sbx pack`.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/sbx&ref=vX.Y.Z   # sbx CLI inside the sandbox
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/sbx <workload> .`

The install runs at **sandbox creation only**. Skipped when the template image already ships the CLI.

## How it works

- Adds Docker's apt repo and keyring, then installs the `docker-sbx` package from it, using an isolated apt list directory (the same pattern as the other mixins) so the sandbox's apt state stays clean.
- The Docker apt repo stays configured so `apt upgrade` keeps the CLI current.

## Network domains

| Domain | Why |
| ------ | --- |
| `download.docker.com:443` | Docker apt repo: the `docker-sbx` package + keyring, and upgrades |
