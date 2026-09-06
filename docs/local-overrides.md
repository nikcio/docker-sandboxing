# Local overrides

The committed `.sbxenv.yaml` is the team's shared setup. Personal or
machine-specific tweaks — more memory, a debug log level, an extra mixin —
belong in a separate `local.sbxenv.yaml` that stays out of version control.
`sbx` merges both files: team defaults first, your overrides on top.

## Set it up

1. **Ignore the local file.** Add `local.sbxenv.yaml` to `.gitignore`:

   ```bash
   echo "local.sbxenv.yaml" >> .gitignore
   ```

2. **Add your overrides.** Only the fields you want to change or add —
   everything else comes from `.sbxenv.yaml`:

   ```yaml
   # local.sbxenv.yaml — not committed
   env:
     LOG_LEVEL: debug

   sandboxOptions:
     memory: 12g
   ```

3. **Run with both files**, team file first:

   ```bash
   sbx env run .sbxenv.yaml local.sbxenv.yaml
   ```

Teammates without a local file keep running plain `sbx env run` — both
commands resolve the same sandbox, because the sandbox name comes from the
first file.

## How merging works

- Nested mappings (`env:`, `sandboxOptions:`, …) merge key by key — your
  `memory:` replaces the team's, everything else stays.
- Lists (`kits:`, `ports:`, …) **concatenate** — a local `kits:` line adds a
  [mixin](mixins.md) on top of the team's list; it doesn't replace it.
- Relative paths (`workspace.path: .`, additional workspaces) resolve from
  the directory of the first file, so keep both files beside the project.

## Daily use

- Pass the same file pair to every command, including `sbx env rm`:

  ```bash
  sbx env rm .sbxenv.yaml local.sbxenv.yaml
  ```

- Re-running applies changed `env:` values to the next session. Changes to
  kits, ports, workspaces, or `sandboxOptions` only apply to **new**
  sandboxes — remove and run again (see [getting-started.md](getting-started.md)).

This repo uses the same pattern: `.sbxenv.yaml` is committed, and a
gitignored `local.sbxenv.yaml` holds machine-specific overrides.

Details: [Docker Sandboxes environment files](https://docs.docker.com/ai/sandboxes/configuration/environment-files/#combine-team-defaults-and-personal-settings).
