# Git authentication

The proxy injects GitHub credentials for HTTPS Git operations — no `gh auth login` needed.
`gh auth status` showing "not logged in" inside the sandbox is expected and does not break Git.

## If `git push` fails with auth errors

`fatal: could not read Username for 'https://github.com'` means no GitHub token secret is
provisioned for this sandbox yet. Tokens live only in sbx's host-side secret store (the OS
keychain), scoped per sandbox — **fine-grained PATs only, never the broad-scope host `gh`
token, and never in files on disk**. The normal flow: the user launches with `sbx-env` (this
repo's host launcher), which prompts through sbx before the sandbox starts. If this sandbox
started without one, ask the user to run on their host, using `$SANDBOX_NAME` (not the branch
or worktree path):

```bash
sbx secret set github --sandbox <sandbox-name>   # prompts; takes effect immediately
```

Rotation is the same command; `sbx env rm` removes the scoped secret again.

## Pushing and PRs

Push and open PRs directly from inside the sandbox — never ask the user to push from their host
terminal. Every non-local host remote is mirrored here under the same name (`origin`, `upstream`,
forks; local-path remotes excepted):

```bash
git push -u origin <branch>
gh pr create --fill
```

If `origin` points at the org repo rather than the fork, push to the fork remote instead:
`git push -u <fork-remote> <branch> && gh pr create --repo <org>/<repo> --head <fork-user>:<branch> --fill`.
