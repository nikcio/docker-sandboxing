# Project-specific config (in-project kit)

Two ways to customize the sandbox for your project. Pick the lightest one that fits:

1. **OpenCode settings only.** Commit an `opencode.jsonc` in your repo root. No kit needed (see below), and this is also where the default model lives.
2. **Network egress, env vars, files, agent notes.** Add a small *in-project kit* to your repo and list it in `sbxenv.yaml`.

## Project-level OpenCode config (no kit)

OpenCode merges the global config with a project-level config file at the workspace root. Since `workspace.path` points at your repo, just commit an `opencode.jsonc` (or `opencode.json`) in the project root. It is picked up when opencode starts.

The sandbox's provider mixins do not set a default model, so the project config owns it. Point `model` at one of the composed providers:

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "model": "zeldoc/zdev"
}
```

Without a `model` key opencode falls back to its own default; pick a different one any time with `/models` in the TUI.

Do not write `~/.config/opencode/opencode.json` from a kit; the sandbox owns that file (it is rewritten at startup for MCP wiring, and the provider mixins merge their config into it). Do not set `OPENCODE_CONFIG` either; the `opencode` mixin owns it (it points at the combined config that the mixin rebuilds from `~/.config/opencode/mixins.d/`). To contribute extra config from your kit, ship a comment-free JSON fragment at `/home/agent/.config/opencode/mixins.d/30-<name>.json` (a `files:` entry under the lifecycle capability). The opencode mixin merges it at every start (see [mixins.md](mixins.md)). Model providers are simpler: add the `opencode-zeldoc`/`opencode-copilot` mixins instead of a fragment.

## Add an in-project kit

An in-project kit is a mixin (`kind: mixin`) that lives in your repo, next to `sbxenv.yaml`. It adds exactly what your project needs on top of the stock mixins.

1. Create `sandbox-kit/my-project.yaml` in your project root (any directory name works, it is referenced explicitly):

   ```yaml
   # syntax=docker/sandbox-kit:3
   # Project mixin: my-project specifics (feeds, env, agent notes).
   schemaVersion: "3"
   kind: mixin
   displayName: My project
   description: >
     Project-specific network rules, env vars, and agent notes.
   version: 0.1.0

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

2. Optional: set environment variables with a small dockerfile. Add `dockerfile: my-project.dockerfile` to the descriptor and create the file:

   ```dockerfile
   ENV MY_FEATURE_FLAG=1
   ```

3. Optional: drop files into the sandbox home with a lifecycle capability:

   ```yaml
   capabilities:
     # ...the network-policy capability above...
     - type: com.docker.sandbox/lifecycle@1
       config:
         files:
           - path: /home/agent/.config/my-tool/config.json
             mode: "0644"
             overwrite: false
             description: Config the sandboxed tool needs
             content: |
               { "endpoint": "https://mycompany.com" }
   ```

4. List it in your `sbxenv.yaml` after the stock kits (local paths are relative to the env file):

    ```yaml
    agent: opencode

    kits:
      - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/opencode&ref=<copy the pin from the examples>
      - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins-v3/env-guard&ref=<same ref>
      # ...the other stock mixins your project keeps...
      - ./sandbox-kit
    ```

   The `opencode` mixin ships the permissive base config and the OpenCode CLI. Keep it unless you have your own opencode setup (model provider mixins work without it; they manage their own config). The list above also includes the optional `env-guard` (the no-.env policy). Your kit only adds capabilities.

5. Validate and recreate the sandbox (kit changes only apply to new sandboxes):

   ```bash
   docker buildx build -f sandbox-kit/my-project.yaml -t sandbox-kit:check sandbox-kit/   # validates the descriptor
   sbx rm <sandbox-name>
   sbx env run
   ```

## Rules of thumb

- **Keep it yours.** Only project-specifics belong here; generic capabilities stay in the stock mixins (see [mixins.md](mixins.md)).
- **Deny-by-default network.** Only add hosts you need. If a download is blocked inside the sandbox, check `sbx policy log` and add the host to this spec (see [troubleshooting.md](troubleshooting.md)).
- **Commit it.** `sandbox-kit/` is part of the repo, so teammates get the same sandbox from the same `sbx env run`.
