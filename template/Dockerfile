# syntax=docker/dockerfile:1
#
# Docker Sandboxes template: OpenCode agent environment with
#   - .NET SDK (LTS)
#   - Node.js via NVM (LTS) + PNPM
#   - Git, GitHub CLI (gh), and common CLI utilities
#
# Build:   docker build -t opencode-node-dotnet:v1 template/
# Load:    docker image save opencode-node-dotnet:v1 -o dist/opencode-node-dotnet-v1.tar
#          sbx template load dist/opencode-node-dotnet-v1.tar
#
# The kit in ../kit/ points at this image via `sandbox.image`.

ARG BASE_IMAGE=docker/sandbox-templates:opencode-docker
FROM ${BASE_IMAGE}

# Pin toolchains here; override at build time with --build-arg.
ARG DOTNET_CHANNEL=10.0
ARG NVM_VERSION=v0.40.3
ARG NODE_VERSION=24
ARG PNPM_VERSION=10

ENV DOTNET_ROOT=/usr/share/dotnet \
    DOTNET_CLI_TELEMETRY_OPTOUT=1 \
    DOTNET_NOLOGO=1 \
    DOTNET_GENERATE_ASPNET_CERTIFICATE=false \
    NUGET_XMLDOC_MODE=skip \
    NVM_DIR=/home/agent/.nvm

# System packages (as root)
USER root
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        git \
        git-lfs \
        jq \
        unzip \
        zip \
    && rm -rf /var/lib/apt/lists/*

# GitHub CLI (`gh`) — official apt repo for a current version (the Ubuntu
# universe build lags). The repo + keyring are removed after install so the
# sandbox's background `apt-get update` never contacts cli.github.com, which
# is not in any mixin's network allowlist. Rebuild the image for gh updates.
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
        -o /usr/share/keyrings/githubcli-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
        > /etc/apt/sources.list.d/github-cli.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends gh \
    && rm -rf /var/lib/apt/lists/* \
        /etc/apt/sources.list.d/github-cli.list \
        /usr/share/keyrings/githubcli-archive-keyring.gpg

# .NET SDK, system-wide (as root so every shell finds `dotnet`)
RUN curl -fsSL https://dot.net/v1/dotnet-install.sh -o /tmp/dotnet-install.sh \
    && chmod +x /tmp/dotnet-install.sh \
    && /tmp/dotnet-install.sh --channel "${DOTNET_CHANNEL}" --install-dir "${DOTNET_ROOT}" \
    && rm /tmp/dotnet-install.sh \
    && ln -sf "${DOTNET_ROOT}/dotnet" /usr/local/bin/dotnet

ENV PATH="${DOTNET_ROOT}/tools:${PATH}"

# NVM + Node + PNPM (as agent: user-level tools must live under /home/agent)
# The base image sets NPM_CONFIG_PREFIX for its own npm-global dir; nvm
# refuses to run while it is set, so unset it for the nvm-managed npm.
USER agent
RUN unset NPM_CONFIG_PREFIX \
    && git clone --depth 1 --branch "${NVM_VERSION}" https://github.com/nvm-sh/nvm.git "${NVM_DIR}" \
    && . "${NVM_DIR}/nvm.sh" \
    && nvm install "${NODE_VERSION}" \
    && nvm alias default "${NODE_VERSION}" \
    && nvm use default \
    && npm install -g "pnpm@${PNPM_VERSION}"

# Expose the baked Node/PNPM to every shell (including non-interactive ones
# that never source nvm; they shadow the base image's /usr/bin/node), and
# wire NVM into interactive/login shells.
USER root
RUN NODE_BIN="$(ls -d "${NVM_DIR}"/versions/node/*/bin | sort -V | tail -n 1)" \
    && for b in node npm npx pnpm pnpx corepack; do \
        if [ -e "$NODE_BIN/$b" ]; then ln -sf "$NODE_BIN/$b" "/usr/local/bin/$b"; fi; \
    done \
    && for rc in /home/agent/.bashrc /home/agent/.profile /etc/profile.d/nvm.sh; do \
        printf '\nunset NPM_CONFIG_PREFIX\nexport NVM_DIR="%s"\n[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"\n' "${NVM_DIR}" >> "$rc"; \
    done \
    && chown agent:agent /home/agent/.bashrc /home/agent/.profile

# `o` — shortcut for opencode. A PATH shim (not an rc-file alias) so it
# works in every shell context: login shells, `sbx exec`, scripts. The retry
# rides out self-update: opencode replaces its npm-global
# binary in place on update, and while that swap is in flight the
# resolved path transiently vanishes (ENOENT -> "exec: opencode: not found").
USER root
RUN printf '#!/bin/sh\nfor i in 1 2 3 4 5; do\n  cmd="$(command -v opencode 2>/dev/null)" && [ -x "$cmd" ] && break\n  sleep 1\ndone\nexec opencode "$@"\n' > /usr/local/bin/o \
    && chmod 755 /usr/local/bin/o

# Git comfort for bind-mounted workspaces (host UID mapping triggers
# "dubious ownership" otherwise) — written to the agent's global gitconfig.
USER agent
RUN git config --global --add safe.directory "*" \
    && git config --global init.defaultBranch main
