## Go / Go modules

- Go is baked into the image (`go` resolves to /usr/local/go/bin/go);
  `go install`-ed tools land in `~/go/bin` (already on PATH)
- Modules come from proxy.golang.org, verified against sum.golang.org —
  both allowed by this mixin
- Newer toolchains auto-download on demand (GOTOOLCHAIN=auto, the default):
  a project's `go`/`toolchain` line newer than the baked Go fetches that
  toolchain as a module through the proxy — no root, no image rebuild
- Private modules: set GOPRIVATE (e.g. `go env -w GOPRIVATE=corp.example/*`)
  and compose the `git` mixin so the VCS fetches can reach the module host
- `gopls` (the LSP) is not baked in — install with
  `go install golang.org/x/tools/gopls@latest`
- Docs lookups: go.dev and pkg.go.dev are reachable (language docs, module
  docs, release notes)
