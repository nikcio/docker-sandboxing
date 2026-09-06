## Node.js / NVM / PNPM

- Node is managed by nvm
- Prefer `pnpm`
- pnpm installs only package versions published at least 24h ago
  (supply-chain safety, applies to transitive deps too). If a version is
  younger than that, wait or pin an older release — do not disable the
  age gate.
