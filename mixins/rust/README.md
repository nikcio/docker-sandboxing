# rust

Rust toolchain egress for the sandbox: a rustup-managed stable toolchain
(when the template image lacks one), cargo, crates.io access, and docs
hosts.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - docker.io/nikcio/sbx-mixin-rust:vX.Y.Z   # Rust / cargo
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/rust <workload> .`

The install runs at **sandbox creation only** and needs egress — the
domains in the table below plus the Ubuntu apt mirrors
(`archive.ubuntu.com`/`security.ubuntu.com`) for its `apt-get` calls. On
template images that already ship cargo the install is a cheap no-op.

## How it works

- **Check-and-install**: when `cargo` is missing, installs the C build
  toolchain + OpenSSL dev headers via apt, then runs the rustup installer
  as the `agent` user with `RUSTUP_HOME`/`CARGO_HOME` in the agent home
  (stable toolchain, plus `rust-analyzer`), and symlinks the cargo bin
  directory onto `PATH` — the same agent-owned rustup layout as the
  template images.
- The env vars are exported via `/etc/sandbox-persistent.sh` so login
  shells pick them up.

## Network domains

| Domain | Why |
| ------ | --- |
| `crates.io`, `*.crates.io` | Index/API queries (`cargo search`/`cargo info`); the wildcard covers the sparse index |
| `static.crates.io:443` | Package files on the crates.io CDN |
| `static.rust-lang.org:443` | rustup toolchain + component downloads, `rustup update` |
| `sh.rustup.rs:443`, `raw.githubusercontent.com:443` | rustup installer (short link + redirect target) |
| `rust-lang.org`, `*.rust-lang.org` | Rust docs and release info |
| `docs.rs:443` | Crate docs |
