# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG GH_VERSION_ARG

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl

RUN mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg -o /etc/apt/keyrings/githubcli-archive-keyring.gpg && \
    chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" > /etc/apt/sources.list.d/github-cli.list && \
    apt-get update && \
    if [ -n "${GH_VERSION_ARG}" ]; then apt-get install -y --no-install-recommends "gh=${GH_VERSION_ARG}"; else apt-get install -y --no-install-recommends gh; fi && \
    gh --version

# The gh binary plus the apt repo definition (so `apt upgrade` inside the
# sandbox keeps working); the keyring rides along.
FROM scratch

COPY --from=build /usr/bin/gh /usr/local/bin/gh
COPY --from=build /etc/apt/sources.list.d/github-cli.list /etc/apt/sources.list.d/github-cli.list
COPY --from=build /etc/apt/keyrings/githubcli-archive-keyring.gpg /etc/apt/keyrings/githubcli-archive-keyring.gpg
