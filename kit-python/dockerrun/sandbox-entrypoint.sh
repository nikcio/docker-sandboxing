#!/bin/bash
# Sandbox entrypoint shim.
#
# v3 lifecycle startup hooks run alongside the agent — there is no
# before-agent ordering guarantee. The workloads ship this shim as the
# image ENTRYPOINT so the checks that must gate the agent run to
# completion first; every stage is best-effort except the guard, and the
# real agent is exec'd with the original arguments at the end.
#
# Agent guidance (AGENTS.md profile + per-mixin notes) and anything that
# tolerates running alongside the agent stay on lifecycle startup hooks
# or on the runtime's own agent-context composition.
set -u

log() { echo "[sandbox-entrypoint] $*"; }

# 1. Workspace .env guard (fail-closed; the guard exits nonzero itself).
if [ -x /opt/sandbox/bin/env-guard.sh ]; then
  if ! /opt/sandbox/bin/env-guard.sh; then
    log "env-guard failed — refusing to start the agent"
    exit 1
  fi
fi

# 2. Merge OpenCode provider config fragments (best-effort; the agent
#    reads the merged file when it loads its config).
if [ -x /opt/sandbox/bin/merge-opencode-config.sh ]; then
  /opt/sandbox/bin/merge-opencode-config.sh || log "opencode config merge failed (continuing)"
fi

log "handing over to the agent"
exec "$@"
