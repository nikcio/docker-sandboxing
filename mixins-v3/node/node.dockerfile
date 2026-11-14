# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG NODE_VERSION_ARG
ARG PNPM_VERSION_ARG
ARG NVM_VERSION_ARG

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl xz-utils libatomic1 git \
    && export NVM_DIR=/opt/nvm \
    && NVM_REF="${NVM_VERSION_ARG:-$(curl -fsSLI -o /dev/null -w '%{url_effective}' https://github.com/nvm-sh/nvm/releases/latest)}" \
    && NVM_REF="${NVM_REF##*/}" \
    && git clone -q --depth 1 --branch "$NVM_REF" https://github.com/nvm-sh/nvm.git "$NVM_DIR" \
    && . "$NVM_DIR/nvm.sh" \
    && nvm install "${NODE_VERSION_ARG:-node}" \
    && nvm alias default "$(nvm version "${NODE_VERSION_ARG:-node}")" \
    && npm install -g "pnpm@${PNPM_VERSION_ARG:-latest}" \
    && NODE_BIN="$(dirname "$(nvm which default)")" \
    && mkdir -p /out/bin \
    && for b in node npm npx pnpm pnpx corepack; do if [ -e "$NODE_BIN/$b" ]; then ln -sf "$NODE_BIN/$b" "/out/bin/$b"; fi; done \
    && node --version \
    && pnpm --version

FROM scratch

# The nvm tree lands under /opt/nvm (not the agent home): a mixin's layers
# are an overlay on an unknown base, so its content must not claim
# /home/agent paths the base or the agent user owns. The /usr/local/bin
# symlinks point here, so the copy must land at the identical path.
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
