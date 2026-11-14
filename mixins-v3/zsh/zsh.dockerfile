# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG DEBIAN_FRONTEND=noninteractive
ARG ZSH_VERSION_ARG
ARG OH_MY_ZSH_REF_ARG

RUN apt-get update \
 && if [ -n "${ZSH_VERSION_ARG}" ]; then \
      apt-get install -y --no-install-recommends zsh="${ZSH_VERSION_ARG}" git curl ca-certificates; \
    else \
      apt-get install -y --no-install-recommends zsh git curl ca-certificates; \
    fi

RUN if [ -n "${OH_MY_ZSH_REF_ARG}" ]; then \
      git clone --depth 1 --branch "${OH_MY_ZSH_REF_ARG}" https://github.com/ohmyzsh/ohmyzsh.git /opt/oh-my-zsh; \
    else \
      git clone --depth 1 https://github.com/ohmyzsh/ohmyzsh.git /opt/oh-my-zsh; \
    fi

# Stage the whole zsh file tree, not just the binary: zsh loads its
# modules (.so) from /usr/lib/<triplet>/zsh/<version>/ and its function
# files from /usr/share/zsh and /etc/zsh at startup — a binary-only copy
# dies with "failed to load module zsh/parameter". --parents preserves
# the absolute layout so the COPY below can drop it straight onto /,
# with any triplet (works on arm64 too).
RUN mkdir -p /opt/zsh-root \
 && cp -a --parents /usr/bin/zsh* /usr/lib/*/zsh /usr/share/zsh /etc/zsh /opt/zsh-root \
 && zsh --version

FROM scratch

COPY --from=build /opt/zsh-root/ /
COPY --from=build /opt/oh-my-zsh /opt/oh-my-zsh
