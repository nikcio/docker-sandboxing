# syntax=docker/dockerfile:1

# OpenCode workload on the shell base image: the base provides the
# platform floor (agent uid 1000, workspace, BASH_ENV persistent shell,
# tini); this recipe adds the official Go toolchain (GOTOOLCHAIN=auto
# pulls newer toolchains via the module proxy), git-lfs, Node.js via nvm,
# PNPM, and Playwright with the Chromium headless shell, plus the launch
# contract.
#
# The agent itself is installed by the opencode mixin's install hook
# (opencode-ai@latest) — compose it, or bake a pinned agent by replacing
# the hook's npm install with a RUN here.
#
# x-release-please-start-version
# (the shell base is unversioned; the block anchors the release bump)
# x-release-please-end-version
FROM docker/sandbox-templates:shell

ARG GO_VERSION=1.27.1
ARG NVM_VERSION=v0.40.7
ARG NODE_VERSION=24
ARG PNPM_VERSION=12
ARG PLAYWRIGHT_VERSION=latest

ENV PATH="/usr/local/go/bin:/home/agent/go/bin:${PATH}" \
    NVM_DIR=/home/agent/.nvm

USER root
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        git-lfs \
        unzip \
        zip \
    && rm -rf /var/lib/apt/lists/* \
    && git lfs install --system

RUN curl -fsSL "https://go.dev/dl/go${GO_VERSION}.linux-$(dpkg --print-architecture).tar.gz" \
        -o /tmp/go.tgz \
    && rm -rf /usr/local/go \
    && tar -C /usr/local -xzf /tmp/go.tgz \
    && rm /tmp/go.tgz \
    && go version

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

# The launch contract: the shell base launches bash under tini; this
# workload launches opencode under the same supervisor.
USER agent
ENTRYPOINT ["tini", "--", "opencode"]
CMD []
