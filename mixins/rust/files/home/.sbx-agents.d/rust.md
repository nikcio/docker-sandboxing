## Rust / cargo

- Rust is rustup-managed (`cargo`/`rustc`/`rustup` resolve to the toolchain
  baked into the image; stable, with rustfmt + clippy + rust-analyzer)
- Prefer cargo for everything: `cargo add`, `cargo build`, `cargo clippy`,
  `cargo fmt`, `cargo test`, `cargo doc`
- Toolchain updates in-sandbox: `rustup update`; a different series:
  `rustup toolchain install <version>`
- Reinstalling/updating rustup itself: rerun the installer —
  `curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path`
  (installs user-level, shadowing the baked rustup)
