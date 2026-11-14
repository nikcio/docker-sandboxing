# rust (v3)

Rust toolchain (rustup-managed, with rust-analyzer) baked into the mixin
layer at image build, with crates.io egress. v3 port of
[`../rust`](../rust) — read that for the full behavior story; see
[`../README.md`](../README.md) for what the v3 format changes.

## Build / use

```bash
docker buildx build -f mixins-v3/rust/rust.yaml -t my/rust-mixin:dev mixins-v3/rust/
sbx run <agent> . --kit ./mixins-v3/rust   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `rustVersion` | *(empty = stable)* | rustup toolchain (`stable`, `beta`, `nightly`, or full version) |
| `profile` | `default` | rustup profile (`minimal`, `default`, `complete`) |

## Runtime egress

`crates.io` + wildcard, `static.crates.io`, `static.rust-lang.org`,
`sh.rustup.rs`, `raw.githubusercontent.com`, `rust-lang.org` + wildcard,
`docs.rs`. No install-phase grant.
