# herdr (v3)

The [Herdr](https://herdr.dev) CLI (agent runtime, keeps coding agents
running and resumes them from any machine) baked into the mixin layer at
image build. Herdr is a background server that owns your agents'
terminals: agents keep working when you detach, and you reconnect from
any machine and find them where you left them.

## Build / use

```bash
docker buildx build -f mixins-v3/herdr/herdr.yaml -t my/herdr-mixin:dev mixins-v3/herdr/
sbx run <agent> . --kit ./mixins-v3/herdr   # or the published image reference
```

In a project's `sbxenv.yaml`, add the mixin to the `kits:` list (pin
`&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/herdr   # Herdr agent runtime
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `herdrVersion` | *(empty = latest)* | Herdr release to install, with or without the leading `v` |

Args resolve when the image builds; pass them with an `args:` entry on
the mixin's `kits:` line.

## How it works

- **Baked install.** At image build, the official installer
  (`curl -fsSL https://herdr.dev/install.sh`) runs, pinned to
  `/usr/local/bin` so the binary is on `PATH` for every user. The
  installer verifies the SHA-256 checksum of the downloaded binary
  against the release manifest. A pinned `herdrVersion` skips the
  installer and downloads the release binary from GitHub releases
  directly (x86_64/aarch64).
- **Updates.** `herdr update` works inside the sandbox: it downloads new
  releases from GitHub releases (covered by the network rules).

## Runtime egress

| Domain | Why |
| ------ | --- |
| `herdr.dev` | Installer script + release manifest (`latest.json`) |
| `github.com` + wildcard, `objects.githubusercontent.com` | Release binaries (pinned install at build + `herdr update` inside the sandbox) |

No install-phase grant: the install is baked at build.
