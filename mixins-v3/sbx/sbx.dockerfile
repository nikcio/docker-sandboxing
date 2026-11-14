# syntax=docker/dockerfile:1
# Ubuntu base: the Docker apt repo (download.docker.com/linux/ubuntu)
# publishes no Debian suites, and the sandbox bases this mixin targets
# are Ubuntu.
FROM ubuntu:24.04 AS build

ARG SBX_VERSION_ARG

RUN set -e; export DEBIAN_FRONTEND=noninteractive; \
    apt-get update && apt-get install -y --no-install-recommends ca-certificates curl; \
    mkdir -p /etc/apt/keyrings; \
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc; \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" > /etc/apt/sources.list.d/docker.list; \
    apt-get update; \
    if [ -n "${SBX_VERSION_ARG}" ]; then \
      apt-get install -y --no-install-recommends "docker-sbx=${SBX_VERSION_ARG}"; \
    else \
      apt-get install -y --no-install-recommends docker-sbx; \
    fi; \
    sbx --version

FROM scratch

COPY --from=build /usr/bin/sbx /usr/local/bin/sbx
COPY --from=build /etc/apt/sources.list.d/docker.list /etc/apt/sources.list.d/docker.list
COPY --from=build /etc/apt/keyrings/docker.asc /etc/apt/keyrings/docker.asc
