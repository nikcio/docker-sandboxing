# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG GH_VERSION_ARG

RUN set -e; install -m 0755 -d /var/lib/apt/sbx-lists; export DEBIAN_FRONTEND=noninteractive; { apt-get -o Dir::State::lists=/var/lib/apt/sbx-lists update -qq || true; }; apt-get -o Dir::State::lists=/var/lib/apt/sbx-lists install -y -qq --no-install-recommends ca-certificates curl; rm -rf /var/lib/apt/sbx-lists; \
    if command -v gh >/dev/null 2>&1; then exit 0; fi; \
    install -m 0755 -d /etc/apt/keyrings; \
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg -o /etc/apt/keyrings/githubcli-archive-keyring.gpg; \
    chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg; \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" > /etc/apt/sources.list.d/github-cli.list; \
    install -m 0755 -d /var/lib/apt/sbx-lists; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-lists update -qq; \
    GH_PKG="gh"; \
    GH_SPEC="${GH_VERSION_ARG}"; \
    if [ -n "${GH_SPEC}" ]; then GH_PKG="gh=${GH_SPEC}"; fi; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-lists install -y --no-install-recommends ${GH_PKG}; \
    rm -rf /var/lib/apt/sbx-lists

# The gh binary plus the apt repo definition (so `apt upgrade` inside the
# sandbox keeps working); the keyring rides along.
FROM scratch

COPY --from=build /usr/bin/gh /usr/local/bin/gh
COPY --from=build /etc/apt/sources.list.d/github-cli.list /etc/apt/sources.list.d/github-cli.list
COPY --from=build /etc/apt/keyrings/githubcli-archive-keyring.gpg /etc/apt/keyrings/githubcli-archive-keyring.gpg
