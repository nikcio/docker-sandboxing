# node

Node.js toolchain egress for the sandbox: nvm-managed Node + pnpm install (when the workload image lacks them), the npm registry for package installs, and docs hosts.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin the version to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/node&ref=vX.Y.Z   # node / npm / pnpm
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/node <workload> .`

The install runs at **sandbox creation only** and needs egress — the domains in the table below plus the Ubuntu apt mirrors (`archive.ubuntu.com`/`security.ubuntu.com`) for its `apt-get` calls. On workload images that already ship Node the install is a cheap no-op.

## How it works

- **Check-and-install**: when `node`/`pnpm` are missing, clones nvm into the agent home, installs Node 24 as the default, symlinks `node`/`npm`/`npx`/`pnpm`/… onto `PATH`, and exports `NVM_DIR` via `/etc/sandbox-persistent.sh` — the same nvm-in-agent-home layout as the workload images.
- **pnpm version gate**: sets `PNPM_CONFIG_MINIMUM_RELEASE_AGE=1440`, so pnpm refuses versions published less than 24h ago — a supply-chain guard against freshly published compromised packages.
- The agent note (shipped via the mixin's `agent-context` capability) tells the agent Node is nvm-managed and to prefer `pnpm`.

## Network domains

| Domain | Why |
| ------ | --- |
| `nodejs.org`, `*.nodejs.org` | Node distribution downloads for nvm |
| `iojs.org` | nvm's io.js mirror (fetched by every `nvm ls-remote`) |
| `github.com`, `*.github.com`, `raw.githubusercontent.com` | nvm itself (repo clone / install script) |
| `registry.npmjs.org`, `*.npmjs.org`, `npmjs.com`, `*.npmjs.com` | npm registry (pnpm/npm/npx installs) |
| `pnpm.io:443` | pnpm docs and settings reference |
