# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG GO_VERSION_ARG

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl

# The tarball URL embeds the exact version, so an empty GO_VERSION_ARG must be
# resolved against go.dev before download.
RUN set -eux; \
    version="${GO_VERSION_ARG:-$(curl -fsSL 'https://go.dev/VERSION?m=text' | head -n 1)}"; \
    curl -fsSL "https://go.dev/dl/${version}.linux-$(dpkg --print-architecture).tar.gz" -o /tmp/go.tgz; \
    tar -C /opt -xzf /tmp/go.tgz; \
    ln -sf /opt/go/bin/go /opt/go/bin/gofmt /usr/local/bin/; \
    go version

FROM scratch

COPY --from=build /opt/go /opt/go
COPY --from=build /usr/local/bin/go /usr/local/bin/go
COPY --from=build /usr/local/bin/gofmt /usr/local/bin/gofmt

COPY <<'EOF' /etc/profile.d/sbx-v3-go.sh
export PATH=$PATH:"${HOME}/go/bin"
EOF
