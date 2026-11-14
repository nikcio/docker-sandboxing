# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG OPENCODE_VERSION_ARG

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl

# The OpenCode install tree lives under /opt (not the agent home): a mixin's
# layers are an overlay on an unknown base, so its content must not claim
# /home/agent paths the base or the agent user owns.
RUN mkdir -p /opt/opencode-home

# Empty arg = latest; set = install that exact version.
RUN if [ -n "$OPENCODE_VERSION_ARG" ]; then \
      curl -fsSL https://opencode.ai/v2/install | env HOME=/opt/opencode-home bash -s -- --version "$OPENCODE_VERSION_ARG"; \
    else \
      curl -fsSL https://opencode.ai/v2/install | env HOME=/opt/opencode-home bash -s --; \
    fi

RUN ln -sf /opt/opencode-home/.opencode/bin/opencode /usr/local/bin/opencode \
 && opencode --version

FROM scratch

COPY --from=build /opt/opencode-home /opt/opencode-home
COPY --from=build /usr/local/bin/opencode /usr/local/bin/opencode

# Keep the locale settings the v2 mixin exported: C.UTF-8 is always
# present on glibc (no locale-gen needed); without it man-db's profile
# script prints "manpath: can't set the locale" on every shell start (the
# template images ship no locale env). XDG_DATA_HOME/XDG_CACHE_HOME are
# deliberately NOT pointed at the baked tree: it is root-owned (the
# build stage has no agent user to chown to), so opencode writing its
# log/state/cache there fails with EACCES — the XDG defaults under the
# agent home are the v2 behavior.
ENV LANG=C.UTF-8 \
    LC_ALL=C.UTF-8
