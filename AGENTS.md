# Docker sandboxing

Docker Sandboxes kits and template images for running OpenCode agents
(.NET + Node) in sandboxed VMs.

## Documentation

Use based on your task:

- **[Versioning](agent-guidance/versioning.md)** — release-please flow,
  version bumps, image publishing, adding an image. Read when preparing a
  release or touching pinned versions.
- **[Commit Messages](agent-guidance/commit-messages.md)** — Conventional
  Commits style, types, scopes, examples. Read when writing commit messages.

## Repository specifics

This repo is published at `github.com/nikcio/docker-sandboxing` and produces
these Docker Sandboxes artifacts:

- `template/Dockerfile` — the sandbox template image
  (`opencode-node-dotnet:v1`): OpenCode base image + .NET SDK, Node via NVM,
  PNPM, Git, and the `o` PATH shim for relaunching opencode (a shim, not an
  rc alias, so it works in every shell context). Rebuild and reload after
  changes (`scripts/bootstrap.ps1` / `scripts/bootstrap.sh`, or
  `docker build` + `docker image save` + `sbx template load`).
- `kit/` — thin declarative sandbox kit (`schemaVersion: "2"`,
  `kind: sandbox`, `extends: opencode`): template image + entrypoint +
  a permissive OpenCode config (`kit/files/home/.config/opencode/opencode.jsonc`,
  dropped into the global config layer; edit/bash/webfetch allowed — the
  sandbox is the isolation boundary). Validate with `sbx kit validate kit/`.
- `kit-published/` — published variant of `kit/`: same spec except
  `sandbox.image` points at the public image on Docker Hub
  (`docker.io/nikcio/opencode-node-dotnet:vX.Y.Z`). Keep it in sync with
  `kit/`; release-please bumps its `version:` + image tag (and the `&ref=`
  pins in `examples/opencode-node-dotnet.sbxenv.yaml`) in the release PR.
- `mixins/<area>/` — one mixin kit per area (`kind: mixin`): `zeldoc`,
  `git`, `node`, `dotnet`, `docker`, `opencode-runtime`, `apt`. Each mixin
  must stay single-purpose — only the network rules, env vars, credentials,
  files, and memory notes for its own area. Composition is explicit at
  launch (`--kit` flags or a `.sbxenv.yaml` `kits:` list); the spec's
  `mixins:` field is not applied by the runtime yet. `mixins/zeldoc/files/`
  holds the Zeldoc opencode config loaded via `OPENCODE_CONFIG`.

Constraints to respect:

- The kit entrypoint is a shell wrapper, not opencode directly: it prints a
  startup banner, auto-runs `opencode`, then `exec`s an interactive login
  shell — quitting the agent must leave a usable shell. Keep that shape.
- The sandbox `AGENTS.md` (written next to the workspace) is rebuilt by the
  kit entrypoint before opencode starts: base content from
  `kit/files/home/.sandbox-agents.md` plus every mixin's
  `mixins/<area>/files/home/.sbx-agents.d/<area>.md`, appended directly. Do
  not use `agentInstructions` for mixin memory — it can only append after
  the runtime's built-in baseline and lands in a Kits-index side file.
- Sandboxes run with a deny-by-default network policy. The union of the
  composed mixins' `permissions.network` is the agent's only egress. If a
  download fails inside a sandbox, check `sbx policy log`, add the host to
  the owning mixin, and recreate the sandbox (kit changes never apply to
  running sandboxes).
- Kits are fetched from this GitHub repo by default; `github.com/nikcio/`
  must stay in the host's `kit.allowedSources` setting (bootstrap merges it).
- `scripts/new-sandbox.*` is the configurable `sbx-new` launcher —
  wizard-first (no args = guided prompts; prompts must go to stderr in bash
  so command substitution only captures the answer), flags/env for scripted
  use (profiles, git/local source, ref pinning). Keep the PS and bash
  variants in sync.
- Do not touch the sandbox-managed `~/.config/opencode/opencode.json` from
  any kit. The base kit's permissive config lives in the sibling
  `opencode.jsonc` (merged by OpenCode); the Zeldoc provider lives in its
  own file referenced by `OPENCODE_CONFIG`.
- Host-side settings (e.g. `clipboard.imagePaste`) belong in
  `scripts/bootstrap.*`, not in the kit.
- Never commit secrets. The Zeldoc key is registered host-side via
  `sbx secret set zeldoc` (service secret; a binding approval lives in
  `%APPDATA%\sbx\credentials.yaml` / `~/.config/sbx/credentials.yaml`) and
  stays out of the sandbox VM.
- Do not commit build outputs: `dist/`, `*.tar`, `*.zip`,
  `local.sbxenv.yaml`.
- Releases are cut by release-please from Conventional Commits on `main`
  (`feat`/`fix`/`deps` trigger a release; other types do not). Merging the
  release PR tags `vX.Y.Z`, and `publish-image.yml` pushes every image in
  `images.json` to Docker Hub as public `:vX.Y.Z` + `:latest` images
  (needs `DOCKERHUB_USERNAME` + `DOCKERHUB_TOKEN` repo secrets). Never
  bump pinned versions by hand — the release PR owns every
  `x-release-please` block.
