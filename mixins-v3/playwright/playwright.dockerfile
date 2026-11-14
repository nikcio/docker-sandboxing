# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG PLAYWRIGHT_VERSION_ARG
ARG NODE_VERSION_ARG
ARG NVM_VERSION_ARG

# Node via nvm, same layout as the node mixin (tree under /opt/nvm: a
# mixin's layers are an overlay on an unknown base).
RUN set -e; install -m 0755 -d /var/lib/apt/sbx-install-lists; \
    export DEBIAN_FRONTEND=noninteractive; \
    { apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists update -qq || true; }; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists install -y -qq --no-install-recommends ca-certificates curl xz-utils libatomic1 git; \
    export NVM_DIR=/opt/nvm; \
    NVM_REF="${NVM_VERSION_ARG}"; \
    if [ -z "${NVM_REF}" ]; then \
      NVM_REF="$(curl -fsSL https://api.github.com/repos/nvm-sh/nvm/releases/latest | grep -o '"tag_name": *"[^"]*"' | cut -d'"' -f4)"; \
    fi; \
    git clone -q --depth 1 --branch "${NVM_REF}" https://github.com/nvm-sh/nvm.git "$NVM_DIR"; \
    unset NPM_CONFIG_PREFIX; \
    . "$NVM_DIR/nvm.sh"; \
    NODE_SPEC="${NODE_VERSION_ARG}"; \
    if [ -z "${NODE_SPEC}" ]; then NODE_SPEC=node; fi; \
    nvm install "${NODE_SPEC}"; \
    NODE_RESOLVED="$(nvm version "${NODE_SPEC}")"; \
    nvm alias default "${NODE_RESOLVED}"; \
    nvm use default; \
    NODE_BIN="$(find "$NVM_DIR"/versions/node -mindepth 2 -maxdepth 2 -name bin -type d | sort -V | tail -n 1)"; \
    for b in node npm npx corepack; do if [ -e "$NODE_BIN/$b" ]; then ln -sf "$NODE_BIN/$b" "/usr/local/bin/$b"; fi; done; \
    unset NPM_CONFIG_PREFIX; \
    PW_SPEC="${PLAYWRIGHT_VERSION_ARG}"; \
    if [ -n "${PW_SPEC}" ]; then PW_SPEC="@${PW_SPEC}"; fi; \
    npm install -g "playwright${PW_SPEC}"; \
    # The browser tree lives under /opt (not the agent home): a mixin's
    # layers are an overlay on an unknown base.
    PLAYWRIGHT_BROWSERS_PATH=/opt/ms-playwright npx playwright install --with-deps chromium-headless-shell; \
    npx playwright --version

RUN NODE_BIN="$(find /opt/nvm/versions/node -mindepth 2 -maxdepth 2 -name bin -type d | sort -V | tail -n 1)"; \
    mkdir -p /out/bin; \
    for b in node npm npx corepack; do \
      if [ -e "$NODE_BIN/$b" ]; then cp -P "$(readlink -f "$NODE_BIN/$b")" "/out/bin/$b"; fi; \
    done; \
    PW_BIN="$(find /opt/nvm/versions/node -mindepth 3 -maxdepth 3 -path '*bin/playwright' 2>/dev/null | head -n 1)"; \
    if [ -n "$PW_BIN" ]; then cp -P "$(readlink -f "$PW_BIN")" /out/bin/playwright; fi; \
    ls /out/bin

FROM scratch

COPY --from=build /opt/nvm /opt/nvm
COPY --from=build /opt/ms-playwright /opt/ms-playwright
COPY --from=build /out/bin/ /usr/local/bin/

COPY <<'EOF' /etc/profile.d/sbx-v3-playwright.sh
export NVM_DIR=/opt/nvm
unset NPM_CONFIG_PREFIX
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
export PLAYWRIGHT_BROWSERS_PATH=/opt/ms-playwright
EOF
