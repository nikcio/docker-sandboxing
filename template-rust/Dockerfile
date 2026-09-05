# syntax=docker/dockerfile:1
#
# Docker Sandboxes template: OpenCode agent environment with
#   - Rust via rustup (stable toolchain, clippy + rustfmt)
#   - Git, GitHub CLI (gh), and common CLI utilities
#
# Build:   docker build -t opencode-rust:v1 template-rust/
# Load:    docker image save opencode-rust:v1 -o dist/opencode-rust-v1.tar
#          sbx template load dist/opencode-rust-v1.tar
#
# The kit in ../kit-rust/ points at this image via `sandbox.image`.

ARG BASE_IMAGE=docker/sandbox-templates:opencode-docker
FROM ${BASE_IMAGE}

# Pin toolchains here; override at build time with --build-arg.
# RUST_VERSION is a partial version — rustup resolves it to the newest
# matching release at build time (1.98 -> 1.98.x). Rebuild the image for
# updates, or pin an exact version with --build-arg RUST_VERSION=1.98.1.
ARG RUST_VERSION=1.98

# rustup + cargo homes under the agent home (user-level tools must live
# under /home/agent; the agent can add toolchains later with
# `rustup toolchain install`). PATH bakes the cargo bin dir into every
# shell context — login shells, `sbx exec`, scripts — with no rc-file
# edits (rustup-init runs with --no-modify-path below).
ENV RUSTUP_HOME=/home/agent/.rustup \
    CARGO_HOME=/home/agent/.cargo \
    PATH="/home/agent/.cargo/bin:${PATH}"

# System packages (as root). build-essential + pkg-config + libssl-dev:
# the C toolchain (linker) and libraries most crates link against
# (e.g. -sys crates like openssl-sys).
USER root
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        git \
        git-lfs \
        jq \
        pkg-config \
        libssl-dev \
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

# rustup — as agent (user-level tools must live under /home/agent).
# --no-modify-path keeps rc files clean: the cargo bin dir is already on
# PATH via ENV. The default profile installs rustc, cargo, rust-std,
# rust-docs, rustfmt, and clippy; rust-analyzer is added on top so the
# agent's LSP works out of the box. "${RUST_VERSION}" resolves to the
# newest matching release at build time.
USER agent
RUN curl -fsSL https://sh.rustup.rs \
        | sh -s -- -y --no-modify-path \
            --profile default \
            --default-toolchain "${RUST_VERSION}" \
            --component rust-analyzer \
    && cargo --version \
    && rustc --version

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
