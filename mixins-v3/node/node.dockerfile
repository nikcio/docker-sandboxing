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
    if [ -z "${NODE_SPEC}" ]; then \
      # nvm's `node` alias resolves to the newest nodejs.org index.tab row,
      # which can be a pre-release (e.g. v27.0.0-alpha.0.1.1): pin the
      # newest stable version instead.
      NODE_SPEC="$(nvm ls-remote --no-colors | grep -Eo 'v[0-9]+\.[0-9]+\.[0-9]+(-[^ ]+)?' | grep -v -- '-' | tail -n1)"; \
    fi; \
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
# whatever the resolved node version is. The links target the nvm bin-dir
# entries (not their readlink -f targets): npm/pnpm are launchers whose
# relative lib paths only resolve inside the nvm tree, so a dereferenced
# copy of them under /usr/local/bin would break outside a login shell
# (where nvm.sh puts the nvm bin dir on PATH first).
RUN NODE_BIN="$(find /opt/nvm/versions/node -mindepth 2 -maxdepth 2 -name bin -type d | sort -V | tail -n 1)"; \
    install -m 0755 -d /out/bin; \
    for b in node npm npx pnpm pnpx corepack; do \
      if [ -e "$NODE_BIN/$b" ]; then ln -sf "$NODE_BIN/$b" "/out/bin/$b"; \
      elif [ -e "/usr/local/bin/$b" ]; then ln -sf "/usr/local/bin/$b" "/out/bin/$b"; fi; \
    done; \
    install -m 0755 -d /out/libatomic-root; \
    cp -a --parents /usr/lib/*/libatomic.so.1* /out/libatomic-root; \
    ls -l /out/bin

FROM scratch

# The nvm tree lands under /opt/nvm (not the agent home): a mixin's layers
# are an overlay on an unknown base, so its content must not claim
# /home/agent paths the base or the agent user owns.
COPY --from=build /opt/nvm /opt/nvm
COPY --from=build /out/bin/ /usr/local/bin/
COPY --from=build /out/libatomic-root/ /

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
