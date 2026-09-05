# Git authentication

The proxy injects GitHub credentials for HTTPS Git operations — no `gh auth login` needed.
`gh auth status` showing "not logged in" inside the sandbox is expected and does not break Git.

## If `git push` fails with auth errors

`fatal: could not read Username for 'https://github.com'` means no GitHub token secret is
provisioned for this sandbox yet. Tokens are set per environment — **fine-grained PATs only,
never the broad-scope host `gh` token**. Ask the user to add a `secrets.github` entry to the
project's `.sbxenv.yaml` — an inline `command:` that resolves the environment's own variable
(e.g. `GITHUB_PAT_MY_PROJECT`) and prompts on `/dev/tty` when it is unset, plus a
`bindings.github` block (see `examples/opencode-node-dotnet.sbxenv.yaml`) — and recreate the
environment. Or, for an immediate fix on this sandbox only, run on their host, using
`$SANDBOX_NAME` (not the branch or worktree path):

```bash
sbx secret set github --sandbox <sandbox-name>   # prompts for the fine-grained PAT
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
