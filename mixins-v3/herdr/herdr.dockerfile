# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG HERDR_VERSION_ARG

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && \
    if [ -n "${HERDR_VERSION_ARG}" ]; then \
      arch="$(uname -m)"; \
      case "${arch}" in x86_64|aarch64) ;; *) echo "unsupported architecture: ${arch}" >&2; exit 1 ;; esac; \
      curl -fsSL "https://github.com/herdrdev/herdr/releases/download/v${HERDR_VERSION_ARG#v}/herdr-linux-${arch}" -o /usr/local/bin/herdr && \
      chmod +x /usr/local/bin/herdr; \
    else \
      curl -fsSL https://herdr.dev/install.sh | HERDR_INSTALL_DIR=/usr/local/bin sh; \
    fi && \
    herdr --version

FROM scratch

COPY --from=build /usr/local/bin/herdr /usr/local/bin/herdr
