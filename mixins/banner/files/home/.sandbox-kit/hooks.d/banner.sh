# Startup banner hook — shipped by the banner mixin
# (mixins/banner/files/home/.sandbox-kit/hooks.d/banner.sh) and sourced by
# the base mixin's entrypoint runtime before opencode starts (in glob
# order, before the env-guard mixin's guard report).
cat <<'BANNER'
------------------------------------------------------------
 Sandbox
------------------------------------------------------------
 opencode is starting automatically.
 - Quit opencode to drop into this shell
 - Relaunch opencode anytime with: o or opencode
------------------------------------------------------------
BANNER
