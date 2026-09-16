# syntax=docker/dockerfile:1

ARG BASE_IMAGE=docker/sandbox-templates:opencode-docker
FROM ${BASE_IMAGE}

ARG RUST_VERSION=1.98
ARG NVM_VERSION=v0.40.7
ARG NODE_VERSION=24
ARG PNPM_VERSION=12
ARG PLAYWRIGHT_VERSION=latest

ENV RUSTUP_HOME=/home/agent/.rustup \
    CARGO_HOME=/home/agent/.cargo \
    PATH="/home/agent/.cargo/bin:${PATH}" \
    NVM_DIR=/home/agent/.nvm

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
    && apt-get purge -y gh \
    && rm -rf /var/lib/apt/lists/*

USER agent
RUN curl -fsSL https://sh.rustup.rs \
        | sh -s -- -y --no-modify-path \
            --profile default \
            --default-toolchain "${RUST_VERSION}" \
            --component rust-analyzer \
    && cargo --version \
    && rustc --version

USER agent
RUN unset NPM_CONFIG_PREFIX \
    && git clone --depth 1 --branch "${NVM_VERSION}" https://github.com/nvm-sh/nvm.git "${NVM_DIR}" \
    && . "${NVM_DIR}/nvm.sh" \
    && nvm install "${NODE_VERSION}" \
    && nvm alias default "${NODE_VERSION}" \
    && nvm use default \
    && npm install -g "pnpm@${PNPM_VERSION}"

USER root
RUN NODE_BIN="$(ls -d "${NVM_DIR}"/versions/node/*/bin | sort -V | tail -n 1)" \
    && for b in node npm npx pnpm pnpx corepack; do \
        if [ -e "$NODE_BIN/$b" ]; then ln -sf "$NODE_BIN/$b" "/usr/local/bin/$b"; fi; \
    done \
    && for rc in /home/agent/.bashrc /home/agent/.profile /etc/profile.d/nvm.sh; do \
        printf '\nunset NPM_CONFIG_PREFIX\nexport NVM_DIR="%s"\n[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"\n' "${NVM_DIR}" >> "$rc"; \
    done \
    && chown agent:agent /home/agent/.bashrc /home/agent/.profile

USER agent
RUN npm install -g "playwright@${PLAYWRIGHT_VERSION}" \
    && PLAYWRIGHT_BROWSERS_PATH=/home/agent/.cache/ms-playwright npx playwright install --with-deps chromium-headless-shell

USER root
RUN chown -R agent:agent /home/agent/.cache/ms-playwright

USER agent
RUN git config --global --add safe.directory "*" \
    && git config --global init.defaultBranch main
