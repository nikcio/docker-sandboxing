# repo-clone

Clones extra GitHub repositories into the sandbox workspace at sandbox
creation, authenticated through the same proxy-managed GitHub token the
[`github-cli`](../github-cli) mixin uses.

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml` and set
the `repositories` argument. The clean way is to declare an environment
argument and pass it through (pin `&ref=<tag>` to a release, as the
[examples](../../examples) do):

```yaml
args:
  repos:
    default: "owner/api owner/docs"   # one owner/repo per line works too

kits:
  - source: git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/repo-clone&ref=v2.3.0
    args:
      repositories: ${{ env.args.repos }}
```

Run `sbx env run --env-arg repos=owner/other-repo` to override the list
for one invocation.

**Requires the `github-cli` mixin** in the same sandbox: the clone
authenticates via the proxy-managed `github` credential it declares.
Public repositories also clone fine as long as `github.com` egress is
allowed (the `github-cli` mixin allows it).

For local development, point `--kit` at the directory and pass the
argument directly:

```bash
sbx run <agent> . --kit ./mixins/repo-clone --kit-arg repositories="owner/api owner/docs"
```

## How it works

- Runs once at sandbox **creation** (an `setup.install` step): clones
  each `owner/repo` entry shallow (`--depth 1`) into
  `$WORKSPACE_DIR/<repo-name>`.
- **Auth**: git-over-HTTPS hits `github.com`, where the proxy injects
  the real `Authorization: Bearer <token>` from the `github` secret —
  the token itself never enters the sandbox (see
  [github-pat.md](../../docs/github-pat.md)). Private repositories the
  token can read clone normally; use a fine-grained PAT scoped to
  exactly the repositories you want cloned.
- **Failure tolerance**: a repository the token cannot access (or that
  does not exist) is skipped with a warning — one bad entry never fails
  the sandbox. Check the creation log for `repo-clone:` lines.
- **Idempotent**: an existing directory with the target name is left
  untouched, so re-creating the sandbox does not clobber anything.
- Repositories are cloned at creation only — new entries need a sandbox
  recreate (see "Changing mixins" in [docs/mixins.md](../../docs/mixins.md)).

## Network domains

| Domain | Why |
| ------ | --- |
| `github.com` | git clone over HTTPS (already allowed by the `github-cli` mixin) |

The mixin declares no network rules of its own beyond what the
`github-cli` mixin already allows for git-over-HTTPS.
