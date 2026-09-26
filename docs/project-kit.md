# Project-specific config (in-project kit)

Two ways to customize the sandbox for your project — pick the lightest one that fits:

1. **OpenCode settings only** → commit an `opencode.jsonc` in your repo root. No kit needed (see below) — this is also where the default model lives.
2. **Network egress, env vars, files, agent notes** → add a small *in-project kit* to your repo and list it in `sbxenv.yaml`.

## Project-level OpenCode config (no kit)

OpenCode merges the global config with a project-level config file at the workspace root. Since `workspace.path` points at your repo, just commit an `opencode.jsonc` (or `opencode.json`) in the project root — it is picked up when opencode starts.

The sandbox's provider mixins do not set a default model, so the project config owns it — point `model` at one of the composed providers:

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "model": "zeldoc/zdev-2"
}
```

Without a `model` key opencode falls back to its own default; pick a different one any time with `/models` in the TUI.

Do not write `~/.config/opencode/opencode.json` from a kit — the sandbox owns that file (it is rewritten at startup for MCP wiring). Do not set `OPENCODE_CONFIG` either — the `opencode` mixin owns it (it points at the combined provider config that the mixin builds from `~/.config/opencode/mixins.d/`). To contribute a provider, ship a comment-free JSON fragment in your kit's image — a file at `files/home/.config/opencode/mixins.d/30-<name>.json`, copied by your `<kit>.dockerfile` — and the opencode mixin merges it with the stock provider fragments (see [mixins.md](mixins.md)).

## Add an in-project kit

An in-project kit is a v3 mixin (`kind: mixin`) that lives in your repo, next to `sbxenv.yaml`. It adds exactly what your project needs on top of the stock mixins.

1. Create `sandbox-kit/sandbox-kit.yaml` in your project root (any directory name works — it is referenced explicitly):

   ```yaml
   # syntax=docker/sandbox-kit:3
   # Project mixin: my-project specifics (feeds, env, agent notes).
   schemaVersion: "3"
   kind: mixin
   displayName: My project
   description: >
     Project-specific network rules, env vars, and agent notes.
   version: "0.1.0"

   capabilities:
     - type: com.docker.sandbox/network-policy@1
       config:
         runtime:
           allow:
             # Private npm registry
             - npm.mycompany.com:443
             # Internal NuGet feed
             - nuget.mycompany.com:443
   ```

   Env vars for the agent go in a companion `sandbox-kit.dockerfile`:

   ```dockerfile
   # syntax=docker/dockerfile:1
   FROM scratch
   ENV MY_FEATURE_FLAG=1
   ```

2. Optional: drop files into the sandbox home and ship a note for the agent — a `contentFile` in an `agent-context` capability contributes to the AGENTS.md profile:

   ```text
   sandbox-kit/
   ├── sandbox-kit.yaml
   ├── sandbox-kit.dockerfile   # COPY + ENV lines
   ├── note.md                  # agent note (agent-context contentFile)
   └── files/
       └── home/
           └── .my-project/     # any files the project needs
   ```

   ```yaml
   # ...in sandbox-kit.yaml:
   capabilities:
     # ...the network policy above...
     - type: com.docker.sandbox/agent-context@1
       config:
         contentFile: ./note.md
   ```

   ```dockerfile
   # ...in sandbox-kit.dockerfile:
   COPY --chown=1000:1000 files/ /home/agent/.my-project/
   ```

The note lands in the agent's profile index (progressive disclosure); the files are just files. Keep the note short and directive — what the agent must know or do for this project. How AGENTS.md files are loaded: [agents-md.md](agents-md.md).

3. List it in your `sbxenv.yaml` after the stock mixins (local paths are relative to the env file; the workload is `agent:` above):

   ```yaml
   kits:
     - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/opencode&ref=vX.Y.Z
     - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/env-guard&ref=vX.Y.Z
     # ...the other stock mixins your project keeps...
     - ./sandbox-kit
   ```

Keep `opencode` (the agent + config + provider merge; pair it with `launch-opencode` to start OpenCode at startup — or compose any other agent's mixin instead). The list above also includes the optional `env-guard` (the no-.env policy). Your kit only adds capabilities.

4. Build and recreate the sandbox (kit changes only apply to new sandboxes):

   ```bash
   docker buildx build ./sandbox-kit --file ./sandbox-kit/sandbox-kit.yaml
   sbx rm <sandbox-name>
   sbx env run
   ```

The build is the validation: the v3 frontend decodes the descriptor strictly (any unknown field is an error) and checks the capabilities (e.g. every credential domain must be in the network allow list).

## Rules of thumb

- **Keep it yours.** Only project-specifics belong here; generic capabilities stay in the stock mixins (see [mixins.md](mixins.md)).
- **Deny-by-default network.** Only add hosts you need. If a download is blocked inside the sandbox, check `sbx policy log` and add the host to the network policy (see [troubleshooting.md](troubleshooting.md)).
- **Commit it.** `sandbox-kit/` is part of the repo, so teammates get the same sandbox from the same `sbx env run`.
