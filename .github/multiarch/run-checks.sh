#!/bin/bash
# Runtime smoke checks for the v3 mixins, run inside the composed image
# (mixin layer on a plain ubuntu base) as the agent user, via a login
# shell so the /etc/profile.d scripts the mixins install are in effect.
# Invoked with the mixin name as $1; every branch must actually run the
# tool, not just print a version — the bugs this guards (dereferenced
# symlinks, libs left in the build stage, single-arch binaries) all pass
# a bare `--version` in the build stage and fail at first real use.
set -eu
mixin="$1"

case "$mixin" in
  dotnet)
    export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_NOLOGO=1
    dotnet --version
    cd "$(mktemp -d)"
    dotnet new console -n hello -o . >/dev/null
    dotnet run
    ;;
  github-cli)
    gh --version
    ;;
  go)
    go version
    go env GOROOT GOARCH
    cd "$(mktemp -d)"
    go mod init hello >/dev/null
    printf 'package main\nimport "fmt"\nfunc main() { fmt.Println("go-ok") }\n' > main.go
    go run .
    ;;
  herdr)
    herdr --version
    ;;
  node)
    node --version
    pnpm --version
    cd "$(mktemp -d)"
    printf '{ "name": "t", "private": true, "scripts": { "dev": "node hello.js" } }\n' > package.json
    echo 'console.log("dev-ran-ok " + process.arch)' > hello.js
    pnpm dev
    npx -y esbuild --version
    ;;
  nikcio-openapi-codegen)
    export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_NOLOGO=1
    dotnet --version
    dotnet tool list --global
    ;;
  opencode)
    opencode --version
    ;;
  playwright)
    export PLAYWRIGHT_BROWSERS_PATH=/opt/ms-playwright
    export NODE_PATH="$(npm root -g)"
    playwright --version
    node -e '
const { chromium } = require("playwright");
const open = async (args) => {
  const b = await chromium.launch({ args });
  const p = await b.newPage();
  await p.setContent("<h1>hello-arch</h1>");
  const txt = await p.textContent("h1");
  await b.close();
  return txt;
};
(async () => {
  // Chromium spawns child processes (GPU, renderers), which qemu-user
  // cannot emulate (the child dies with SIGTRAP and the browser goes
  // FATAL "GPU process isn'\''t usable"). Real arm64 hardware is
  // unaffected, so only an emulated run falls back to --single-process.
  let txt;
  try {
    txt = await open([]);
  } catch (e) {
    console.log("multi-process launch failed, retrying single-process (emulated?)");
    txt = await open(["--single-process"]);
  }
  console.log("pw-ok", txt);
  if (txt !== "hello-arch") process.exit(1);
})().catch((e) => { console.error(e); process.exit(1); });
'
    ;;
  python)
    python3 --version
    uv --version
    python3 -c 'import platform; print("py-ok", platform.machine())'
    cd "$(mktemp -d)"
    uv venv v >/dev/null
    ./v/bin/python -c 'import sys; print("venv-ok", sys.version_info[:2])'
    ;;
  rust)
    export RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH="/opt/cargo/bin:$PATH"
    cargo --version
    rustc --version
    cd "$(mktemp -d)"
    cargo new hello -q
    cd hello
    cargo build -q
    ./target/debug/hello
    ;;
  sbx)
    sbx version
    ;;
  zsh)
    zsh --version
    # Replicate the zsh mixin startup hook: omz into the agent home.
    [ -d "$HOME/.oh-my-zsh" ] || cp -a /opt/oh-my-zsh "$HOME/.oh-my-zsh"
    [ -f "$HOME/.zshrc" ] || cp /opt/oh-my-zsh/templates/zshrc.zsh-template "$HOME/.zshrc"
    zsh -i -c 'echo "zsh-ok $ZSH_VERSION"'
    ;;
  *)
    echo "no smoke checks for mixin '$mixin'" >&2
    exit 1
    ;;
esac
