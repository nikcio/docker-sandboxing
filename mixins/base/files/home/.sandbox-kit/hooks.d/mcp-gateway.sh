# MCP gateway registration backstop — shipped by the base mixin
# (mixins/base/files/home/.sandbox-kit/hooks.d/mcp-gateway.sh) and sourced
# by the base mixin's entrypoint runtime. The startup hook in this mixin's spec
# normally writes ~/.config/opencode/opencode.json before the entrypoint
# runs; if it didn't (older runtime, startup hooks from mixins
# unsupported), register the gateway here instead. No-op when MCP isn't
# enabled or the file already exists (opencode owns the file afterwards).
# sandboxd injects MCP_GATEWAY_URL + MCP_SENTINEL_TOKEN_NAME; the sentinel
# name is not a credential — the proxy substitutes the real token at
# request time.
#
# The values are interpolated through jq (jq -n --arg), never a heredoc:
# raw interpolation would let a value containing quotes/backslashes
# rewrite the config shape (JSON injection — e.g. point the gateway at an
# attacker URL or add an extra mcp entry). Template images ship jq; with
# jq missing the backstop refuses to write instead of writing an
# interpolatable config.
if [ -n "${MCP_GATEWAY_URL:-}" ] && [ ! -f "$HOME/.config/opencode/opencode.json" ]; then
  if command -v jq >/dev/null 2>&1; then
    mkdir -p "$HOME/.config/opencode"
    jq -n --arg url "$MCP_GATEWAY_URL" --arg token "${MCP_SENTINEL_TOKEN_NAME:-}" '{
      "$schema": "https://opencode.ai/config.json",
      "mcp": {
        "mcp-gateway": {
          "type": "remote",
          "url": $url,
          "enabled": true,
          "headers": {
            "Authorization": ("Bearer " + $token)
          }
        }
      }
    }' > "$HOME/.config/opencode/opencode.json"
  else
    echo "WARNING [mcp-gateway]: jq not found — skipping MCP gateway registration (would-be-injected config not written)" >&2
  fi
fi
