# node

Node.js toolchain egress for the sandbox: nvm-managed Node + pnpm install (when the base image lacks them), the npm registry for package installs, and docs hosts.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/node   # node / npm / pnpm
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/node <agent> .`

The install runs at sandbox creation only and needs egress. It uses the domains in the table below plus the Ubuntu apt mirrors (`archive.ubuntu.com`/`security.ubuntu.com`) for its `apt-get` calls. On base images that already ship Node the install is a no-op.

## How it works

- **Check-and-install.** When `node`/`pnpm` are missing, clones nvm into the agent home, installs the latest Node (or a pinned version) as the default, symlinks `node`/`npm`/`npx`/`pnpm`/… onto `PATH`, and exports `NVM_DIR` via `/etc/sandbox-persistent.sh`, the same nvm-in-agent-home layout as the base image.
- **pnpm version gate.** Sets `PNPM_CONFIG_MINIMUM_RELEASE_AGE=1440`, so pnpm refuses versions published less than 24h ago. This guards against freshly published compromised packages.

## Version arguments

| Arg | Default | Description |
| --- | ------- | ----------- |
| `nodeVersion` | *(empty = latest)* | Node.js version to install — a major (`MAJOR`), a partial (`MAJOR.MINOR`), or a full (`MAJOR.MINOR.PATCH`) |
| `pnpmVersion` | *(empty = latest)* | PNPM version to install — a major or a full version |
| `nvmVersion` | *(empty = latest release)* | NVM version to clone in the upstream `vMAJOR.MINOR.PATCH` form |

Pass values with `--kit-arg node.nodeVersion=<version>` (or an `args:` entry on the mixin's `kits:` line in `sbxenv.yaml`) — see [Version arguments](../../docs/mixins.md#version-arguments).

## Network domains

| Domain | Why |
| ------ | --- |
| `nodejs.org`, `*.nodejs.org` | Node distribution downloads for nvm |
| `iojs.org` | nvm's io.js mirror (fetched by every `nvm ls-remote`) |
| `github.com`, `*.github.com`, `raw.githubusercontent.com` | nvm itself (repo clone / install script) |
| `registry.npmjs.org`, `*.npmjs.org`, `npmjs.com`, `*.npmjs.com` | npm registry (pnpm/npm/npx installs) |
| `pnpm.io:443` | pnpm docs and settings reference |
