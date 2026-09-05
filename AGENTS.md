# Docker sandboxing

Docker Sandboxes kits and template images for running OpenCode agents
(.NET + Node, Python + uv) in sandboxed VMs.

## Documentation

Use based on your task:

- **[Versioning](agent-guidance/versioning.md)** — release-please flow,
  version bumps, image publishing, adding an image. Read when preparing a
  release or touching pinned versions.
- **[Commit Messages](agent-guidance/commit-messages.md)** — Conventional
  Commits style, types, scopes, examples. Read when writing commit messages.
- **User guides (`docs/`)** — consumer-facing step guides (getting started,
  Zeldoc API key, GitHub PAT, mixins, in-project kits, troubleshooting).
  Keep them simple and step-based: `examples/` stays a minimal
  "copy → adjust → `sbx env run`"
  template, and every "how do I …" that a user needs belongs in `docs/`,
  not in spec comments.

## Repository specifics

This repo is published at `github.com/nikcio/docker-sandboxing` and produces
these Docker Sandboxes artifacts:

- `template-node-dotnet/Dockerfile` — the sandbox template image
  (`opencode-node-dotnet:v1`): OpenCode base image + .NET SDK, Node via NVM,
  PNPM, Git, and the `o` PATH shim for relaunching opencode (a shim, not an
  rc alias, so it works in every shell context). Rebuild and reload after
  changes (`scripts/bootstrap.ps1` / `scripts/bootstrap.sh`, or
  `docker build` + `docker image save` + `sbx template load`).
- `template-python/Dockerfile` — the Python variant of the template image
  (`opencode-python:v1`): OpenCode base image + uv-managed CPython
  (default `PYTHON_VERSION=3.14`, user-level under the agent home, symlinked
  to `/usr/local/bin`) + uv (system-wide, unpinned like `gh`; updates come
  from image rebuilds), Git, and the same `o` shim. `UV_LINK_MODE=copy` is
  set because workspace bind mounts live on another filesystem than the uv
  cache.
- `kit-node-dotnet/` — thin declarative sandbox kit (`schemaVersion: "2"`,
  `kind: sandbox`, `extends: opencode`): template image + entrypoint +
  a permissive OpenCode config
  (`kit-node-dotnet/files/home/.config/opencode/opencode.jsonc`,
  dropped into the global config layer; edit/bash/webfetch allowed — the
  sandbox is the isolation boundary). Validate with
  `sbx kit validate kit-node-dotnet/`.
- `kit-python/` + `kit-published-python/` — Python + uv variants of
  `kit-node-dotnet/` and `kit-published-node-dotnet/` (`opencode-python`):
  same entrypoint/setup/files shape; only the memory base's environment
  facts and the config comment differ. Keep all four kits in sync when
  touching shared kit content.
- `kit-published-node-dotnet/` — published variant of `kit-node-dotnet/`:
  same spec except `sandbox.image` points at the public image on Docker Hub
  (`docker.io/nikcio/opencode-node-dotnet:vX.Y.Z`). Keep it in sync with
  `kit-node-dotnet/`; release-please bumps its `version:` + image tag (and
  the `&ref=` pins in `examples/*.sbxenv.yaml`) in the release PR.
- `mixins/<area>/` — one mixin kit per area (`kind: mixin`): `zeldoc`,
  `git`, `node`, `dotnet`, `python`, `docker`, `opencode-runtime`, `apt`,
  `browser`
  (Chrome, software only), three playwright levels (`playwright`,
  `playwright-chromium`, `playwright-all`), and `sbx` (the Docker
  Sandboxes CLI for in-sandbox kit authoring). Each mixin
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
  kit entrypoint before opencode starts: base content from the dev kit's
  `files/home/.sandbox-agents.md` plus every mixin's
  `mixins/<area>/files/home/.sbx-agents.d/<area>.md`, appended directly. Do
  not use `agentInstructions` for mixin memory — it can only append after
  the runtime's built-in baseline and lands in a Kits-index side file.
- Sandboxes run with a deny-by-default network policy. The union of the
  composed mixins' `permissions.network` is the agent's only egress. If a
  download fails inside a sandbox, check `sbx policy log`, add the host to
  the owning mixin, and recreate the sandbox (kit changes never apply to
  running sandboxes).
- Kits are fetched from this GitHub repo by default; `github.com/nikcio/`
  must stay in the host's `kit.allowedSources` setting (documented as a
  standard `sbx settings set` command in `docs/getting-started.md`).
- `scripts/new-sandbox.*` is the sandbox creation wizard —
  wizard-first (no args = guided prompts; prompts must go to stderr in bash
  so command substitution only captures the answer), flags/env for scripted
  use (profiles, git/local source, ref pinning). Keep the PS and bash
  variants in sync. Nothing registers it automatically; docs call it as
  `./scripts/new-sandbox.*`.
- Do not touch the sandbox-managed `~/.config/opencode/opencode.json` from
  any kit. The base kit's permissive config lives in the sibling
  `opencode.jsonc` (merged by OpenCode); the Zeldoc provider lives in its
  own file referenced by `OPENCODE_CONFIG`.
- Host-side settings (e.g. `clipboard.imagePaste`) are documented as
  standard `sbx settings set` commands in the user guides (`docs/`), never
  in the kit — and `bootstrap.*` only automates template/kit/mixin
  development (build, load, secrets, validate).
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
