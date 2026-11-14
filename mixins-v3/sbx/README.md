# sbx (v3)

The Docker Sandboxes CLI (`docker-sbx`, the `sbx` command) baked into the
mixin layer at image build, for kit authoring inside the sandbox:
`sbx validate`, `sbx inspect`, `sbx pack`.

## Build / use

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
`&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/sbx   # sbx CLI inside the sandbox
```

For local development, point `--kit` at the directory instead —
`sbx run <agent> . --kit ./mixins-v3/sbx` — or build the image yourself:

```bash
docker buildx build -f mixins-v3/sbx/sbx.yaml -t my/sbx-mixin:dev mixins-v3/sbx/
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `sbxVersion` | *(empty = latest apt candidate)* | `docker-sbx` apt package version — use the full version including the `~ubuntu...` suffix; apt rejects prefixes with `=` |

## How it works

- Docker's apt repo + keyring are added and the `docker-sbx` package is
  installed at build; the `sbx` binary, the repo definition, and the
  keyring are copied into the mixin layer. The repo stays configured in
  the sandbox, so `apt upgrade` keeps the CLI current — hence the
  `download.docker.com` egress.
- The build stage uses `ubuntu:24.04`: the Docker apt repo publishes no
  Debian suites, and the sandbox bases this mixin targets are Ubuntu.
- The resolved version is baked at build, so the image is a snapshot:
  rebuild (or pin via the build arg) to update it.

## Runtime egress

`download.docker.com` (Docker apt repo: the `docker-sbx` package +
keyring are baked at build; kept for in-sandbox upgrades). No
install-phase grant.
