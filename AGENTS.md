# AGENTS.md

Instructions for AI coding agents working in this repository.

## Git worktrees — required workflow

Do all code changes in a dedicated git worktree so the primary checkout stays
clean. Never commit directly to `main`/`master`, and never edit files in the
primary checkout while a worktree for the task exists.

1. If `.worktrees/` is not git-ignored yet, add it to `.gitignore` first.
2. From the repository root, create a worktree per task:
   `git worktree add .worktrees/<task-name> -b <task-name>`
3. Work only inside `.worktrees/<task-name>/` — edit, build, and run tests
   there.
4. Commit in small, focused commits with clear messages.
5. When done, merge the branch or push it and open a PR, then clean up:
   `git worktree remove .worktrees/<task-name> && git branch -d <task-name>`
6. Run `git worktree prune` if a stale worktree lingers.

## Repository specifics

This repo produces two Docker Sandboxes artifacts:

- `template/Dockerfile` — the sandbox template image
  (`opencode-node-dotnet:v1`): OpenCode base image + .NET SDK, Node via NVM,
  PNPM, Git. Rebuild and reload after changes (`scripts/bootstrap.ps1` /
  `scripts/bootstrap.sh`, or `docker build` + `docker image save` +
  `sbx template load`).
- `kit/` — thin declarative sandbox kit (`schemaVersion: "2"`,
  `kind: sandbox`, `extends: opencode`): template image + entrypoint only.
  Validate with `sbx kit validate kit/`.
- `mixins/<area>/` — one mixin kit per area (`kind: mixin`): `zeldoc`,
  `git`, `node`, `dotnet`, `docker`, `opencode-runtime`, `apt`. Each mixin
  must stay single-purpose — only the network rules, env vars, credentials,
  files, and memory notes for its own area. Composition is explicit at
  launch (`--kit` flags or a `.sbxenv.yaml` `kits:` list); the spec's
  `mixins:` field is not applied by the runtime yet. `mixins/zeldoc/files/`
  holds the Zeldoc opencode config loaded via `OPENCODE_CONFIG`.

Constraints to respect:

- Sandboxes run with a deny-by-default network policy. The union of the
  composed mixins' `permissions.network` is the agent's only egress. If a
  download fails inside a sandbox, check `sbx policy log`, add the host to
  the owning mixin, and recreate the sandbox (kit changes never apply to
  running sandboxes).
- Do not touch the sandbox-managed `~/.config/opencode/opencode.json` from
  the kit; the Zeldoc provider lives in its own file referenced by
  `OPENCODE_CONFIG`.
- Host-side settings (e.g. `clipboard.imagePaste`) belong in
  `scripts/bootstrap.*`, not in the kit.
- Never commit secrets. The Zeldoc key is registered host-side via
  `sbx secret set zeldoc` (service secret; a binding approval lives in
  `%APPDATA%\sbx\credentials.yaml` / `~/.config/sbx/credentials.yaml`) and
  stays out of the sandbox VM.
- Do not commit build outputs: `dist/`, `*.tar`, `*.zip`,
  `local.sbxenv.yaml`.
