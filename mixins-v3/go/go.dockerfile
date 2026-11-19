# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG GO_VERSION_ARG

RUN set -e; \
    install -m 0755 -d /var/lib/apt/sbx-install-lists; \
    export DEBIAN_FRONTEND=noninteractive; \
    { apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists update -qq || true; }; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists install -y -qq --no-install-recommends ca-certificates curl xz-utils; \
    GO_VERSION="${GO_VERSION_ARG}"; \
    if [ -z "${GO_VERSION}" ]; then \
      GO_VERSION="$(curl -fsSL 'https://go.dev/VERSION?m=text' | head -n 1)"; \
    fi; \
    curl -fsSL "https://go.dev/dl/${GO_VERSION}.linux-$(dpkg --print-architecture).tar.gz" -o /tmp/sbx-go.tgz; \
    tar -C /opt -xzf /tmp/sbx-go.tgz; \
    rm -f /tmp/sbx-go.tgz; \
    ln -sf /opt/go/bin/go /usr/local/bin/go; \
    ln -sf /opt/go/bin/gofmt /usr/local/bin/gofmt; \
    go version

# Stage the PATH links as symlinks (cp -P keeps them): the go binary finds
# GOROOT through its executable path, so a dereferenced copy under
# /usr/local/bin runs but cannot locate its toolchain ("go: cannot find
# GOROOT directory") — it must stay a symlink to /opt/go/bin/go. A plain
# COPY of the link itself would dereference it, hence the directory COPY.
RUN install -m 0755 -d /out/go-bin; \
    cp -P /usr/local/bin/go /usr/local/bin/gofmt /out/go-bin/

FROM scratch

COPY --from=build /opt/go /opt/go
COPY --from=build /out/go-bin/ /usr/local/bin/

# `go install` results on PATH. GOBIN is left at Go's default, $HOME/go/bin
# (the v2 mixin added the same entry); nothing is claimed in the agent home,
# Go writes there at the user's request.
COPY <<'EOF' /etc/profile.d/sbx-v3-go.sh
export PATH=$PATH:"${HOME}/go/bin"
EOF
