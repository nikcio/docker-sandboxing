# Provider config merge hook — shipped by the global-opencode-config mixin
# (mixins/global-opencode-config/files/home/.sandbox-kit/hooks.d/provider-config.sh)
# and sourced by the base mixin's entrypoint runtime (in glob order, after
# the banner and env-guard hooks).
#
# Merges every provider mixin's fragment in ~/.config/opencode/providers.d/
# into the combined file OPENCODE_CONFIG points at (see
# ../merge-global-opencode-config.sh). This mixin's spec owns
# OPENCODE_CONFIG (provider mixins must not set it — that is what lets any
# number of provider mixins compose); export it defensively here too so
# `o`/`opencode` relaunched from the login shell keep working even if a
# runtime skipped mixin env injection. The merge script seeds a valid empty
# config when there are no fragments, so OPENCODE_CONFIG always points at a
# loadable file. Run via bash: the script carries `set -euo pipefail` and
# must not be sourced into this shell.
export OPENCODE_CONFIG="${OPENCODE_CONFIG:-$HOME/.config/opencode/providers.jsonc}"
if ! bash "$HOME/.sandbox-kit/merge-global-opencode-config.sh"; then
  echo "WARNING [provider-config]: merge failed - opencode may start without the expected model providers" >&2
fi
