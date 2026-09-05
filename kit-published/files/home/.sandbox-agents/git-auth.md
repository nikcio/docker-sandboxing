# Git authentication

The proxy injects GitHub credentials for HTTPS Git operations — no `gh auth login` needed.
`gh auth status` showing "not logged in" inside the sandbox is expected and does not break Git.

## If `git push` fails with auth errors

`fatal: could not read Username for 'https://github.com'` means no GitHub token secret is
configured yet. Ask the user to run on their host, using `$SANDBOX_NAME` (not the branch or
worktree path):

```bash
sbx secret set github --sandbox <sandbox-name> -t "$(gh auth token)"  # existing sandbox, immediate
sbx secret set github -t "$(gh auth token)"                           # global; needs sandbox recreate
```

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
