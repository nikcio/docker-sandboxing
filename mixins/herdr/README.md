# herdr

Installs the [**Herdr**](https://herdr.dev) CLI — the runtime coding agents
run on. Herdr is a background server that owns your agents' terminals:
agents keep working when you detach, and you reconnect from any machine
and find them where you left them.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/herdr   # Herdr agent runtime
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/herdr <agent> .`

The install runs at **sandbox creation only** and needs egress — the
domains in the table below. Skipped when the base image already
ships `herdr`.

## How it works

- **Check-and-install**: when `herdr` is missing, runs the official
  installer (`curl -fsSL https://herdr.dev/install.sh | sh`) pinned to
  `/usr/local/bin` so it is on `PATH` for every user. The installer
  verifies the SHA-256 checksum of the downloaded binary against the
  release manifest before installing.
- **Updates**: `herdr update` works inside the sandbox — it downloads
  new releases from GitHub releases (covered by the network rules).

## Version arguments

| Arg | Default | Description |
| --- | ------- | ----------- |
| `herdrVersion` | *(empty = latest)* | Herdr release to install (`0.9.5`) |

Pass values with `--kit-arg herdr.herdrVersion=0.9.5` (or an `args:`
entry on the mixin's `kits:` line in `sbxenv.yaml`) — see
[Version arguments](../../docs/mixins.md#version-arguments).

## Network domains

| Domain | Why |
| ------ | --- |
| `herdr.dev:443` | Installer script + release manifest (`latest.json`) |
| `github.com`, `*.github.com`, `objects.githubusercontent.com` | Release binaries (install + `herdr update`) |
