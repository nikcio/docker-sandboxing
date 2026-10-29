# node (v3)

Node.js toolchain (nvm-managed Node + pnpm) baked into the mixin layer at
image build, with the npm ecosystem egress. v3 port of
[`../node`](../node) — read that for the full behavior story; see
[`../README.md`](../README.md) for what the v3 format changes.

## Build / use

```bash
docker buildx build -f mixins-v3/node/node.yaml -t my/node-mixin:dev mixins-v3/node/
sbx run <agent> . --kit ./mixins-v3/node   # or the published image reference
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `nodeVersion` | *(empty = latest)* | Node.js version — `MAJOR`, `MAJOR.MINOR`, or full |
| `pnpmVersion` | *(empty = latest)* | PNPM version |
| `nvmVersion` | *(empty = latest release)* | NVM ref to clone (`vMAJOR.MINOR.PATCH`) |

## Runtime egress

`nodejs.org` + wildcard, `iojs.org`, `github.com` + wildcard +
`raw.githubusercontent.com` (nvm), `registry.npmjs.org` + wildcards,
`npmjs.com` + wildcard, `pnpm.io`. No install-phase grant: the toolchain
is baked at build.
