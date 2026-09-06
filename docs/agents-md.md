# Agent memory (AGENTS.md)

OpenCode reads `AGENTS.md` files to customize the agent's behavior. With
these sandboxes, three such files are in play — this page explains what
each one is, which one the agent actually loads, and where to put your
own rules.

## The three files

| File | Lives | Written by |
| ---- | ----- | ---------- |
| Project rules | `AGENTS.md` in your repo root | You — commit it with the repo |
| Sandbox environment file | Next to the workspace folder, outside your repo | Generated at sandbox start: kit base + one note per mixin |
| Global rules | `~/.config/opencode/AGENTS.md` on your host | You — personal, not committed |

## Which one the agent loads

opencode starts in the workspace and looks for an `AGENTS.md` there, then
in each parent directory — **the first match wins** (with `CLAUDE.md` as
the fallback name), and files above the match are never read:

- **Your repo has its own `AGENTS.md`** → that is the file the agent
  loads.
- **Your repo has none** → the sandbox environment file (the generated
  one next to the workspace folder) is what loads.
- The global file is combined on top — for host sessions. Your host's
  global file is not copied into sandboxes (see
  [agent-skills.md](agent-skills.md) for the skills equivalent, which
  is).

If `AGENTS.md` and `CLAUDE.md` sit at the same level, only the
`AGENTS.md` is used.

## What the sandbox environment file contains

The kit entrypoint rebuilds it before opencode starts, from:

- the kit base (`files/home/.sandbox-agents.md` in the agent kit) —
  environment facts plus an index of guidance files (network blocks, git
  auth, workspace mode, persistent shell);
- one note per composed mixin (`mixins/<area>/files/home/.sbx-agents.d/<area>.md`
  in the docker-sandboxing repo);
- the runtime's Kits index section, preserved across rebuilds.

Because it is generated:

- it sits **outside your repo** — it never shows up in `git status`;
- it is **rebuilt at every sandbox start** — hand-edits are lost.

## Where to put your rules

1. **Rules for working on your project** (build commands, conventions,
   gotchas): commit an `AGENTS.md` in the repo root. It arrives with the
   workspace and wins over the generated file.

2. **Personal, cross-project rules** (tone, review habits): put them in
   `~/.config/opencode/AGENTS.md` on your host. Nothing copies this file
   into sandboxes — global rules apply to host sessions only.

3. **Project-specific sandbox notes** (private feeds, egress, agent
   notes): add a note to your in-project kit — see
   [project-kit.md](project-kit.md).

## Caveats

- **Repo file shadows the generated one.** First match wins: when your
  repo has its own `AGENTS.md`, the agent never reads the sandbox
  environment file. If the agent seems unaware of sandbox facts (HTTP 403
  block shapes, proxy-injected git auth), point it at the generated file
  next to the workspace folder — or copy the lines you always want into
  your repo's `AGENTS.md`.
- **Parent folders don't cascade.** Rules in an `AGENTS.md` above the
  repo (or above the workspace) are skipped — only the first match
  loads.
- **Mixin notes need a recreate.** Kit changes only apply to new
  sandboxes — `sbx rm <sandbox-name>` then `sbx env run` (see
  [mixins.md](mixins.md)).

Details: [Rules — opencode docs](https://opencode.ai/docs/rules/).
