# Docker sandboxing

Docker Sandboxes kits and template images for running OpenCode agents
(.NET + Node, Node, Python + uv, Go, Rust + cargo) in sandboxed VMs.

## Documentation

Use based on your task:

- **[Worktrees](agent-guidance/worktrees.md)** — the mandatory isolated
  workspace for every code change. Read before writing any code in this
  repo.
- **[Versioning](agent-guidance/versioning.md)** — release-please flow,
  version bumps, image publishing, adding an image. Read when preparing a
  release or touching pinned versions.
- **[Commit Messages](agent-guidance/commit-messages.md)** — Conventional
  Commits style, types, scopes, examples. Read when writing commit messages.
- **User guides (`docs/`)** — consumer-facing step guides. Keep them simple
  and step-based: `examples/` stays a minimal "copy → adjust →
  `sbx env run`" template, and every "how do I …" that a user needs belongs
  in `docs/`, not in spec comments.

## Artifacts

- `template-*/Dockerfile` — the sandbox template images
  (`opencode-node-dotnet:v1`, `opencode-node:v1`, `opencode-python:v1`,
  `opencode-go:v1`, `opencode-rust:v1`): the OpenCode base image plus the
  stack toolchain, Git, and the `o` PATH shim for relaunching opencode.
  Rebuild and reload after changes (`scripts/bootstrap.ps1` /
  `scripts/bootstrap.sh`, or `docker build` + `docker image save` +
  `sbx template load`).
- `kit-<stack>/` — thin declarative sandbox kits (`schemaVersion: "2"`,
  `kind: sandbox`): template image + a one-line
  entrypoint wrapper that execs the shared runtime from `mixins/base/`.
  Validate with `sbx kit validate kit-<stack>/`.
- `kit-published-<stack>/` — same spec as the dev kit except
  `sandbox.image` points at the public Docker Hub image; release-please
  bumps its `version:` + image tag (and the `&ref=` pins in
  `examples/*.sbxenv.yaml`) in the release PR.
- `mixins/base/` — the entrypoint runtime, AGENTS.md logic, and MCP
  gateway every OpenCode kit sandbox composes (`kind: mixin`): the
  entrypoint runtime (`files/home/.sandbox-kit/entrypoint.sh`: mixin hook
  runner, opencode autostart, login shell on exit) the kit wrappers exec,
  the agent guidance files (`files/home/.sandbox-agents/*.md`), the
  shared `AGENTS.md` base (`files/home/.sandbox-agents.md`), the AGENTS.md
  rebuild hook (`files/home/.sandbox-kit/hooks.d/agents-md.sh`), MCP
  gateway registration (startup hook + idempotent backstop hook
  `files/home/.sandbox-kit/hooks.d/mcp-gateway.sh`), and the toolchain
  install library (`files/home/.sandbox-kit/lib/install-toolchain.sh`)
  other mixins' check-and-install steps source (any mixin composes on any
  base template image — installs the tool only when the template lacks
  it).
- `mixins/global-opencode-config/` — the permissive OpenCode config
  (`files/home/.config/opencode/opencode.jsonc`, dropped into the global
  config layer; edit/bash/webfetch allowed — the sandbox is the isolation
  boundary; sharing + websearch disabled) plus the combined provider
  config: `environment.variables.OPENCODE_CONFIG` points at
  `~/.config/opencode/providers.jsonc` (owned here — provider mixins must
  not set it), rebuilt at every start by
  `files/home/.sandbox-kit/merge-global-opencode-config.sh` from the provider
  mixins' `providers.d/NN-<provider>.json` fragments; the merge runs
  through the mixin's `files/home/.sandbox-kit/hooks.d/provider-config.sh`
  hook (sourced by the entrypoint runtime, which also re-exports the var
  defensively).
- `mixins/env-guard/` — the workspace `.env` guard (`kind: mixin`): the
  no-.env policy as a hook (`files/home/.sandbox-kit/hooks.d/env-guard.sh`,
  sourced by the entrypoint runtime) — clone mode removes `.env` files
  from the sandbox-local copy, direct mode refuses to start.
- `mixins/banner/` — the startup banner (`kind: mixin`): a hook
  (`files/home/.sandbox-kit/hooks.d/banner.sh`, sourced by the entrypoint
  runtime) printing the "opencode is starting automatically" notice.
  Cosmetic — the examples compose it.
- The `base` mixin is required by every kit: every sandbox's `kits:` list
  must include it. `global-opencode-config` is required when composing a
  model provider (`zeldoc`, `copilot`) — their fragments only merge
  through it. `env-guard` and `banner` are optional (the examples compose
  both).
- `mixins/<area>/` — one single-purpose mixin kit per area (`kind: mixin`):
  only the network rules, env vars, credentials, files, and memory notes
  for its own area. Composition is explicit at launch (`--kit` flags or a
  `.sbxenv.yaml` `kits:` list); the spec's `mixins:` field is not applied
  by the runtime yet. Model-provider mixins (`zeldoc`, `copilot`) ship
  pure-JSON config fragments to
  `files/home/.config/opencode/providers.d/NN-<provider>.json` — the
  global-opencode-config mixin merges them (filename order, `enabled_providers`
  unioned, later files win scalar conflicts) into the combined file that
  `OPENCODE_CONFIG` points at. Toolchain/tool mixins (`node`, `dotnet`,
  `python`, `go`, `rust`, `browser`, `playwright*`, `sbx`,
  `nikcio-openapi-codegen`) carry check-and-install `setup.install` steps
  (sourcing the base mixin's install library) so any mixin composes on any
  base template image — keep that shape in new mixins: guard the install
  on the tool already being present.
- `scripts/new-sandbox.*` — the sandbox creation wizard: wizard-first
  (no args = guided prompts), flags/env for scripted use (profiles,
  git/local source, ref pinning). Nothing registers it automatically; docs
  call it as `./scripts/new-sandbox.*`.

## Constraints

- Always work in a dedicated git worktree branched from `origin/main` —
  never in the main checkout: multiple agent sessions share it, it can
  switch branches under you, and its untracked files are not yours. See
  [agent-guidance/worktrees.md](agent-guidance/worktrees.md).
- Keep all ten kits in sync when touching shared kit content — the only
  intended differences are the image reference and the version. Shared
  content lives once in `mixins/` (entrypoint runtime + guidance files +
  AGENTS.md rebuild + MCP gateway in `base`, permissive OpenCode config +
  provider merge in `global-opencode-config`, the `.env` guard in
  `env-guard`, the banner in `banner`); don't reintroduce copies into the
  kits.
- The kit entrypoint is a shell wrapper, not opencode directly: it execs
  the shared runtime script from the `base` mixin
  (`~/.sandbox-kit/entrypoint.sh`), which runs the composed mixins' hooks
  from `~/.sandbox-kit/hooks.d/` in glob order (the base mixin's
  AGENTS.md rebuild + MCP backstop, the banner mixin's banner, the
  env-guard mixin's guard, the config mixin's provider merge), then
  auto-runs `opencode` and `exec`s an interactive login shell — quitting
  the agent must leave a usable shell. Keep that shape; the wrapper must
  error clearly (and still drop into a login shell) when the `base` mixin
  is missing. Entry-point ordering (AGENTS.md rebuild, banner, guard
  refusal, MCP backstop, provider merge) must stay in these hooks, not
  `setup.startup` — startup commands run alongside the entrypoint without
  gating it and without a terminal (kit reference: setup.startup).
- The sandbox `AGENTS.md` (written next to the workspace) is rebuilt
  before opencode starts: the entrypoint runtime sources the base mixin's
  rebuild hook (`files/home/.sandbox-kit/hooks.d/agents-md.sh`), which
  writes the base content from the base mixin's
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
- Prompts in `scripts/new-sandbox.sh` must go to stderr so command
  substitution only captures the answer. Keep the PS and bash variants in
  sync.
- Do not touch the sandbox-managed `~/.config/opencode/opencode.json` from
  any kit. The permissive config lives in the `global-opencode-config` mixin's
  sibling `opencode.jsonc` (merged by OpenCode); providers layer on via
  `providers.d/` fragments (merged by the global-opencode-config mixin's
  `merge-global-opencode-config.sh` into the generated `providers.jsonc` — that
  file is rebuilt every start, never hand-edit it).
- Never set `OPENCODE_CONFIG` in a kit or mixin — the `global-opencode-config`
  mixin owns it. That single owner is what lets any number of provider
  mixins compose.
- Host-side settings are documented as standard `sbx settings set` commands
  in the user guides (`docs/`), never in the kit.
- Never commit secrets. The Zeldoc key is registered host-side via
  `sbx secret set zeldoc` and stays out of the sandbox VM.
- Do not commit build outputs: `dist/`, `*.tar`, `*.zip`,
  `local.sbxenv.yaml`.
- Releases are cut by release-please from Conventional Commits on `main`
  (`feat`/`fix`/`deps` trigger a release; other types do not). Never bump
  pinned versions by hand — the release PR owns every `x-release-please`
  block. See [Versioning](agent-guidance/versioning.md).
