#!/usr/bin/env bash
# Shared OpenCode kit entrypoint runtime — shipped by the base mixin
# (mixins/base/files/home/.sandbox-kit/entrypoint.sh) and exec'd by every
# OpenCode kit's entrypoint wrapper. Do not edit the kit specs' wrappers:
# changes belong here.
#
# What it does, in order:
#   1. Guards the workspace against .env files (clone mode removes them
#      from the sandbox-local copy; direct mode refuses to start).
#   2. Rebuilds the sandbox AGENTS.md from ~/.sandbox-agents.md (this
#      mixin) plus every composed mixin's note in ~/.sbx-agents.d/,
#      keeping the runtime's Kits index section.
#   3. Rebuilds the combined OpenCode provider config: merges every
#      provider mixin's fragment in ~/.config/opencode/providers.d/
#      into the file OPENCODE_CONFIG points at (see
#      merge-opencode-config.sh).
#   4. Prints the startup banner and reports the env-guard result.
#   5. Registers the sandbox MCP gateway when the runtime's startup hook
#      didn't (backstop — see the comment at that block).
#   6. Auto-starts opencode.
#   7. When opencode exits, execs an interactive login shell (relaunch
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

# 2. Rebuild the sandbox AGENTS.md: the runtime's baseline can't be
# replaced from the kit grammar, so overwrite it before opencode
# starts with the baseline copy (~/.sandbox-agents.md, shipped by this
# mixin) plus every mixin note in ~/.sbx-agents.d/, keeping the
# runtime's Kits index section.
__agents="$(dirname "${WORKSPACE_DIR:-}")/AGENTS.md"
__mine="$HOME/.sandbox-agents.md"
if [ -n "${WORKSPACE_DIR:-}" ] && [ -f "$__agents" ] && [ -f "$__mine" ]; then
  __kits="$(sed -n '/<!-- sbx:kits-section start -->/,/<!-- sbx:kits-section end -->/p' "$__agents")"
  {
    cat "$__mine"
    for __note in "$HOME"/.sbx-agents.d/*.md; do
      [ -f "$__note" ] || continue
      printf '\n'
      cat "$__note"
    done
    if [ -n "$__kits" ]; then printf '\n%s\n' "$__kits"; fi
  } > "$__agents"
fi

# 3. Rebuild the combined OpenCode provider config: every provider mixin
#    (zeldoc, copilot, ...) ships a fragment to
#    ~/.config/opencode/providers.d/ and this merge produces the single
#    file OPENCODE_CONFIG points at. The base mixin's spec owns
#    OPENCODE_CONFIG (provider mixins must not set it — that is what
#    lets any number of them compose); export it defensively here too so
#    `o`/`opencode` relaunched from the login shell keep working even if
#    a runtime skipped mixin env injection.
export OPENCODE_CONFIG="${OPENCODE_CONFIG:-$HOME/.config/opencode/providers.jsonc}"
if [ -x "$HOME/.sandbox-kit/merge-opencode-config.sh" ]; then
  if ! "$HOME/.sandbox-kit/merge-opencode-config.sh"; then
    echo "WARNING [provider-config]: merge failed - opencode may start without the expected model providers" >&2
  fi
else
  echo "WARNING [provider-config]: $HOME/.sandbox-kit/merge-opencode-config.sh is missing (update the base mixin)" >&2
fi

# 4. Banner.
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

# 5. MCP gateway registration backstop. The startup hook in this mixin's
# spec normally writes ~/.config/opencode/opencode.json before this
# script runs. If the hook didn't run (older runtime, startup hooks from
# mixins unsupported), register the gateway here instead. No-op when MCP
# isn't enabled or the file already exists (opencode owns the file
# afterwards). sandboxd injects MCP_GATEWAY_URL + MCP_SENTINEL_TOKEN_NAME;
# the sentinel name is not a credential — the proxy substitutes the real
# token at request time.
if [ -n "${MCP_GATEWAY_URL:-}" ] && [ ! -f "$HOME/.config/opencode/opencode.json" ]; then
  mkdir -p "$HOME/.config/opencode"
  cat > "$HOME/.config/opencode/opencode.json" <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
  "mcp": {
    "mcp-gateway": {
      "type": "remote",
      "url": "$MCP_GATEWAY_URL",
      "enabled": true,
      "headers": {
        "Authorization": "Bearer $MCP_SENTINEL_TOKEN_NAME"
      }
    }
  }
}
EOF
fi

# 6. Launch opencode (the version baked into the template image).
if command -v opencode >/dev/null 2>&1; then
  opencode
  echo "opencode exited - back in the sandbox shell (relaunch: o or opencode)"
else
  echo "WARNING: opencode not found in image - dropping to shell" >&2
fi

# 7. Drop into the login shell.
exec bash -il
