# syntax=docker/dockerfile:1

# Shell workload on the shell base image: the base provides the
# platform floor (agent uid 1000, workspace, BASH_ENV persistent shell,
# tini); this recipe adds git-lfs, Node.js via nvm, PNPM, Playwright with
# the Chromium headless shell, and the launch contract.
#
# (The shell base is unversioned — nothing here is release-bumped; the
# kit's version lives in kit-node.yaml.)
FROM docker/sandbox-templates:shell

ARG NVM_VERSION=v0.40.7
ARG NODE_VERSION=24
ARG PNPM_VERSION=12
ARG PLAYWRIGHT_VERSION=latest

ENV NVM_DIR=/home/agent/.nvm

USER root
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        git-lfs \
        unzip \
        zip \
    && rm -rf /var/lib/apt/lists/* \
    && git lfs install --system

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

USER root
# Install the Playwright package first, then the browser OS deps as
# root (npx playwright install-deps uses apt; doing it here keeps the
# agent-side step sudo-free, which matters under QEMU arm64 builds
# where setuid does not work).
RUN npm install -g "playwright@${PLAYWRIGHT_VERSION}" \
    && npx playwright install-deps chromium-headless-shell

USER agent
RUN PLAYWRIGHT_BROWSERS_PATH=/home/agent/.cache/ms-playwright npx playwright install chromium-headless-shell

USER root
RUN chown -R agent:agent /home/agent/.cache/ms-playwright

USER agent
RUN git config --global --add safe.directory "*" \
    && git config --global init.defaultBranch main

# The launch contract: bash by default. When the launch-opencode mixin
# is composed it writes /usr/local/bin/agent-launch (exec opencode) and
# this entrypoint hands over to it; without the mixin the shell runs.
# The agent-under-bash contract (sbx@1) is satisfied either way: the
# host launches this command under bash, so BASH_ENV is sourced.
USER agent
ENTRYPOINT ["tini", "--", "bash", "-c", "[ -x /usr/local/bin/agent-launch ] && exec /usr/local/bin/agent-launch || exec bash"]
CMD []
