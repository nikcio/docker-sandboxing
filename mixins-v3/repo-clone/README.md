# repo-clone (v3)

Clones extra GitHub repositories into the sandbox workspace at sandbox
creation, authenticated through the same proxy-managed GitHub token the
`github-cli` mixin uses.

Declaration-only: no image content, nothing to build. The clone is a
`com.docker.sandbox/lifecycle@1` install hook; the hook declares
`env: [WORKSPACE_DIR]` explicitly — hook environments are
deny-by-default.

## Build / use

Point a sandbox at the mixin and set the `repositories` argument. The
clean way is to declare an environment argument and pass it through
(pin `&ref=<tag>` to a release, as the examples do):

```yaml
args:
  repos:
    default: "owner/api owner/docs"   # one owner/repo per line works too

kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/repo-clone&ref=<tag>
    args:
      repositories: ${{ env.args.repos }}
```

Run `sbx env run --env-arg repos=owner/other-repo` to override the list
for one invocation.

For local development, point `--kit` at the directory instead:

```bash
sbx run <agent> . --kit ./mixins-v3/repo-clone
```

The `github-cli` mixin is required in the same sandbox (for the
`github.com` egress): the clone authenticates via the proxy-managed
`github` credential the built-in agent kit declares. Public repositories
also clone fine as long as `github.com` egress is allowed (the
`github-cli` mixin allows it).

## Kit args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `repositories` | *(empty)* | Repos to clone, one `owner/repo` per line (whitespace separated works too) |

The value is interpolated into a single-quoted shell string, so a `'` in
a repository name would break the quoting. Stick to `owner/repo` names
(they cannot contain quotes anyway).

## How it works

- Runs once at sandbox creation (the lifecycle install hook). It clones
  each `owner/repo` entry shallow (`--depth 1`) into
  `$WORKSPACE_DIR/<repo-name>`.
- **Auth.** Git-over-HTTPS hits `github.com`, where the proxy injects
  the real token from the `github` secret. The token itself never enters
  the sandbox (see
  [docs/github-pat.md](../../docs/github-pat.md)). Private repositories
  the token can read clone normally; use a fine-grained PAT scoped to
  exactly the repositories you want cloned.
- **Failure tolerance.** A repository the token cannot access (or that
  does not exist) is skipped with a warning, so one bad entry never
  fails the sandbox. A failed clone leaves nothing behind (the partial
  target directory is removed), and entries that are not `owner/repo`
  are skipped. Check the creation log for `repo-clone:` lines.
- **Idempotent.** An existing directory (or symlink) with the target
  name is left untouched, so re-creating the sandbox does not clobber
  anything.
- Repositories are cloned at creation only. New entries need a sandbox
  recreate (see "Changing mixins" in
  [docs/mixins.md](../../docs/mixins.md)).

## Runtime egress

None of its own: `github.com` comes from the `github-cli` mixin
(required in the same sandbox).
