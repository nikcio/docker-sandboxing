# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG HERDR_VERSION_ARG

RUN set -e; \
    if command -v herdr >/dev/null 2>&1; then exit 0; fi; \
    install -m 0755 -d /var/lib/apt/sbx-install-lists; \
    export DEBIAN_FRONTEND=noninteractive; \
    { apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists update -qq || true; }; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists install -y -qq --no-install-recommends ca-certificates curl; \
    HERDR_VERSION="${HERDR_VERSION_ARG}"; \
    case "${HERDR_VERSION}" in v*) HERDR_VERSION="${HERDR_VERSION#v}";; esac; \
    if [ -n "${HERDR_VERSION}" ]; then \
      case "$(uname -m)" in \
        x86_64|amd64) HERDR_ARCH=x86_64 ;; \
        aarch64|arm64) HERDR_ARCH=aarch64 ;; \
        *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;; \
      esac; \
      curl -fsSL "https://github.com/herdrdev/herdr/releases/download/v${HERDR_VERSION}/herdr-linux-${HERDR_ARCH}" -o /usr/local/bin/herdr; \
      chmod +x /usr/local/bin/herdr; \
    else \
      curl -fsSL https://herdr.dev/install.sh | HERDR_INSTALL_DIR=/usr/local/bin sh; \
    fi; \
    herdr --version

FROM scratch

COPY --from=build /usr/local/bin/herdr /usr/local/bin/herdr
