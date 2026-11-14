# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG RUST_VERSION_ARG
ARG RUSTUP_PROFILE

# The rustup home lives under /opt (not the agent home): a mixin's layers
# are an overlay on an unknown base, so its content must not claim
# /home/agent paths the base or the agent user owns.
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates curl xz-utils build-essential pkg-config libssl-dev \
    && curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile "${RUSTUP_PROFILE:-default}" --default-toolchain "${RUST_VERSION_ARG:-stable}" --component rust-analyzer \
    && /opt/cargo/bin/cargo --version

# Stage a link to every proxy rustup installed (rustfmt/clippy ship with
# the default profile) so the final layer exposes the same set whatever
# the profile resolves to.
RUN mkdir -p /out/rust-bin \
    && for f in /opt/cargo/bin/*; do ln -s "$f" /out/rust-bin/; done

FROM scratch

COPY --from=build /opt/rustup /opt/rustup
COPY --from=build /opt/cargo /opt/cargo
COPY --from=build /out/rust-bin/ /usr/local/bin/

COPY <<'EOF' /etc/profile.d/sbx-v3-rust.sh
export RUSTUP_HOME=/opt/rustup
export CARGO_HOME=/opt/cargo
EOF
