# github-cli

Installs the **GitHub CLI** (`gh`) and wires proxy-managed GitHub auth so `gh` and git-over-HTTPS work without the token ever being readable inside the sandbox.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` (pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/github-cli   # GitHub CLI + auth
```

For local development, point `--kit` at the directory instead: `sbx run --kit ./mixins/github-cli <agent> .`

The sandbox network policy is deny-by-default, so `gh` and git-over-HTTPS fail until the GitHub hosts are allowed.

## How it works

- **Credential**: comes from the built-in agent kit (`opencode`, `shell`, `docker-agent`), which declares the proxy-managed `github` service — the sandbox only sees a sentinel `GH_TOKEN` placeholder; the proxy injects the real `Authorization: Bearer <token>` header on requests to the GitHub domains. Register the token host-side with `sbx secret set github` (see [docs/github-pat.md](../../docs/github-pat.md)) and approve the injection when sbx asks. Use a fine-grained PAT scoped to only the repositories and permissions the agent needs. The mixin deliberately declares no credential of its own: sbx composition rejects two artifacts defining the same service, and the built-in one already covers the hosts below.
- **Auth domains** (via the built-in credential): `api.github.com`, `github.com`, and `raw.githubusercontent.com`.
- `gh` itself is on `PATH` after creation. If the base image does not ship `gh`, the mixin installs it from the official `cli.github.com` apt repository (skipped when `gh` is already available).
- **Git identity**: a startup command resolves the GitHub login, name, and email from `GH_TOKEN` (via `gh api user`) and writes them into the sandbox's git config. If the account has no public email set, the GitHub noreply address (`<id>+<login>@users.noreply.github.com`) is used instead. The committer identity is then locked down: a `/usr/local/bin/git` wrapper forces the identity from `/etc/git-identity` and refuses `git config` overrides of `user.name`/`user.email`, and both `/etc/gitconfig` and the agent's `~/.gitconfig` are made read-only (`chattr +i`). The committer on every commit is therefore always the token owner; note this is not bulletproof — the agent can still set the *author* on individual commits (`git commit --author=...`) or call a git binary directly.
- **Non-interactive git defaults**: the sandbox has no interactive editor, so the startup command also sets `core.editor` to `true` (a no-op command) and configures merge pulls (`pull.rebase=false`, `merge.ff=false`). This makes `git pull` and `git merge` work from the git CLI without hanging on an editor prompt.

## Version arguments

| Arg | Default | Description |
| --- | ------- | ----------- |
| `ghVersion` | *(empty = latest apt package)* | GitHub CLI version to install (`MAJOR[.MINOR[.PATCH]]`) |

Pass values with `--kit-arg github-cli.ghVersion=<version>` (or an `args:` entry on the mixin's `kits:` line in `sbxenv.yaml`) — see [Version arguments](../../docs/mixins.md#version-arguments).

## Network domains

| Domain | Why |
| ------ | --- |
| `cli.github.com` | gh CLI apt repo (install + upgrades) |
| `github.com`, `*.github.com` | `gh` API calls, git-over-HTTPS, gists |
| `*.githubusercontent.com` | Raw content, avatars, release assets |
