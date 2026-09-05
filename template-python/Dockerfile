# syntax=docker/dockerfile:1
#
# Docker Sandboxes template: OpenCode agent environment with
#   - Python (uv-managed CPython) + uv
#   - Git, GitHub CLI (gh), and common CLI utilities
#
# Build:   docker build -t opencode-python:v1 template-python/
# Load:    docker image save opencode-python:v1 -o dist/opencode-python-v1.tar
#          sbx template load dist/opencode-python-v1.tar
#
# The kit in ../kit-python/ points at this image via `sandbox.image`.

ARG BASE_IMAGE=docker/sandbox-templates:opencode-docker
FROM ${BASE_IMAGE}

# Pin toolchains here; override at build time with --build-arg.
# uv itself is intentionally unpinned (like gh): the installer bakes the
# current release at build time — rebuild the image for uv updates.
ARG PYTHON_VERSION=3.14

ENV UV_LINK_MODE=copy \
    UV_PYTHON_INSTALL_DIR=/home/agent/.local/share/uv/python

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

# uv — system-wide (as root so every shell and user finds `uv`/`uvx`).
# UV_UNMANAGED_INSTALL keeps the installer from editing shell rc files and
# disables `uv self update`: uv ships from astral.sh/github.com, which is not
# in any mixin's network allowlist. Updates come from image rebuilds.
RUN curl -LsSf https://astral.sh/uv/install.sh \
        | env UV_UNMANAGED_INSTALL="/usr/local/bin" sh \
    && uv --version

# Python — uv-managed CPython, installed as agent (user-level tools must
# live under /home/agent; the agent can add more interpreters later with
# `uv python install`). "${PYTHON_VERSION}" resolves to the newest matching
# CPython build at build time.
USER agent
RUN uv python install "${PYTHON_VERSION}" \
    && uv python find "${PYTHON_VERSION}"

# Expose the managed Python to every shell (including non-interactive ones;
# it shadows the base image's system python) — same pattern as the
# node-dotnet template's node/pnpm symlinks. Points at the interpreter baked
# at build time; projects needing a different version use uv's own
# resolution (.python-version + `uv python install`, requires egress).
USER root
RUN PY_BIN="$(ls -d "${UV_PYTHON_INSTALL_DIR}"/cpython-*/bin | sort -V | tail -n 1)" \
    && ln -sf "${PY_BIN}/python3" /usr/local/bin/python3 \
    && ln -sf "${PY_BIN}/python3" /usr/local/bin/python

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
