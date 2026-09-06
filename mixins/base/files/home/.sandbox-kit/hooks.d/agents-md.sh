# AGENTS.md rebuild hook — shipped by the base mixin
# (mixins/base/files/home/.sandbox-kit/hooks.d/agents-md.sh) and sourced by
# the opencode-entrypoint runtime before the banner.
#
# The runtime's generated baseline can't be replaced from the kit grammar,
# so overwrite it before opencode starts with the baseline copy
# (~/.sandbox-agents.md, shipped by this mixin) plus every mixin note in
# ~/.sbx-agents.d/, keeping the runtime's Kits index section. No-op when
# the base mixin isn't composed (no ~/.sandbox-agents.md) or there is no
# workspace (WORKSPACE_DIR unset).
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
unset __agents __mine __kits __note
