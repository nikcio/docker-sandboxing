#!/usr/bin/env bash
# Configurable launcher (alias target) for creating/attaching an
# opencode-node-dotnet Docker Sandboxes sandbox.
#
# Composes the sandbox kit with the mixins for a chosen profile and runs
# `sbx run`. If a sandbox with the resolved name already exists, it
# re-attaches without kit flags (kits only apply at creation).
#
# Configurable via arguments or environment variables (args win):
#
#   SBX_SANDBOX_PROFILE   full (default) | node | dotnet | node-docker | none
#   SBX_SANDBOX_MIXINS    comma-separated mixin override, e.g. zeldoc,git,node
#   SBX_SANDBOX_SOURCE    git (default; fetch from GitHub) | local (clone)
#   SBX_SANDBOX_REPO      GitHub repo (default: nikcio/docker-sandboxing)
#   SBX_SANDBOX_REF       pin git kits to a branch/tag/commit (optional)
#   SBX_SANDBOX_REPO_DIR  local repo root for --source local (default: repo root)
#
# Usage:
#   sbx-new [WORKSPACE] [--name NAME] [--profile PROFILE] [--mixins m1,m2]
#           [--source git|local] [--repo OWNER/REPO] [--ref REF]
#           [--repo-dir DIR] [--detach] [--list-profiles]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PROFILES_full="opencode-runtime zeldoc git node dotnet docker apt"
PROFILES_node="opencode-runtime zeldoc git node"
PROFILES_dotnet="opencode-runtime zeldoc git dotnet"
PROFILES_node_docker="opencode-runtime zeldoc git node docker"
PROFILES_none=""

WORKSPACE="."
NAME=""
PROFILE="${SBX_SANDBOX_PROFILE:-full}"
MIXINS="${SBX_SANDBOX_MIXINS:-}"
SOURCE="${SBX_SANDBOX_SOURCE:-git}"
REPO="${SBX_SANDBOX_REPO:-nikcio/docker-sandboxing}"
REF="${SBX_SANDBOX_REF:-}"
REPO_DIR="${SBX_SANDBOX_REPO_DIR:-$SCRIPT_DIR/..}"
DETACH=""

while [ $# -gt 0 ]; do
    case "$1" in
        --name) NAME="$2"; shift 2 ;;
        --profile) PROFILE="$2"; shift 2 ;;
        --mixins) MIXINS="$2"; shift 2 ;;
        --source) SOURCE="$2"; shift 2 ;;
        --repo) REPO="$2"; shift 2 ;;
        --ref) REF="$2"; shift 2 ;;
        --repo-dir) REPO_DIR="$2"; shift 2 ;;
        --detach) DETACH=1; shift ;;
        --list-profiles)
            for p in full node dotnet node-docker none; do
                list="$(eval "echo \${PROFILES_${p//-/_}}" | tr ' ' ',')"
                printf "%-12s %s\n" "$p" "$list"
            done
            exit 0
            ;;
        -h|--help)
            sed -n '2,26p' "${BASH_SOURCE[0]}"; exit 0 ;;
        *) WORKSPACE="$1"; shift ;;
    esac
done

# Resolve the profile's mixin list (PROFILES_none is legitimately empty).
var="PROFILES_${PROFILE//-/_}"
if ! eval "[ \"\${${var}+x}\" = x ]"; then
    echo "Unknown profile '$PROFILE'. Known: full, node, dotnet, node-docker, none (or pass --mixins)." >&2
    exit 1
fi
PROFILE_MIXINS="$(eval "echo \"\${${var}}\"")"

if [ -z "$MIXINS" ]; then
    MIXINS="$PROFILE_MIXINS"
fi

# Resolve sandbox name (sbx default naming: <agent>-<workspace basename>).
if [ -z "$NAME" ]; then
    ws="$(cd "$WORKSPACE" && pwd)"
    NAME="opencode-node-dotnet-$(basename "$ws")"
fi

# Re-attach when the sandbox already exists (kits only apply at creation).
if sbx ls -q 2>/dev/null | grep -qxF "$NAME"; then
    echo "==> Attaching to existing sandbox '$NAME'"
    exec sbx run --name "$NAME"
fi

# Build the kit references.
KITS=()
if [ "$SOURCE" = "git" ]; then
    git_base="git+https://github.com/${REPO}.git"
    if [ -n "$REF" ]; then
        KITS+=("${git_base}#ref=${REF}&dir=kit")
    else
        KITS+=("${git_base}#dir=kit")
    fi
    if [ -n "$MIXINS" ]; then
        for m in $(echo "$MIXINS" | tr ',' ' '); do
            if [ -n "$REF" ]; then
                KITS+=("${git_base}#ref=${REF}&dir=mixins/${m}")
            else
                KITS+=("${git_base}#dir=mixins/${m}")
            fi
        done
    fi
else
    REPO_DIR="$(cd "$REPO_DIR" && pwd)"
    KITS+=("$REPO_DIR/kit")
    if [ -n "$MIXINS" ]; then
        for m in $(echo "$MIXINS" | tr ',' ' '); do
            KITS+=("$REPO_DIR/mixins/$m")
        done
    fi
fi

KIT_ARGS=()
for k in "${KITS[@]}"; do KIT_ARGS+=(--kit "$k"); done

echo "==> Creating sandbox '$NAME' (profile: $PROFILE, source: $SOURCE)"
echo "    kits: ${KITS[*]}"

if [ -n "$DETACH" ]; then
    sbx create -q --name "$NAME" "${KIT_ARGS[@]}" opencode-node-dotnet "$WORKSPACE"
else
    sbx run --name "$NAME" "${KIT_ARGS[@]}" opencode-node-dotnet "$WORKSPACE"
fi
