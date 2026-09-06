#!/usr/bin/env bash
# Shared OpenCode kit entrypoint runtime — shipped by the base mixin
# (mixins/base/files/home/.sandbox-kit/entrypoint.sh) and exec'd by every
# OpenCode kit's entrypoint wrapper. Do not edit the kit specs' wrappers:
# changes belong here.
#
# What it does, in order:
#   1. Sources the composed mixins' hooks from ~/.sandbox-kit/hooks.d/*.sh
#      (glob order) — the base mixin's AGENTS.md rebuild and MCP gateway
#      backstop, the env-guard mixin's .env guard, the banner mixin's
#      startup banner, and the global-opencode-config mixin's provider
#      config merge.
#   2. Auto-starts opencode.
#   3. When opencode exits, execs an interactive login shell (relaunch
#      opencode with `o` or `opencode`; quitting the shell ends the
#      session).

# 1. Mixin hooks. Composed mixins drop optional startup hooks here — the
# base mixin's AGENTS.md rebuild and MCP gateway backstop, the env-guard
# mixin's .env guard, the banner mixin's startup banner, and the
# global-opencode-config mixin's provider config merge live there.
# Sourced in glob order so hooks can read this script's state; a missing
# directory just means no hooks (e.g. when no hook mixin is composed).
for __hook in "$HOME"/.sandbox-kit/hooks.d/*.sh; do
  [ -f "$__hook" ] || continue
  # shellcheck disable=SC1090
  . "$__hook"
done
unset __hook

# 2. Launch opencode (the version baked into the template image).
if command -v opencode >/dev/null 2>&1; then
  opencode
  echo "opencode exited - back in the sandbox shell (relaunch: o or opencode)"
else
  echo "WARNING: opencode not found in image - dropping to shell" >&2
fi

# 3. Drop into the login shell.
exec bash -il
