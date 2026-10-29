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

FROM scratch

COPY --from=build /opt/go /opt/go
COPY --from=build /usr/local/bin/go /usr/local/bin/go
COPY --from=build /usr/local/bin/gofmt /usr/local/bin/gofmt

# User-level bin dir on PATH for `go install` results. Login shells pick
# it up; /opt/go/bin comes via the /usr/local/bin symlinks above.
COPY <<'EOF' /etc/profile.d/sbx-v3-go.sh
export PATH=$PATH:/opt/go-user-bin
EOF
