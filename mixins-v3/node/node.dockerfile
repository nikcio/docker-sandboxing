# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG NODE_VERSION_ARG
ARG PNPM_VERSION_ARG
ARG NVM_VERSION_ARG

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
    for b in node npm npx pnpm pnpx corepack; do if [ -e "$NODE_BIN/$b" ]; then ln -sf "$NODE_BIN/$b" "/usr/local/bin/$b"; fi; done; \
    unset NPM_CONFIG_PREFIX; \
    PNPM_SPEC="${PNPM_VERSION_ARG}"; \
    if [ -n "${PNPM_SPEC}" ]; then PNPM_SPEC="@${PNPM_SPEC}"; fi; \
    npm install -g "pnpm${PNPM_SPEC}"; \
    PNPM_BIN="$(command -v pnpm)"; \
    if [ "$(dirname "$PNPM_BIN")" != "/usr/local/bin" ]; then ln -sf "$PNPM_BIN" /usr/local/bin/pnpm; fi; \
    node --version; \
    pnpm --version

# Stage the bins into a stable path so the scratch stage can copy them
# whatever the resolved node version is.
RUN NODE_BIN="$(find /opt/nvm/versions/node -mindepth 2 -maxdepth 2 -name bin -type d | sort -V | tail -n 1)"; \
    mkdir -p /out/bin; \
    for b in node npm npx pnpm pnpx corepack; do \
      if [ -e "$NODE_BIN/$b" ] || [ -e "/usr/local/bin/$b" ]; then \
        cp -P "$(readlink -f "$NODE_BIN/$b" 2>/dev/null || echo /usr/local/bin/$b)" "/out/bin/$b"; \
      fi; \
    done; \
    ls /out/bin

FROM scratch

# The nvm tree lands under /opt/nvm (not the agent home): a mixin's layers
# are an overlay on an unknown base, so its content must not claim
# /home/agent paths the base or the agent user owns.
COPY --from=build /opt/nvm /opt/nvm
COPY --from=build /out/bin/ /usr/local/bin/

# Login shells (the agent launches through one) pick up NVM_DIR; nvm.sh
# finds the installed default under it.
COPY <<'EOF' /etc/profile.d/sbx-v3-node.sh
export NVM_DIR=/opt/nvm
unset NPM_CONFIG_PREFIX
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
EOF

# pnpm refuses versions published less than 24h ago, guarding against
# freshly published compromised packages.
ENV PNPM_CONFIG_MINIMUM_RELEASE_AGE=1440
