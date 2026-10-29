# repo-clone (v3)

Clones extra GitHub repositories into the sandbox workspace as a
`com.docker.sandbox/lifecycle@1` install hook. v3 port of
[`../repo-clone`](../repo-clone) — read that for the full behavior story
(auth via the proxy-managed github credential, failure tolerance); see
[`../README.md`](../README.md) for what the v3 format changes.

Declaration-only: no image content. The install hook declares
`env: [WORKSPACE_DIR]` explicitly — v3 hook environments are
deny-by-default.

## Kit args

| Arg | Default | Description |
| --- | ------- | ----------- |
| `repositories` | *(empty)* | Repos to clone, one `owner/repo` per line (whitespace separated works too) |

Set it from an environment argument (pin `&ref=<tag>` to a release):

```yaml
args:
  repos:
    default: "owner/api owner/docs"

kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/repo-clone&ref=<tag>
    args:
      repositories: ${{ env.args.repos }}
```

## Runtime egress

None of its own: `github.com` comes from the `github-cli` mixin (required
in the same sandbox).
