# .NET local tools hook — shipped by the dotnet-tools mixin
# (mixins/dotnet-tools/files/home/.sandbox-kit/hooks.d/dotnet-tools.sh)
# and sourced by the base mixin's entrypoint runtime before opencode
# starts.
#
# Runs `dotnet tool restore` for every .config/dotnet-tools.json manifest
# in the workspace so pinned local tools are on PATH before the agent
# starts. No-op when dotnet is missing, WORKSPACE_DIR is unset, or no
# manifest exists (no `exit` here — the hook is sourced inside the
# entrypoint and must never end it). Restore failures print a warning
# but never block startup.
if command -v dotnet >/dev/null 2>&1 && [ -n "${WORKSPACE_DIR:-}" ]; then
  __manifests="$(find "$WORKSPACE_DIR" -type f -name dotnet-tools.json -path '*/.config/*' -not -path '*/.git/*' 2>/dev/null)"
  if [ -n "$__manifests" ]; then
    printf '%s\n' "$__manifests" | while IFS= read -r __manifest; do
      __dir="$(dirname "$(dirname "$__manifest")")"
      echo "[dotnet-tools] restoring local tools ($__dir/.config/dotnet-tools.json)..."
      if (cd "$__dir" && dotnet tool restore); then
        echo "[dotnet-tools] local tools restored"
      else
        echo "WARNING [dotnet-tools]: dotnet tool restore failed in $__dir - continuing; re-run 'dotnet tool restore' there manually."
      fi
    done
  fi
  unset __manifests __manifest __dir
fi
