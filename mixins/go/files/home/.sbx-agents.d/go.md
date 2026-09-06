## Go / Go modules

- Go is baked into the image (`go` resolves to /usr/local/go/bin/go);
  `go get` / `go mod tidy` manage modules; `go install`-ed tools land in
  `~/go/bin` (already on PATH)
- Newer toolchains auto-download on demand (GOTOOLCHAIN=auto, the default):
  a project's `go`/`toolchain` line newer than the baked Go fetches that
  toolchain automatically — no root, no image rebuild
- Private modules: set GOPRIVATE (e.g. `go env -w GOPRIVATE=corp.example/*`)
  so their fetches skip the module proxy
- `gopls` (the LSP) is not baked in — install with
  `go install golang.org/x/tools/gopls@latest`
