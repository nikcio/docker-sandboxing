# Agent skills

Your host has global agent skills — `SKILL.md` folders from Claude Code,
Cursor, Copilot, and friends. The `sbx` CLI can copy them into a persistent
store that sandboxes share, so the sandboxed agent loads the same skills you
use outside it.

## Import your skills

1. **Preview** what `sbx` finds on your host:

   ```bash
   sbx skills import --dry-run
   ```

2. **Import** them into the shared store:

   ```bash
   sbx skills import
   ```

   Re-run the command when a skill changed on the host — importing replaces
   the stored copy of that skill. Add `--force` to replace existing skills
   without the confirmation prompt.

The command scans these directories (when two contain a skill with the same
name, the first source wins):

| Host source | What usually lives there |
| ----------- | ------------------------ |
| `~/.agents/skills` | agent-neutral skills |
| `~/.claude/skills` | Claude Code skills |
| `~/.copilot/skills` | GitHub Copilot skills |
| `~/.cursor/skills` | Cursor skills |
| `~/.factory/skills` | Droid (Factory) skills |

## What the sandbox sees

- Sandboxes mount the shared store read-write by default, and the mount
  refreshes on every start — so you can import before or after creating the
  sandbox.
- OpenCode loads global skills from `~/.config/opencode/skills/`,
  `~/.claude/skills/`, and `~/.agents/skills/` — skills imported from the
  Claude-compatible and agent-compatible sources are the ones it picks up.
- Skills load per session: after importing, quit opencode and relaunch it
  with `o` to pick up new skills.
- Project skills need no import: an `.opencode/skills/<name>/SKILL.md`
  folder in your repo arrives with the workspace, and the global config
  layer (from the `global-opencode-config` mixin) already allows the
  `skill` tool.

Quick check from your project root that the store is mounted:

```bash
sbx env exec -- ls /home/agent/.claude/skills /home/agent/.agents/skills
```

## Caveats

- Shared agent skills are an experimental `sbx` feature, and Docker's guide
  lists Claude Code, Codex, Copilot, Cursor, and Droid as the supported
  agents. If the check above shows nothing, the store isn't mounted in your
  sandbox — fall back to project skills in `.opencode/skills/`.
- Every sandbox sharing the store shares one trust boundary: a sandbox can
  modify a skill for all the others. Opt out per sandbox with
  `--no-share-skills` at creation.
- `sbx reset` clears the shared store.

Details: [Share agent skills](https://docs.docker.com/ai/sandboxes/workflows/agent-skills/).
