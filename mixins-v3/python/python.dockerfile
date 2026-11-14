# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG PYTHON_VERSION_ARG
ARG UV_VERSION_ARG

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl xz-utils

# An empty arg resolves to nothing, so the unversioned URL / spec installs latest.
RUN curl -LsSf "https://astral.sh/uv/${UV_VERSION_ARG:+"${UV_VERSION_ARG}/"}install.sh" \
    | env UV_UNMANAGED_INSTALL="/usr/local/bin" sh

# The interpreter tree lives under /opt (not the agent home): a mixin's layers
# overlay an unknown base, so its content must not claim /home/agent paths.
RUN env UV_PYTHON_INSTALL_DIR=/opt/uv/python UV_LINK_MODE=copy \
    uv python install ${PYTHON_VERSION_ARG:+"${PYTHON_VERSION_ARG}"}

RUN PY_BIN="$(find /opt/uv/python -mindepth 2 -maxdepth 2 -name bin -type d -path '*cpython-*' | sort -V | tail -n 1)" \
    && ln -sf "$PY_BIN/python3" /usr/local/bin/python3 \
    && ln -sf "$PY_BIN/python3" /usr/local/bin/python

RUN python3 --version && uv --version

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
