#!/usr/bin/env bash
# Shared OpenCode kit entrypoint runtime — shipped by the
# opencode-entrypoint mixin
# (mixins/opencode-entrypoint/files/home/.sandbox-kit/entrypoint.sh) and
# exec'd by every OpenCode kit's entrypoint wrapper. Do not edit the kit
# specs' wrappers: changes belong here.
#
# What it does, in order:
#   1. Guards the workspace against .env files (clone mode removes them
#      from the sandbox-local copy; direct mode refuses to start).
#   2. Sources the composed mixins' hooks from ~/.sandbox-kit/hooks.d/*.sh
#      (glob order) — e.g. the base mixin's AGENTS.md rebuild and MCP
#      gateway backstop.
#   3. Prints the startup banner and reports the env-guard result.
#   4. Auto-starts opencode.
#   5. When opencode exits, execs an interactive login shell (relaunch
#      opencode with `o` or `opencode`; quitting the shell ends the
#      session).

# 1. Env guard, scan phase (reported under the banner below). The
# workspace must never contain a .env file. Clone mode removes
# offenders from the sandbox-local copy; direct mode (host tree
# mounted in place) refuses to start instead of deleting host files.
__envs=""
__clean="workspace clean (no .env files)"
__guard_fatal=0
if [ -n "${WORKSPACE_DIR:-}" ]; then
  __envs="$(find "$WORKSPACE_DIR" -type f -name .env -not -path '*/.git/*' 2>/dev/null)"
else
  __clean="workspace not checked (WORKSPACE_DIR is not set)"
fi
if [ -d /run/sandbox/source ]; then
  if [ -n "$__envs" ]; then
    printf '%s\n' "$__envs" | while IFS= read -r __f; do rm -f -- "$__f"; done
  fi
elif [ -n "$__envs" ]; then
  __guard_fatal=1
fi

# 2. Mixin hooks. Composed mixins drop optional startup hooks here — the
# base mixin's AGENTS.md rebuild and MCP gateway backstop live there.
# Sourced in glob order so hooks can read this script's state; a missing
# directory just means no hooks (e.g. when the base mixin isn't composed).
for __hook in "$HOME"/.sandbox-kit/hooks.d/*.sh; do
  [ -f "$__hook" ] || continue
  # shellcheck disable=SC1090
  . "$__hook"
done
unset __hook

# 3. Banner.
cat <<'BANNER'
------------------------------------------------------------
 Sandbox
------------------------------------------------------------
 opencode is starting automatically.
 - Quit opencode to drop into this shell
 - Relaunch opencode anytime with: o or opencode
------------------------------------------------------------
BANNER

# Env guard report: WARNING + 5s pause when files were deleted
# (clone mode), fatal ERROR when the session is refused (direct).
if [ "$__guard_fatal" = "1" ]; then
  echo "============================================================"
  echo "ERROR [env-guard]: .env file(s) found in the workspace (direct mode):"
  printf '%s\n' "$__envs" | sed 's/^/  - /'
  echo "This sandbox refuses to start with .env files in the workspace."
  echo "Delete them on the host (they are mounted into the sandbox), then recreate the sandbox."
  echo "============================================================"
  exit 1
fi
if [ -d /run/sandbox/source ]; then
  if [ -n "$__envs" ]; then
    echo "============================================================"
    echo "WARNING [env-guard]: .env file(s) were removed from the"
    echo "workspace (clone mode - sandbox-local copy):"
    printf '%s\n' "$__envs" | sed 's/^/  - /'
    echo "============================================================"
    echo "[env-guard] opencode opens in 5 seconds..."
    sleep 5
  fi
else
  echo "[env-guard] direct mode - $__clean"
fi

# 4. Launch opencode (the version baked into the template image).
if command -v opencode >/dev/null 2>&1; then
  opencode
  echo "opencode exited - back in the sandbox shell (relaunch: o or opencode)"
else
  echo "WARNING: opencode not found in image - dropping to shell" >&2
fi

# 5. Drop into the login shell.
exec bash -il
