# go (v3)

Go toolchain baked into the mixin layer at image build, with module-proxy
egress.

## Build / use

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
`&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/go   # Go toolchain
```

For local development, point `--kit` at the directory instead —
`sbx run <agent> . --kit ./mixins-v3/go` — or build the image yourself:

```bash
docker buildx build -f mixins-v3/go/go.yaml -t my/go-mixin:dev mixins-v3/go/
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `goVersion` | *(empty = latest)* | Go version (`goMAJOR` or `goMAJOR.MINOR[.PATCH]`) |

## How it works

- The official tarball for the build architecture is unpacked to `/opt/go`,
  with `go`/`gofmt` symlinked onto `/usr/local/bin`. A `/etc/profile.d/`
  script adds `${HOME}/go/bin` to `PATH` so `go install`ed binaries resolve
  in login shells; nothing in the mixin claims the agent home — Go writes
  there at the user's request.
- The version (empty = latest, resolved from `go.dev/VERSION`) is resolved
  at build, so the image is a snapshot: rebuild (or pin via the build arg)
  to update it.
- With `GOTOOLCHAIN=auto`, `go` downloads newer toolchains as modules from
  the proxy — covered by the egress below, with `sum.golang.org` (the
  default `GOSUMDB`) verifying every module and toolchain download.

## Runtime egress

`proxy.golang.org` (module proxy: `go get`, `go mod download`,
`go install`, plus toolchain downloads), `sum.golang.org`, `go.dev` +
wildcard (docs, `/dl/` download redirects, release info), `golang.org` +
wildcard (redirects to go.dev; `x/` module paths resolve via the proxy),
`dl.google.com` (download targets behind `go.dev/dl`, e.g. `golang.org/dl`
wrapper SDKs: `go install golang.org/dl/go1.x@latest`, then
`go1.x download`). No install-phase grant: the toolchain is baked at
build.
