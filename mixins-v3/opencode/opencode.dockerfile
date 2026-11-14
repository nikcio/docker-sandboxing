# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG OPENCODE_VERSION_ARG

RUN set -e; \
    if command -v opencode >/dev/null 2>&1; then exit 0; fi; \
    install -m 0755 -d /var/lib/apt/sbx-install-lists; \
    export DEBIAN_FRONTEND=noninteractive; \
    { apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists update -qq || true; }; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists install -y -qq --no-install-recommends ca-certificates curl; \
    rm -rf /var/lib/apt/sbx-install-lists; \
    # The OpenCode home lives under /opt (not the agent home): a mixin's
    # layers are an overlay on an unknown base, so its content must not
    # claim /home/agent paths the base or the agent user owns. The startup
    # hook and profile script hand it back to the agent at boot.
    install -d -m 0755 /opt/opencode-home/.opencode /opt/opencode-home/.cache; \
    OC_VERSION="${OPENCODE_VERSION_ARG}"; \
    OC_INSTALL_ARGS=""; \
    if [ -n "${OC_VERSION}" ]; then OC_INSTALL_ARGS="--version ${OC_VERSION}"; fi; \
    curl -fsSL https://opencode.ai/v2/install | env HOME=/opt/opencode-home bash -s -- ${OC_INSTALL_ARGS}; \
    ln -sf /opt/opencode-home/.opencode/bin/opencode /usr/local/bin/opencode; \
    opencode --version

FROM scratch

COPY --from=build /opt/opencode-home /opt/opencode-home
COPY --from=build /usr/local/bin/opencode /usr/local/bin/opencode

# Point OpenCode's data/config dirs at the baked tree for every process
# (image environment, not just login shells), and keep the locale settings
# the v2 mixin exported: C.UTF-8 is always present on glibc (no
# locale-gen needed); without it man-db's profile script prints "manpath:
# can't set the locale" on every shell start (the template images ship no
# locale env).
ENV XDG_DATA_HOME=/opt/opencode-home/.local/share \
    XDG_CACHE_HOME=/opt/opencode-home/.cache \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8
