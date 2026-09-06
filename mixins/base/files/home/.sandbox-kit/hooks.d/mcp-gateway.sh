# MCP gateway registration backstop — shipped by the base mixin
# (mixins/base/files/home/.sandbox-kit/hooks.d/mcp-gateway.sh) and sourced
# by the opencode-entrypoint runtime. The startup hook in this mixin's spec
# normally writes ~/.config/opencode/opencode.json before the entrypoint
# runs; if it didn't (older runtime, startup hooks from mixins
# unsupported), register the gateway here instead. No-op when MCP isn't
# enabled or the file already exists (opencode owns the file afterwards).
# sandboxd injects MCP_GATEWAY_URL + MCP_SENTINEL_TOKEN_NAME; the sentinel
# name is not a credential — the proxy substitutes the real token at
# request time.
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
