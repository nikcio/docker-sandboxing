# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG PYTHON_VERSION_ARG
ARG UV_VERSION_ARG

RUN set -e; \
    install -m 0755 -d /var/lib/apt/sbx-install-lists; \
    export DEBIAN_FRONTEND=noninteractive; \
    { apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists update -qq || true; }; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists install -y -qq --no-install-recommends ca-certificates curl xz-utils; \
    UV_SPEC="${UV_VERSION_ARG}"; \
    if [ -n "${UV_SPEC}" ]; then \
      curl -LsSf "https://astral.sh/uv/${UV_SPEC}/install.sh" | env UV_UNMANAGED_INSTALL="/usr/local/bin" sh; \
    else \
      curl -LsSf https://astral.sh/uv/install.sh | env UV_UNMANAGED_INSTALL="/usr/local/bin" sh; \
    fi; \
    PY_SPEC="${PYTHON_VERSION_ARG}"; \
    # The interpreter tree lives under /opt (not the agent home): a mixin's
    # layers are an overlay on an unknown base, so its content must not
    # claim /home/agent paths the base or the agent user owns.
    env UV_PYTHON_INSTALL_DIR=/opt/uv/python UV_LINK_MODE=copy uv python install ${PY_SPEC:+"${PY_SPEC}"}; \
    PY_BIN="$(find /opt/uv/python -mindepth 2 -maxdepth 2 -name bin -type d -path '*cpython-*' | sort -V | tail -n 1)"; \
    ln -sf "$PY_BIN/python3" /usr/local/bin/python3; \
    ln -sf "$PY_BIN/python3" /usr/local/bin/python; \
    python3 --version; \
    uv --version

FROM scratch

COPY --from=build /usr/local/bin/uv /usr/local/bin/uv
COPY --from=build /usr/local/bin/uvx /usr/local/bin/uvx
COPY --from=build /opt/uv /opt/uv
COPY --from=build /usr/local/bin/python3 /usr/local/bin/python3
COPY --from=build /usr/local/bin/python /usr/local/bin/python

COPY <<'EOF' /etc/profile.d/sbx-v3-python.sh
export UV_PYTHON_INSTALL_DIR=/opt/uv/python
export UV_LINK_MODE=copy
EOF
