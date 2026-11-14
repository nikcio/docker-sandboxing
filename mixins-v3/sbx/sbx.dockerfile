# syntax=docker/dockerfile:1
# Ubuntu base: the Docker apt repo (download.docker.com/linux/ubuntu)
# publishes no Debian suites, and the sandbox bases this mixin targets
# are Ubuntu.
FROM ubuntu:24.04 AS build

ARG SBX_VERSION_ARG

RUN set -e; install -m 0755 -d /var/lib/apt/sbx-lists; export DEBIAN_FRONTEND=noninteractive; { apt-get -o Dir::State::lists=/var/lib/apt/sbx-lists update -qq || true; }; apt-get -o Dir::State::lists=/var/lib/apt/sbx-lists install -y -qq --no-install-recommends ca-certificates curl; \
    if command -v sbx >/dev/null 2>&1; then exit 0; fi; \
    install -m 0755 -d /etc/apt/keyrings; \
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc; \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" > /etc/apt/sources.list.d/docker.list; \
    install -m 0755 -d /var/lib/apt/sbx-lists; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-lists update -qq; \
    SBX_PKG="docker-sbx"; \
    SBX_SPEC="${SBX_VERSION_ARG}"; \
    if [ -n "${SBX_SPEC}" ]; then SBX_PKG="docker-sbx=${SBX_SPEC}"; fi; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-lists install -y --no-install-recommends ${SBX_PKG}; \
    rm -rf /var/lib/apt/sbx-lists

FROM scratch

COPY --from=build /usr/bin/sbx /usr/local/bin/sbx
COPY --from=build /etc/apt/sources.list.d/docker.list /etc/apt/sources.list.d/docker.list
COPY --from=build /etc/apt/keyrings/docker.asc /etc/apt/keyrings/docker.asc
