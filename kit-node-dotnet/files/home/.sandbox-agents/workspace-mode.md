# Git workspace mode

```bash
if [ -d /run/sandbox/source ]; then echo "clone mode"; else echo "direct mode"; fi
```

## Direct mode (default)

The host working tree is mounted here — edits, commits, and branches appear on the host
immediately. Stage, commit, and push exactly as on the host.

## Clone mode (`--clone`)

A standalone Git clone of the host repo, taken at sandbox start. Nothing syncs automatically.

- HEAD matches the host checkout at create time; create your own branch with `git checkout -b`.
- Commits stay in the sandbox until the host fetches them: `git fetch sandbox-<name>`. That
  git-daemon remote only runs while the sandbox is up — commits never pushed to a Git host are
  lost if the sandbox is removed.
- Host commits made after sandbox start: `git fetch /run/sandbox/source` (the host repo is
  bind-mounted read-only there; `origin` points at the upstream Git host and lacks unpushed local
  commits), then merge/pull from `FETCH_HEAD`.
