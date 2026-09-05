## Rust / cargo

- Rust is rustup-managed (`cargo`/`rustc`/`rustup` resolve to the toolchain
  baked into the image; stable, with rustfmt + clippy + rust-analyzer)
- Prefer cargo for everything: `cargo add`, `cargo build`, `cargo clippy`,
  `cargo fmt`, `cargo test`, `cargo doc`
- Toolchain updates in-sandbox: `rustup update` (downloads from
  static.rust-lang.org, allowed by this mixin); a different series:
  `rustup toolchain install <version>`
- Reinstalling/updating rustup itself: rerun the installer —
  `curl -fsSL https://sh.rustup.rs | sh -s -- -y --no-modify-path`
  (installs user-level, shadowing the baked rustup; the script redirects
  to raw.githubusercontent.com, so compose the `git` mixin)
- Git dependencies fetch over github.com — compose the `git` mixin
- Docs lookups: the rust-lang.org sites and docs.rs are reachable
  (doc.rust-lang.org — std/book/reference/edition guide,
  www.rust-lang.org — releases, docs.rs — crate docs)
