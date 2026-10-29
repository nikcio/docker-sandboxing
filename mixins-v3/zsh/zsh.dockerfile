# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG ZSH_VERSION_ARG
ARG OH_MY_ZSH_REF_ARG

RUN set -e; \
    export DEBIAN_FRONTEND=noninteractive; \
    install -m 0755 -d /var/lib/apt/sbx-lists; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-lists update -qq || true; \
    ZSH_PKG="zsh"; \
    ZSH_SPEC="${ZSH_VERSION_ARG}"; \
    if [ -n "${ZSH_SPEC}" ]; then ZSH_PKG="zsh=${ZSH_SPEC}"; fi; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-lists install -y -qq --no-install-recommends ${ZSH_PKG} git curl ca-certificates; \
    rm -rf /var/lib/apt/sbx-lists; \
    # The Oh My Zsh tree lives under /opt: a mixin's layers are an overlay
    # on an unknown base, so its content must not claim /home/agent paths
    # the base or the agent user owns.
    OMZ_REF="${OH_MY_ZSH_REF_ARG}"; \
    git clone -q --depth 1 ${OMZ_REF:+--branch "${OMZ_REF}"} https://github.com/ohmyzsh/ohmyzsh.git /opt/oh-my-zsh; \
    zsh --version

FROM scratch

COPY --from=build /usr/bin/zsh /usr/bin/zsh
COPY --from=build /opt/oh-my-zsh /opt/oh-my-zsh
