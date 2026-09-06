# Workspace .env guard hook — shipped by the env-guard mixin
# (mixins/env-guard/files/home/.sandbox-kit/hooks.d/env-guard.sh) and
# sourced by the base mixin's entrypoint runtime before opencode starts
# (after the banner hook, in glob order).
#
# The workspace must never contain a .env file. Clone mode removes
# offenders from the sandbox-local copy; direct mode (host tree mounted
# in place) refuses to start instead of deleting host files. No-op when
# there is no workspace (WORKSPACE_DIR unset). The fatal branch exits the
# entrypoint — the sandbox refuses to start.
__envs=""
__clean="workspace clean (no .env files)"
if [ -n "${WORKSPACE_DIR:-}" ]; then
  __envs="$(find "$WORKSPACE_DIR" -type f -name .env -not -path '*/.git/*' 2>/dev/null)"
else
  __clean="workspace not checked (WORKSPACE_DIR is not set)"
fi
if [ -d /run/sandbox/source ]; then
  if [ -n "$__envs" ]; then
    printf '%s\n' "$__envs" | while IFS= read -r __f; do rm -f -- "$__f"; done
    echo "============================================================"
    echo "WARNING [env-guard]: .env file(s) were removed from the"
    echo "workspace (clone mode - sandbox-local copy):"
    printf '%s\n' "$__envs" | sed 's/^/  - /'
    echo "============================================================"
    echo "[env-guard] opencode opens in 5 seconds..."
    sleep 5
  fi
elif [ -n "$__envs" ]; then
  echo "============================================================"
  echo "ERROR [env-guard]: .env file(s) found in the workspace (direct mode):"
  printf '%s\n' "$__envs" | sed 's/^/  - /'
  echo "This sandbox refuses to start with .env files in the workspace."
  echo "Delete them on the host (they are mounted into the sandbox), then recreate the sandbox."
  echo "============================================================"
  exit 1
else
  echo "[env-guard] direct mode - $__clean"
fi
unset __envs __clean __f
