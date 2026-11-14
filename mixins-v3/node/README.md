# node (v3)

Node.js toolchain (nvm-managed Node + pnpm) baked into the mixin layer at
image build, with the npm ecosystem egress.

## Build / use

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin
`&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/node   # node / npm / pnpm
```

For local development, point `--kit` at the directory instead —
`sbx run <agent> . --kit ./mixins-v3/node` — or build the image yourself:

```bash
docker buildx build -f mixins-v3/node/node.yaml -t my/node-mixin:dev mixins-v3/node/
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `nodeVersion` | *(empty = latest)* | Node.js version — `MAJOR`, `MAJOR.MINOR`, or full |
| `pnpmVersion` | *(empty = latest)* | PNPM version |
| `nvmVersion` | *(empty = latest release)* | NVM ref to clone (`vMAJOR.MINOR.PATCH`) |

## How it works

- The toolchain is baked into the mixin image: nvm is cloned to
  `/opt/nvm`, the requested Node (latest by default) is installed and
  aliased as the default, and `node`/`npm`/`npx`/`pnpm`/`pnpx`/`corepack`
  are symlinked onto `/usr/local/bin`. A `/etc/profile.d/` script exports
  `NVM_DIR` and sources `nvm.sh`, so login shells can list, install, or
  switch Node versions — that runtime use is why the egress below includes
  the Node distribution hosts.
- The versions resolve "latest" at build, so the image is a snapshot:
  rebuild (or pin via the build args) to update it. Every sandbox composed
  with the mixin starts with the toolchain present — no create-time
  install and no egress needed for it.
- **pnpm version gate.** `PNPM_CONFIG_MINIMUM_RELEASE_AGE=1440` ships as
  image environment: pnpm refuses versions published less than 24h ago,
  guarding against freshly published compromised packages.

## Runtime egress

`nodejs.org` + wildcard, `iojs.org`, `github.com` + wildcard +
`raw.githubusercontent.com` (nvm), `registry.npmjs.org` + wildcards,
`npmjs.com` + wildcard, `pnpm.io` (docs and settings reference; pnpm's own
errors point here). No install-phase grant: the toolchain is baked at
build.
