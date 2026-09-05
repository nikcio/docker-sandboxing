# syntax=docker/dockerfile:1
#
# Docker Sandboxes template: OpenCode agent environment with
#   - Go toolchain (official tarball) + module tooling
#   - Git, GitHub CLI (gh), and common CLI utilities
#
# Build:   docker build -t opencode-go:v1 template-go/
# Load:    docker image save opencode-go:v1 -o dist/opencode-go-v1.tar
#          sbx template load dist/opencode-go-v1.tar
#
# The kit in ../kit-go/ points at this image via `sandbox.image`.

ARG BASE_IMAGE=docker/sandbox-templates:opencode-docker
FROM ${BASE_IMAGE}

# Pin toolchains here; override at build time with --build-arg.
# GOTOOLCHAIN keeps its default (`auto`): projects asking for a newer Go
# download that toolchain as a module through the proxy (go mixin egress) —
# rebuild the image for a newer baked toolchain.
ARG GO_VERSION=1.27.1

# `go` (baked toolchain) and `go install`-ed tools (~/go/bin) on every PATH.
ENV PATH="/usr/local/go/bin:/home/agent/go/bin:${PATH}"

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

# Go — official tarball from go.dev/dl, system-wide (as root so every shell
# and user finds `go`). GOPATH stays at its default ~/go (agent-writable);
# toolchain updates come from image rebuilds, or in-sandbox via
# GOTOOLCHAIN=auto (toolchain modules through the go mixin's proxy egress).
RUN curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-$(dpkg --print-architecture).tar.gz" \
        -o /tmp/go.tgz \
    && rm -rf /usr/local/go \
    && tar -C /usr/local -xzf /tmp/go.tgz \
    && rm /tmp/go.tgz \
    && go version

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
