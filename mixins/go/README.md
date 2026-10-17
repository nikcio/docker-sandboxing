# go

Go toolchain egress for the sandbox: the toolchain install (when the base image lacks it), module proxy + checksum database for `go get`/`go install`, and docs hosts.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/go   # Go toolchain
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/go <agent> .`

The install runs at **sandbox creation only** and needs egress — the domains in the table below plus the Ubuntu apt mirrors (`archive.ubuntu.com`/`security.ubuntu.com`) for its `apt-get` calls. On base images that already ship Go the install is a cheap no-op.

## How it works

- **Check-and-install**: when `go` is missing, installs the official tarball for the running architecture into `/usr/local/go` (the same layout as the base images) and symlinks `go`/`gofmt` onto `PATH`.
- `~/go/bin` is added to `PATH` via `/etc/sandbox-persistent.sh` so `go install`ed binaries are picked up in login shells.
- With `GOTOOLCHAIN=auto` (template default), newer toolchain downloads also flow through the module proxy — covered by the rules below.

## Version arguments

| Arg | Default | Description |
| --- | ------- | ----------- |
| `goVersion` | *(empty = latest)* | Go version to install (`go1.27.2`) |

Pass values with `--kit-arg go.goVersion=go1.26.0` (or an `args:` entry on the mixin's `kits:` line in `sbxenv.yaml`) — see [Version arguments](../../docs/mixins.md#version-arguments).

## Network domains

| Domain | Why |
| ------ | --- |
| `proxy.golang.org:443` | Module proxy (`go get`, `go mod download`, `go install`) + `GOTOOLCHAIN` downloads |
| `sum.golang.org:443` | Checksum database verifying every module/toolchain download |
| `go.dev`, `*.go.dev` | Docs, `/dl/` download redirects, release info |
| `golang.org`, `*.golang.org` | Redirects to go.dev; x/ module paths via the proxy |
| `dl.google.com:443` | go.dev/dl download targets (e.g. `golang.org/dl` wrapper SDKs) |
