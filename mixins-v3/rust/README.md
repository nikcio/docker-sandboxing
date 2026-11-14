# rust (v3)

Rust toolchain (rustup-managed, with rust-analyzer) baked into the mixin
layer at image build, with crates.io egress.

## Build / use

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
`&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/rust   # Rust / cargo
```

For local development, point `--kit` at the directory instead —
`sbx run <agent> . --kit ./mixins-v3/rust` — or build the image yourself:

```bash
docker buildx build -f mixins-v3/rust/rust.yaml -t my/rust-mixin:dev mixins-v3/rust/
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `rustVersion` | *(empty = stable)* | rustup toolchain (`stable`, `beta`, `nightly`, or full version) |
| `profile` | `default` | rustup profile (`minimal`, `default`, `complete`) |

## How it works

- The rustup installer runs with `RUSTUP_HOME=/opt/rustup` and
  `CARGO_HOME=/opt/cargo`, installing the requested toolchain (stable by
  default) plus `rust-analyzer`, with the profile settable via the
  `profile` arg — the `default` profile includes rustfmt and clippy.
  Every rustup proxy in the cargo bin directory is linked onto
  `/usr/local/bin`, and a `/etc/profile.d/` script exports `RUSTUP_HOME`/
  `CARGO_HOME`, so `rustup update` and `rustup toolchain install` in login
  shells land in the same tree.
- The toolchain resolves at build, so the image is a snapshot: rebuild
  (or pin via the build args) to update it.

## Runtime egress

`crates.io` + wildcard (index/API queries for `cargo search`/`cargo
info`; the wildcard covers `index.crates.io`, the sparse index cargo
uses), `static.crates.io` (package files on the crates.io CDN),
`static.rust-lang.org` (rustup toolchain + component downloads, `rustup
update`), `sh.rustup.rs` + `raw.githubusercontent.com` (rustup installer
short link + its redirect target), `rust-lang.org` + wildcard (docs and
release info), `docs.rs` (crate docs). No install-phase grant: the
toolchain is baked at build.
