# syntax=docker/dockerfile:1

ARG BASE_IMAGE=docker/sandbox-templates:opencode-docker
FROM ${BASE_IMAGE}

ARG PYTHON_VERSION=3.14
ARG NVM_VERSION=v0.40.7
ARG NODE_VERSION=24
ARG PNPM_VERSION=12
ARG PLAYWRIGHT_VERSION=latest

ENV UV_LINK_MODE=copy \
    UV_PYTHON_INSTALL_DIR=/home/agent/.local/share/uv/python \
    NVM_DIR=/home/agent/.nvm

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
    && apt-get purge -y gh \
    && rm -rf /var/lib/apt/lists/*

USER root
RUN curl -LsSf https://astral.sh/uv/install.sh \
        | env UV_UNMANAGED_INSTALL="/usr/local/bin" sh \
    && uv --version

USER agent
RUN uv python install "${PYTHON_VERSION}" \
    && uv python find "${PYTHON_VERSION}"

USER root
RUN PY_BIN="$(ls -d "${UV_PYTHON_INSTALL_DIR}"/cpython-*/bin | sort -V | tail -n 1)" \
    && ln -sf "${PY_BIN}/python3" /usr/local/bin/python3 \
    && ln -sf "${PY_BIN}/python3" /usr/local/bin/python

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
