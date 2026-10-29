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
    ln -sf /opt/cargo/bin/* /usr/local/bin/; \
    env RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo cargo --version

FROM scratch

COPY --from=build /opt/rustup /opt/rustup
COPY --from=build /opt/cargo /opt/cargo
COPY --from=build /usr/local/bin/rustc /usr/local/bin/rustc
COPY --from=build /usr/local/bin/rustdoc /usr/local/bin/rustdoc
COPY --from=build /usr/local/bin/cargo /usr/local/bin/cargo
COPY --from=build /usr/local/bin/rustup /usr/local/bin/rustup
COPY --from=build /usr/local/bin/rust-analyzer /usr/local/bin/rust-analyzer
COPY --from=build /usr/local/bin/rust-gdb /usr/local/bin/rust-gdb
COPY --from=build /usr/local/bin/rust-lldb /usr/local/bin/rust-lldb

COPY <<'EOF' /etc/profile.d/sbx-v3-rust.sh
export RUSTUP_HOME=/opt/rustup
export CARGO_HOME=/opt/cargo
EOF
