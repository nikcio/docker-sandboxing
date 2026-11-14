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
    # Stage the whole zsh file tree, not just the binary: zsh loads its
    # modules (.so) from /usr/lib/<triplet>/zsh/<version>/zsh, reads the
    # package function files (add-zsh-hook, is-at-least, ...) from
    # /usr/share/zsh and /etc/zsh at startup. A binary-only layer starts
    # but dies with "failed to load module zsh/parameter" on first use.
    # --parents preserves the absolute layout so the COPY below can drop
    # it straight onto /, with any triplet (works on arm64 too).
    install -d /opt/zsh-root; \
    cp -a --parents /usr/bin/zsh* /usr/lib/*/zsh /usr/share/zsh /etc/zsh /opt/zsh-root; \
    zsh --version

FROM scratch

COPY --from=build /opt/zsh-root/ /
COPY --from=build /opt/oh-my-zsh /opt/oh-my-zsh
