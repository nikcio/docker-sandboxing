# syntax=docker/dockerfile:1
FROM debian:bookworm-slim AS build

ARG RUST_VERSION_ARG
ARG RUSTUP_PROFILE

RUN set -e; \
    install -m 0755 -d /var/lib/apt/sbx-install-lists; \
    export DEBIAN_FRONTEND=noninteractive; \
    { apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists update -qq || true; }; \
    apt-get -o Dir::State::lists=/var/lib/apt/sbx-install-lists install -y -qq --no-install-recommends ca-certificates curl xz-utils build-essential pkg-config libssl-dev; \
    RUST_TOOLCHAIN="${RUST_VERSION_ARG}"; \
    if [ -z "${RUST_TOOLCHAIN}" ]; then RUST_TOOLCHAIN=stable; fi; \
    # The rustup home lives under /opt (not the agent home): a mixin's
    # layers are an overlay on an unknown base, so its content must not
    # claim /home/agent paths the base or the agent user owns.
    env RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo bash -c 'curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path --profile '"${RUSTUP_PROFILE}"' --default-toolchain '"${RUST_TOOLCHAIN}"' --component rust-analyzer'; \
    ls /opt/cargo/bin/cargo >/dev/null; \
    # Stage a link to every proxy rustup installed (rustfmt/clippy ships
    # with the default profile) so the final layer exposes the same set
    # the v2 mixin did, whatever the profile resolves to.
    install -m 0755 -d /out/rust-bin; \
    for f in /opt/cargo/bin/*; do ln -s "$f" "/out/rust-bin/$(basename "$f")"; done; \
    env RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo PATH="/opt/cargo/bin:$PATH" cargo --version

FROM scratch

COPY --from=build /opt/rustup /opt/rustup
COPY --from=build /opt/cargo /opt/cargo
COPY --from=build /out/rust-bin/ /usr/local/bin/

COPY <<'EOF' /etc/profile.d/sbx-v3-rust.sh
export RUSTUP_HOME=/opt/rustup
export CARGO_HOME=/opt/cargo
EOF
