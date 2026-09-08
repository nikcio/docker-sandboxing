# shellcheck shell=bash
# Toolchain install library — shipped by the base mixin
# (mixins/base/files/home/.sandbox-kit/lib/install-toolchain.sh).
#
# Lets any mixin compose on top of any base template image: the check-
# and-install helpers below install the toolchain at sandbox creation
# only when the template doesn't already have it (source it in a
# `setup.install` step: `. "$HOME/.sandbox-kit/lib/install-toolchain.sh" &&
# install_<toolchain>`).
#
# Install paths mirror the template images (template-*/Dockerfile) so the
# same toolchain, same versions and same PATH entries are produced
# whether the image baked it in or this script installs it now.
# Requires root (kit install commands run as root by default) and a
# Debian/Ubuntu base with apt-get + curl.
#
# Network egress is the composing mixins' responsibility: the apt mixin
# covers OS package mirrors, and each toolchain mixin's allowlist covers
# its own downloads (same hosts the template builds used). A failed
# download aborts the install step — fix the composition (compose the
# owning mixin, or check `sbx policy log`) and recreate the sandbox.
#
# Sourced, never executed: every helper is a function; running this file
# directly does nothing.
set -u

# Mark loaded (idempotent sourcing from multiple install steps).
export SBX_INSTALL_LIB_LOADED=1

# Common Ubuntu toolchain packages every install path may need. No-op
# when the image lacks apt-get (non-Debian bases).
__sbx_apt_prepare() {
  command -v apt-get >/dev/null 2>&1 || return 1
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  apt-get install -y -qq --no-install-recommends ca-certificates curl xz-utils
}

# Write persistent shell env (survives restarts, login shells, agent bash
# commands) — idempotent per line.
__sbx_persist() {
  # shellcheck disable=SC2124
  __line="$1"
  touch /etc/sandbox-persistent.sh
  grep -qxF "$__line" /etc/sandbox-persistent.sh \
    || printf '\n%s\n' "$__line" >> /etc/sandbox-persistent.sh
  unset __line
}

# install_dotnet — .NET SDK (template: template-node-dotnet, apt 10.0
# channel; falls back to the dotnet-install script pinned to the same
# channel when the apt feed lacks the codename).
install_dotnet() {
  if command -v dotnet >/dev/null 2>&1; then return 0; fi
  echo "[install-toolchain] dotnet missing — installing (channel ${SBX_DOTNET_CHANNEL:-10.0})"
  __sbx_apt_prepare
  if apt-get install -y -qq --no-install-recommends "dotnet-sdk-${SBX_DOTNET_CHANNEL:-10.0}" 2>/dev/null; then
    __sbx_persist 'export DOTNET_CLI_TELEMETRY_OPTOUT=1'
    __sbx_persist 'export DOTNET_NOLOGO=1'
    __sbx_persist 'export DOTNET_GENERATE_ASPNET_CERTIFICATE=false'
    __sbx_persist 'export NUGET_XMLDOC_MODE=skip'
    command -v dotnet >/dev/null 2>&1 && dotnet --version && return 0
  fi
  echo "[install-toolchain] apt feed lacked dotnet-sdk — using dot.net install script"
  DOTNET_CHANNEL="${SBX_DOTNET_CHANNEL:-10.0}" DOTNET_ROOT="${DOTNET_ROOT:-/usr/share/dotnet}" \
    bash -c '
      set -e
      curl -fsSL https://dot.net/v1/dotnet-install.sh -o /tmp/sbx-dotnet-install.sh
      chmod +x /tmp/sbx-dotnet-install.sh
      /tmp/sbx-dotnet-install.sh --channel "$DOTNET_CHANNEL" --install-dir "$DOTNET_ROOT"
      rm -f /tmp/sbx-dotnet-install.sh
      ln -sf "$DOTNET_ROOT/dotnet" /usr/local/bin/dotnet
    '
  mkdir -p "${DOTNET_ROOT:-/usr/share/dotnet}/tools"
  __sbx_persist "export DOTNET_ROOT=${DOTNET_ROOT:-/usr/share/dotnet}"
  __sbx_persist "export PATH=\${PATH}:${DOTNET_ROOT:-/usr/share/dotnet}/tools"
  __sbx_persist 'export DOTNET_CLI_TELEMETRY_OPTOUT=1'
  __sbx_persist 'export DOTNET_NOLOGO=1'
  __sbx_persist 'export NUGET_XMLDOC_MODE=skip'
  dotnet --version
}

# install_node — Node.js via nvm (template: template-node-dotnet, nvm
# pinned in the agent home, default-alias node symlinked into
# /usr/local/bin). NVM_DIR is respected when the base image already
# manages one.
install_node() {
  if command -v node >/dev/null 2>&1; then return 0; fi
  echo "[install-toolchain] node missing — installing via nvm (major ${SBX_NODE_VERSION:-22})"
  NVM_VERSION="${SBX_NVM_VERSION:-v0.40.3}"
  NODE_VERSION="${SBX_NODE_VERSION:-22}"
  NVM_DIR="${NVM_DIR:-/home/agent/.nvm}"
  __sbx_apt_prepare
  git clone -q --depth 1 --branch "$NVM_VERSION" https://github.com/nvm-sh/nvm.git "$NVM_DIR"
  # nvm refuses to run while the template's NPM_CONFIG_PREFIX is set.
  unset NPM_CONFIG_PREFIX
  # shellcheck disable=SC1091
  . "$NVM_DIR/nvm.sh"
  nvm install "$NODE_VERSION"
  nvm alias default "$NODE_VERSION"
  nvm use default
  NODE_BIN="$(find "$NVM_DIR"/versions/node -mindepth 2 -maxdepth 2 -name bin -type d | sort -V | tail -n 1)"
  for b in node npm npx corepack; do
    [ -e "$NODE_BIN/$b" ] && ln -sf "$NODE_BIN/$b" "/usr/local/bin/$b"
  done
  __sbx_persist "export NVM_DIR=$NVM_DIR"
  __sbx_persist 'unset NPM_CONFIG_PREFIX'
  # Deliberately single-quoted: persisted verbatim, expanded when sourced.
  # shellcheck disable=SC2016
  __sbx_persist '[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"'
  node --version
}

# install_pnpm — pnpm on top of the resolved node (template:
# template-node-dotnet, global npm install).
install_pnpm() {
  if command -v pnpm >/dev/null 2>&1; then return 0; fi
  echo "[install-toolchain] pnpm missing — installing (major ${SBX_PNPM_VERSION:-12})"
  unset NPM_CONFIG_PREFIX
  npm install -g "pnpm@${SBX_PNPM_VERSION:-12}"
  PNPM_BIN="$(command -v pnpm)"
  [ "$(dirname "$PNPM_BIN")" = "/usr/local/bin" ] || ln -sf "$PNPM_BIN" /usr/local/bin/pnpm
  pnpm --version
}

# install_python — uv-managed CPython (template: template-python,
# astral.sh installer + `uv python install`, symlinked onto /usr/local).
install_python() {
  if command -v python3 >/dev/null 2>&1 && python3 -m ensurepip --version >/dev/null 2>&1; then
    return 0
  fi
  echo "[install-toolchain] python3 (with venv/ensurepip) missing — installing uv + CPython ${SBX_PYTHON_VERSION:-3.12}"
  PYTHON_VERSION="${SBX_PYTHON_VERSION:-3.12}"
  __sbx_apt_prepare
  if [ ! -x /usr/local/bin/uv ]; then
    # uv may exist in a non-PATH location (e.g. /usr/local/bin/uv is
    # missing but /usr/bin/uv exists on some images) — reinstall
    # unmanaged to /usr/local/bin.
    curl -LsSf https://astral.sh/uv/install.sh \
      | env UV_UNMANAGED_INSTALL="/usr/local/bin" sh
  fi
  UV_PYTHON_INSTALL_DIR="${UV_PYTHON_INSTALL_DIR:-/home/agent/.local/share/uv/python}"
  sudo -u agent env \
    UV_PYTHON_INSTALL_DIR="$UV_PYTHON_INSTALL_DIR" \
    UV_LINK_MODE=copy \
    uv python install "$PYTHON_VERSION"
  PY_BIN="$(find "$UV_PYTHON_INSTALL_DIR" -mindepth 2 -maxdepth 2 -name bin -type d -path "*cpython-*" | sort -V | tail -n 1)"
  ln -sf "$PY_BIN/python3" /usr/local/bin/python3
  ln -sf "$PY_BIN/python3" /usr/local/bin/python
  __sbx_persist "export UV_PYTHON_INSTALL_DIR=$UV_PYTHON_INSTALL_DIR"
  __sbx_persist 'export UV_LINK_MODE=copy'
  python3 --version
  # Debian bases need python3-venv for stdlib venv/ensurepip; the
  # uv-managed interpreter already bundles it.
  apt-get install -y -qq --no-install-recommends python3-venv >/dev/null 2>&1 || true
}

# install_go — Go toolchain from the official tarball (template:
# template-go, /usr/local/go). Version is read from SBX_GO_VERSION.
install_go() {
  if command -v go >/dev/null 2>&1; then return 0; fi
  echo "[install-toolchain] go missing — installing ${SBX_GO_VERSION:-1.25.5} from go.dev/dl"
  GO_VERSION="${SBX_GO_VERSION:-1.25.5}"
  __sbx_apt_prepare
  curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-$(dpkg --print-architecture).tar.gz" -o /tmp/sbx-go.tgz
  rm -rf /usr/local/go
  tar -C /usr/local -xzf /tmp/sbx-go.tgz
  rm -f /tmp/sbx-go.tgz
  ln -sf /usr/local/go/bin/go /usr/local/bin/go
  ln -sf /usr/local/go/bin/gofmt /usr/local/bin/gofmt
  # Deliberately single-quoted: persisted verbatim, expanded when sourced.
  # shellcheck disable=SC2016
  __sbx_persist 'export PATH=${PATH}:/home/agent/go/bin'
  go version
}

# install_rust — Rust via rustup (template: template-rust, agent-owned
# RUSTUP_HOME/CARGO_HOME, default profile + rust-analyzer).
install_rust() {
  if command -v cargo >/dev/null 2>&1; then return 0; fi
  echo "[install-toolchain] rust missing — installing via rustup (${SBX_RUST_VERSION:-stable})"
  RUST_VERSION="${SBX_RUST_VERSION:-stable}"
  __sbx_apt_prepare
  # Build essentials cargo links against (libssl for openssl-sys crates).
  apt-get install -y -qq --no-install-recommends build-essential pkg-config libssl-dev
  RUSTUP_HOME="${RUSTUP_HOME:-/home/agent/.rustup}"
  CARGO_HOME="${CARGO_HOME:-/home/agent/.cargo}"
  sudo -u agent env \
    RUSTUP_HOME="$RUSTUP_HOME" CARGO_HOME="$CARGO_HOME" \
    bash -c 'curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile default --default-toolchain "$1" --component rust-analyzer' \
    _ "$RUST_VERSION"
  ln -sf "$CARGO_HOME/bin/"* /usr/local/bin/
  __sbx_persist "export RUSTUP_HOME=$RUSTUP_HOME"
  __sbx_persist "export CARGO_HOME=$CARGO_HOME"
  cargo --version
}

# install_gh — GitHub CLI from the official apt repo (template images
# bake it; the repo entry is removed after install so background
# `apt-get update` never contacts cli.github.com).
install_gh() {
  if command -v gh >/dev/null 2>&1; then return 0; fi
  echo "[install-toolchain] gh missing — installing from cli.github.com"
  __sbx_apt_prepare
  curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
    -o /usr/share/keyrings/githubcli-archive-keyring.gpg
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
    > /etc/apt/sources.list.d/github-cli.list
  apt-get update -qq
  apt-get install -y -qq --no-install-recommends gh
  rm -rf /var/lib/apt/lists/* /etc/apt/sources.list.d/github-cli.list \
    /usr/share/keyrings/githubcli-archive-keyring.gpg
  gh --version
}
