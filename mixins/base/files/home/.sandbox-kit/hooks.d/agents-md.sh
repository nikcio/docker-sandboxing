# AGENTS.md rebuild hook — shipped by the base mixin
# (mixins/base/files/home/.sandbox-kit/hooks.d/agents-md.sh) and sourced by
# the base mixin's entrypoint runtime (in glob order, before the banner
# and env-guard hooks).
#
# The runtime's generated baseline can't be replaced from the kit grammar,
# so overwrite it before opencode starts with the baseline copy
# (~/.sandbox-agents.md, shipped by this mixin) plus every mixin note in
# ~/.sbx-agents.d/, keeping the runtime's Kits index section. No-op when
# the base mixin isn't composed (no ~/.sandbox-agents.md) or there is no
# workspace (WORKSPACE_DIR unset).
#
# Kits-section integrity: the runtime writes the Kits index once at
# sandbox creation as the last <!-- sbx:kits-section --> pair in this
# file (this hook always appends it last). Rebuilds scrape that LAST
# complete pair — never the first — so a foreign pair earlier in the
# file cannot shadow the real index and persist across restarts.
# Composed content can never introduce a pair of its own: the base copy
# is stripped of marker regions, and a mixin note containing any
# sbx:kits-section marker is skipped whole (a note claiming to be the
# kits index is tampering, not content).
__agents="$(dirname "${WORKSPACE_DIR:-}")/AGENTS.md"
__mine="$HOME/.sandbox-agents.md"
if [ -n "${WORKSPACE_DIR:-}" ] && [ -f "$__agents" ] && [ -f "$__mine" ]; then
  __start_line="$(grep -n '<!-- sbx:kits-section start -->' "$__agents" | tail -n 1 | cut -d: -f1)"
  __end_line="$(grep -n '<!-- sbx:kits-section end -->' "$__agents" | tail -n 1 | cut -d: -f1)"
  __kits=""
  if [ -n "$__start_line" ] && [ -n "$__end_line" ] && [ "$__end_line" -gt "$__start_line" ] 2>/dev/null; then
    __kits="$(sed -n "${__start_line},${__end_line}p" "$__agents")"
  fi
  {
    sed '/<!-- sbx:kits-section start -->/,/<!-- sbx:kits-section end -->/d' "$__mine"
    for __note in "$HOME"/.sbx-agents.d/*.md; do
      [ -f "$__note" ] || continue
      if grep -q 'sbx:kits-section' "$__note"; then
        echo "WARNING [agents-md]: skipping ${__note} — it contains sbx:kits-section markers (kits-sections are runtime-owned)" >&2
        continue
      fi
      printf '\n'
      cat "$__note"
    done
    if [ -n "$__kits" ]; then printf '\n%s\n' "$__kits"; fi
  } > "$__agents"
fi
unset __agents __mine __kits __note __start_line __end_line
