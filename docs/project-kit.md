# Project-specific config (in-project kit)

Two ways to customize the sandbox for your project — pick the lightest
one that fits:

1. **OpenCode settings only** → commit an `opencode.jsonc` in your repo
   root. No kit needed (see below).
2. **Network egress, env vars, files, agent notes** → add a small
   *in-project kit* to your repo and list it in `.sbxenv.yaml`.

## Project-level OpenCode config (no kit)

OpenCode merges the global config with a project-level config file at
the workspace root. Since `workspace.path` points at your repo, just
commit an `opencode.jsonc` (or `opencode.json`) in the project root —
it is picked up when opencode starts.

Do not write `~/.config/opencode/opencode.json` from a kit — the
sandbox owns that file (it is rewritten at startup for MCP wiring). Do
not set `OPENCODE_CONFIG` either — the `base` mixin owns it (it points
at the combined provider config that the entrypoint rebuilds from
`~/.config/opencode/providers.d/`). To contribute a provider, ship a
comment-free JSON fragment at
`files/home/.config/opencode/providers.d/30-<name>.json` in your kit —
the entrypoint merges it with the stock provider fragments (see
[mixins.md](mixins.md)).

## Add an in-project kit

An in-project kit is a mixin (`kind: mixin`) that lives in your repo,
next to `.sbxenv.yaml`. It adds exactly what your project needs on top
of the stock mixins.

1. Create `sandbox-kit/spec.yaml` in your project root (any directory
   name works — it is referenced explicitly):

   ```yaml
   # Project mixin: my-project specifics (feeds, env, agent notes).
   schemaVersion: "2"
   kind: mixin
   name: my-project
   displayName: My project
   description: >
     Project-specific network rules, env vars, and agent notes.
   version: 0.1.0

   permissions:
     network:
       allow:
         # Private npm registry
         - npm.mycompany.com:443
         # Internal NuGet feed
         - nuget.mycompany.com:443

   environment:
     variables:
       MY_FEATURE_FLAG: "1"
   ```

2. Optional: drop files into the sandbox home — `files/home/...` maps
   to `/home/agent/...`:

   ```text
   sandbox-kit/
   ├── spec.yaml
   └── files/
       └── home/
           └── .sbx-agents.d/
               └── my-project.md   # agent memory note
   ```

   A note in `.sbx-agents.d/` is appended to the sandbox `AGENTS.md`
   when the sandbox starts. Keep it short and directive — what the
   agent must know or do for this project (see `mixins/git` in the
   docker-sandboxing repo for an example). How `AGENTS.md` files are
   loaded: [agents-md.md](agents-md.md).

3. List it in your `.sbxenv.yaml` after the stock kits (local paths are
   relative to the env file):

   ```yaml
   kits:
      - git+https://github.com/nikcio/docker-sandboxing.git#dir=kit-published-node-dotnet&ref=v0.6.0
     - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/base&ref=v0.6.0
     # ...the other stock mixins your project keeps...
     - ./sandbox-kit
   ```

   Keep the stock agent kit (`kit-published`) and the `base` mixin — the
   kit defines the image and entrypoint wrapper, the `base` mixin ships
   the entrypoint runtime, OpenCode config, and agent guidance; your kit
   only adds capabilities.

4. Validate and recreate the sandbox (kit changes only apply to new
   sandboxes):

   ```bash
   sbx kit validate ./sandbox-kit
   sbx rm <sandbox-name>
   sbx env run
   ```

## Rules of thumb

- **Keep it yours.** Only project-specifics belong here; generic
  capabilities stay in the stock mixins (see [mixins.md](mixins.md)).
- **Deny-by-default network.** Only add hosts you need. If a download
  is blocked inside the sandbox, check `sbx policy log` and add the
  host to this spec (see [troubleshooting.md](troubleshooting.md)).
- **Commit it.** `sandbox-kit/` is part of the repo, so teammates get
  the same sandbox from the same `sbx env run`.
