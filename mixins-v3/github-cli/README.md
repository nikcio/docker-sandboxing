# github-cli (v3)

GitHub CLI (`gh`) baked into the mixin layer at image build, with gh and
git-over-HTTPS egress plus a startup hook that locks the git committer
identity to the owner of the github credential. The sandbox network
policy is deny by default, so `gh` and git-over-HTTPS fail until these
hosts are allowed.

## Build / use

```bash
docker buildx build -f mixins-v3/github-cli/github-cli.yaml -t my/gh-mixin:dev mixins-v3/github-cli/
sbx run <agent> . --kit ./mixins-v3/github-cli   # or the published image reference
```

In a project's `sbxenv.yaml`, add the mixin to the `kits:` list (pin
`&ref=<tag>` to a release, as the examples do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/github-cli   # GitHub CLI + auth
```

## Build args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `ghVersion` | *(empty = latest apt package)* | gh version (`MAJOR[.MINOR[.PATCH]]`) |

Args resolve when the image builds; pass them with an `args:` entry on
the mixin's `kits:` line.

## How it works

- **Install.** At image build, gh is installed from the official
  `cli.github.com` apt repository (a no-op on bases that already ship
  gh). The apt repo definition and keyring ship in the layer, so
  `apt upgrade` inside the sandbox keeps updating gh.
- **Credential.** Comes from the built-in agent kit (`opencode`,
  `shell`, `docker-agent`), which declares the proxy-managed `github`
  service. The sandbox only sees a sentinel `GH_TOKEN` placeholder; the
  proxy injects the real `Authorization: Bearer <token>` header on
  requests to the GitHub domains. Register the token host-side with
  `sbx secret set github` (see
  [docs/github-pat.md](../../docs/github-pat.md)) and approve the
  injection when sbx asks. Use a fine-grained PAT scoped to only the
  repositories and permissions the agent needs. The mixin deliberately
  declares no credential of its own: composition rejects two artifacts
  defining the same service, and the built-in one already covers the
  hosts below.
- **Auth domains** (via the built-in credential): `api.github.com`,
  `github.com`, and `raw.githubusercontent.com`.
- **Git identity.** A startup hook (runs as root on every boot) resolves
  the GitHub login name and email through `gh api user` — the request
  rides the built-in proxy-managed credential, so the hook only sees the
  sentinel `GH_TOKEN` — and locks the committer identity: both
  `/etc/gitconfig` and the agent's `~/.gitconfig` are written and made
  read-only (`chattr +i`), and a `/usr/local/bin/git` wrapper exports the
  identity from `/etc/git-identity` and refuses `git config` overrides of
  `user.name`/`user.email`. If the account has no public email, the GitHub
  noreply address (`<id>+<login>@users.noreply.github.com`) is used. The
  committer on every commit is therefore always the credential owner; this
  is not bulletproof, because the agent can still set the *author* on
  individual commits (`git commit --author=...`) or call a git binary
  directly. Skipped with a warning when `gh`, `git`, or the credential are
  unavailable.

## Runtime egress

| Domain | Why |
| ------ | --- |
| `cli.github.com` | gh apt repo (package + keyring are baked at build; kept for `apt-get upgrade` runs inside the sandbox) |
| `github.com`, `*.github.com` | `gh` API calls, git-over-HTTPS, gists |
| `*.githubusercontent.com` | Raw content, avatars, release assets |

No install-phase grant: the install is baked at build.
