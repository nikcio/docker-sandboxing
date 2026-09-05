# docker-sandboxing

A [Docker Sandboxes](https://docs.docker.com/ai/sandboxes/) sandbox for
[OpenCode](https://opencode.ai), composed from a **template image**, a thin
**sandbox kit**, and optional **mixins** (one capability area each: model
provider, git, node, dotnet, python, docker, apt, browser automation, …). Two
templates ship: **.NET + Node.js** (NVM + PNPM) and **Python + uv**, both with
Git and GitHub CLI preinstalled.

The sandbox is the isolation boundary: outbound network is deny-by-default
(only the composed mixins' hosts are allowed), and API keys are injected by
a proxy — the sandbox only ever sees placeholders.

## Use it in your project

1. **Copy the example.** Copy the example matching your stack —
   `examples/opencode-node-dotnet.sbxenv.yaml` (.NET + Node) or
   `examples/opencode-python.sbxenv.yaml` (Python + uv) — into your project's
   root and rename it `.sbxenv.yaml`. Commit it so teammates get the same
   sandbox.
2. **Adjust the config to your project.** Set `name:`; drop the mixin lines
   your project doesn't need (`workspace.path: .` targets the repo itself).
3. **Run it** from your project root.

   ```bash
   sbx env run
   ```

Full walkthrough: [docs/getting-started.md](docs/getting-started.md).

## Guides

| Guide | Covers |
| ----- | ------ |
| [Getting started](docs/getting-started.md) | prerequisites, the three steps, first run, daily use |
| [Set your Zeldoc API key](docs/zeldoc-api-key.md) | get, register, and approve the model provider key |
| [Create a GitHub PAT](docs/github-pat.md) | correct permissions and scope, store it per sandbox, rotate it |
| [Mixins](docs/mixins.md) | what each mixin adds, common sets, changing them |
| [Project-specific config](docs/project-kit.md) | an in-project kit: project feeds, env vars, files, agent notes |
| [Troubleshooting](docs/troubleshooting.md) | blocked downloads, git auth, the .env guard, stale changes |

## Mixins

| Mixin | Adds |
| ----- | ---- |
| `opencode-runtime` | egress the agent itself needs (updates, models.dev, plugins) |
| `zeldoc` | Zeldoc.ai model provider (proxy-managed key, config, hosts) |
| `git` | git hosting egress, proxy-managed GitHub auth, worktree workflow |
| `node` | nodejs.org + npm registry egress |
| `dotnet` | NuGet/Microsoft egress, telemetry opt-out |
| `python` | PyPI egress for uv/pip |
| `docker` | registry egress for the in-sandbox Docker engine |
| `apt` | Ubuntu/Microsoft package mirrors for `sudo apt-get` |
| `browser` | Google Chrome install (software only) |
| `playwright` / `playwright-chromium` / `playwright-all` | Playwright + the listed browsers |
| `sbx` | the `sbx` CLI inside the sandbox (kit authoring) |

Drop what you don't need — a pure Node project keeps only `opencode-runtime`,
`zeldoc`, `git`, `node`; a pure Python project keeps `opencode-runtime`,
`zeldoc`, `git`, `python`. The `playwright*` and `sbx` mixins need the `apt`
mixin. Details: [docs/mixins.md](docs/mixins.md).

## The sandbox shell

Attaching lands you in opencode directly. Quitting opencode drops you into
the sandbox's login shell (git, builds, `dotnet`/`pnpm`, …) — relaunch
opencode anytime with `o`, and exit the shell when you're done with the
sandbox.

## Developing this repo

Everything below is for developing the template, kit, and mixins.

```text
├── template-node-dotnet/Dockerfile         # sandbox template image (.NET + Node)
├── template-python/Dockerfile              # sandbox template image (Python + uv)
├── kit-node-dotnet/                        # sandbox kit (local image, .NET + Node)
├── kit-published-node-dotnet/              # same kit, published image tag
├── kit-python/                             # sandbox kit (local image, Python + uv)
├── kit-published-python/                   # same kit, published image tag
├── mixins/<area>/                          # one mixin per capability area
├── scripts/
│   ├── bootstrap.ps1 / bootstrap.sh        # local dev setup (build, load, secrets, validate)
│   └── new-sandbox.ps1 / new-sandbox.sh    # sandbox creation wizard
├── examples/*.sbxenv.yaml                  # consumer environment examples (one per stack)
├── docs/                                   # user guides (see above)
├── agent-guidance/                         # versioning + commit conventions
└── opencode-builtin-kit.spec.yaml          # reference copy of Docker's built-in opencode kit
```

- **Templates** (`template-node-dotnet/Dockerfile` → `opencode-node-dotnet:v1`,
  `template-python/Dockerfile` → `opencode-python:v1`): the Node image ships
  the .NET SDK, Node LTS via NVM, PNPM, Git (+ git-lfs), `gh`, and the `o`
  PATH shim for relaunching opencode; the Python image ships uv-managed
  CPython, uv, Git (+ git-lfs), `gh`, and the same `o` shim. Both extend
  `docker/sandbox-templates:opencode-docker`.
- **Kits** (`kit-node-dotnet/`, `kit-python/`, `kind: sandbox`): point at the
  local template images, set the entrypoint (banner → auto-start opencode →
  login shell on exit), drop a permissive OpenCode config, and rebuild the
  sandbox `AGENTS.md` from the kit's base plus every composed mixin's note.
- **`kit-published-node-dotnet/`, `kit-published-python/`**: same kits,
  `sandbox.image` pinned to the public Docker Hub tags. Release-please bumps
  versions + tags; keep them in sync with `kit-node-dotnet/` and
  `kit-python/`.
- **Mixins** (`mixins/<area>/`, `kind: mixin`): network rules, env vars,
  credentials, install steps, and an agent memory note
  (`files/home/.sbx-agents.d/<area>.md`) for one area each. Compose
  explicitly at launch (`--kit` flags or a `.sbxenv.yaml` `kits:` list).
- **`opencode-builtin-kit.spec.yaml`**: reference snapshot of Docker's
  built-in `opencode` kit (providers, MCP wiring) that our kit extends.
  Nothing in this repo uses it.

### Working on the repo

Work in a git worktree branched from `main`
([agent-guidance/worktrees.md](agent-guidance/worktrees.md)) — other agent
sessions share this checkout concurrently.

```bash
./scripts/bootstrap.sh     # or bootstrap.ps1: build + load the template,
                           # register the Zeldoc key, validate kit + mixins
sbx kit validate kit-node-dotnet/   # validate a kit or mixin after edits
./scripts/new-sandbox.sh   # wizard launcher for a dev sandbox
                           # (--source local picks up uncommitted edits)
```

Kit changes only apply to new sandboxes: `sbx rm <name>` + `sbx env run`
(or the wizard again).
Releases are cut by release-please from Conventional Commits on `main` —
see [agent-guidance/versioning.md](agent-guidance/versioning.md) and
[agent-guidance/commit-messages.md](agent-guidance/commit-messages.md).
Repo conventions: [AGENTS.md](AGENTS.md).
