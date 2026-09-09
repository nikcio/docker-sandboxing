#!/usr/bin/env bash
# Wizard-style launcher for creating/attaching an opencode-node-dotnet
# Docker Sandboxes sandbox.
#
# Run with no arguments for a guided wizard: it asks for the workspace,
# mixin profile, kit source, sandbox name, and launch mode, shows a
# summary, and creates/attaches the sandbox.
#
# Any argument/flag skips the wizard (scripted mode) — unset values fall
# back to environment variables, then defaults:
#
#   SBX_SANDBOX_PROFILE   full (default) | node | dotnet | node-docker | browser | none
#   SBX_SANDBOX_MIXINS    comma-separated mixin override, e.g. zeldoc,git,node
#   SBX_SANDBOX_SOURCE    git (default; fetch from GitHub) | local (clone)
#   SBX_SANDBOX_REPO      GitHub repo (default: nikcio/docker-sandboxing)
#   SBX_SANDBOX_REF       pin git kits to a branch/tag/commit (optional)
#   SBX_SANDBOX_REPO_DIR  local repo root for --source local (default: repo root)
#
# If a sandbox with the resolved name already exists, it re-attaches
# without kit flags (kits only apply at creation).
#
# Usage:
#   ./scripts/new-sandbox.sh                            # guided wizard
#   ./scripts/new-sandbox.sh [WORKSPACE] [--name NAME] [--profile PROFILE]
#           [--mixins m1,m2] [--source git|local] [--repo OWNER/REPO]
#           [--ref REF] [--repo-dir DIR] [--detach] [--yes] [--list-profiles]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Canonical mixins ("area|description").
ALL_MIXINS=(
    "base|entrypoint runtime (hooks, opencode autostart, login shell) + agent guidance + MCP gateway (required)"
    "global-opencode-config|permissive OpenCode config + provider config merge (required with model providers)"
    "env-guard|workspace .env guard: refuses/removes .env files"
    "banner|startup banner (cosmetic)"
    "opencode-runtime|OpenCode runtime egress: updates, models.dev, Zen, plugins"
    "zeldoc|Zeldoc.ai (zdev) model provider: proxy-managed key + network"
    "copilot|GitHub Copilot model provider: device-flow sign-in + network"
    "git|git hosting (GitHub/GitLab) + gh auth + worktree workflow"
    "node|Node.js/NVM/PNPM: nodejs.org + npm registry"
    "dotnet|.NET/NuGet + Microsoft hosts, telemetry off"
    "docker|container registries for the in-sandbox Docker engine"
    "apt|Ubuntu/Microsoft package mirrors for apt (+ background cache warm)"
    "browser|Google Chrome browser software (no network rules)"
    "playwright|Playwright + Chromium headless shell (lightest)"
    "playwright-chromium|Playwright + full Chromium"
    "playwright-all|Playwright + Chromium, Firefox, WebKit"
    "sbx|sbx CLI inside the sandbox: kit authoring (validate/inspect/pack)"
)

PROFILES_full="base global-opencode-config env-guard banner opencode-runtime zeldoc git node dotnet docker apt browser playwright"
PROFILES_node="base global-opencode-config env-guard banner opencode-runtime zeldoc git node"
PROFILES_dotnet="base global-opencode-config env-guard banner opencode-runtime zeldoc git dotnet"
PROFILES_node_docker="base global-opencode-config env-guard banner opencode-runtime zeldoc git node docker"
PROFILES_browser="base global-opencode-config env-guard banner opencode-runtime zeldoc git node apt browser playwright"
PROFILES_none=""

# Profile lookup — a static allowlist, never an eval of a constructed
# variable name: PROFILE/--profile and SBX_SANDBOX_PROFILE are untrusted
# input, and "PROFILES_${x}" + eval turned any value containing shell
# syntax into command injection. Prints the mixin list; returns 1 for an
# unknown profile.
profile_mixins() {
    case "$1" in
        full) echo "$PROFILES_full" ;;
        node) echo "$PROFILES_node" ;;
        dotnet) echo "$PROFILES_dotnet" ;;
        node-docker) echo "$PROFILES_node_docker" ;;
        browser) echo "$PROFILES_browser" ;;
        none) echo "" ;;
        *) return 1 ;;
    esac
}

KNOWN_PROFILES="full node dotnet node-docker browser none"

WORKSPACE=""
NAME=""
PROFILE="${SBX_SANDBOX_PROFILE:-}"
MIXINS="${SBX_SANDBOX_MIXINS:-}"
SOURCE="${SBX_SANDBOX_SOURCE:-}"
REPO="${SBX_SANDBOX_REPO:-nikcio/docker-sandboxing}"
REF="${SBX_SANDBOX_REF:-}"
REPO_DIR="${SBX_SANDBOX_REPO_DIR:-}"
DETACH=""
YES=""

read_default() { # prompt [default]
    local p="$1" d="${2:-}" v=""
    if [ -n "$d" ]; then read -r -e -p "$p [$d]: " v || v=""; else read -r -e -p "$p: " v || v=""; fi
    echo "${v:-$d}"
}

read_confirm() { # prompt [y|n]
    local p="$1" d="${2:-y}" v=""
    local hint="[y/N]"; [ "$d" = "y" ] && hint="[Y/n]"
    while true; do
        read -r -p "$p $hint: " v || v=""
        v="${v:-$d}"
        case "$v" in
            Y|y|Yes|yes) return 0 ;;
            N|n|No|no) return 1 ;;
            *) echo "    Please answer y or n." ;;
        esac
    done
}

read_option() { # prompt [default] "key|desc"...
    local prompt="$1" def="${2:-}"; shift 2
    local keys=() descs=() opt key desc i n
    for opt in "$@"; do keys+=("${opt%%|*}"); descs+=("${opt#*|}"); done
    n=${#keys[@]}
    echo "$prompt" >&2
    for ((i = 0; i < n; i++)); do
        if [ -n "${descs[$i]}" ]; then
            printf "  %d) %-14s %s\n" $((i + 1)) "${keys[$i]}" "${descs[$i]}" >&2
        else
            printf "  %d) %s\n" $((i + 1)) "${keys[$i]}" >&2
        fi
    done
    while true; do
        local v=""
        read -r -p "Choose${def:+ [$def]}: " v || v=""
        v="${v:-$def}"
        if [ -z "$v" ]; then echo "    Invalid choice."; continue; fi
        if [[ "$v" =~ ^[0-9]+$ ]] && [ "$v" -ge 1 ] && [ "$v" -le "$n" ]; then
            echo "${keys[$((v - 1))]}"; return 0
        fi
        for ((i = 0; i < n; i++)); do
            [ "${keys[$i]}" = "$v" ] && { echo "$v"; return 0; }
        done
        echo "    Invalid choice."
    done
}

LIST_ONLY=""
ARG_COUNT=$#
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
        --yes|-y) YES=1; shift ;;
        --list-profiles) LIST_ONLY=1; shift ;;
        -h|--help) sed -n '2,32p' "${BASH_SOURCE[0]}"; exit 0 ;;
        *) WORKSPACE="$1"; shift ;;
    esac
done

if [ -n "$LIST_ONLY" ]; then
    for p in $KNOWN_PROFILES; do
        printf "%-12s %s\n" "$p" "$(profile_mixins "$p" | tr ' ' ',')"
    done
    exit 0
fi

[ -n "$SOURCE" ] || SOURCE="git"

# Wizard mode: no arguments at all, interactive TTY, and not forced off.
WIZARD=0
if [ -z "$YES" ] && [ "$ARG_COUNT" -eq 0 ] && [ -t 0 ]; then
    WIZARD=1
fi

if [ "$WIZARD" = "1" ]; then
    echo "==> Create a new opencode-node-dotnet sandbox"
    echo

    # 1. Workspace
    if [ -z "$WORKSPACE" ]; then
        while true; do
            WORKSPACE="$(read_default "Workspace directory" "$PWD")"
            if [ -d "$WORKSPACE" ]; then WORKSPACE="$(cd "$WORKSPACE" && pwd)"; break; fi
            if read_confirm "Workspace '$WORKSPACE' does not exist. Create it?" y; then
                mkdir -p "$WORKSPACE"
                WORKSPACE="$(cd "$WORKSPACE" && pwd)"
                break
            fi
        done
    else
        WORKSPACE="$(cd "$WORKSPACE" && pwd)"
    fi

    # 2. Mixin profile
    if [ -z "$MIXINS" ]; then
        default_choice="full"
        if [ -n "$PROFILE" ] && profile_mixins "$PROFILE" >/dev/null; then
            default_choice="$PROFILE"
        fi
        opts=()
        for p in $KNOWN_PROFILES; do
            opts+=("${p}|$(profile_mixins "$p" | tr ' ' ',')")
        done
        opts+=("custom|pick mixins yourself")
        choice="$(read_option "Mixin profile" "$default_choice" "${opts[@]}")"
        if [ "$choice" = "custom" ]; then
            while true; do
                echo "    Available mixins:"
                for entry in "${ALL_MIXINS[@]}"; do
                    printf "      %-18s %s\n" "${entry%%|*}" "${entry#*|}"
                done
                raw="$(read_default "Mixins (comma-separated, 'all', or 'none')" "all")"
                if [ "$raw" = "all" ]; then MIXINS="$PROFILES_full"; break; fi
                if [ "$raw" = "none" ]; then MIXINS=""; break; fi
                ok=1 tokens=""
                IFS=',' read -r -a parts <<< "$raw"
                for part in "${parts[@]}"; do
                    part="$(echo -n "$part" | tr -d '[:space:]')"
                    [ -z "$part" ] && continue
                    valid=0
                    for entry in "${ALL_MIXINS[@]}"; do
                        [ "${entry%%|*}" = "$part" ] && valid=1 && break
                    done
                    if [ "$valid" -ne 1 ]; then
                        echo "    Unknown mixin '$part'."
                        ok=0
                        break
                    fi
                    tokens="$tokens $part"
                done
                if [ "$ok" -eq 1 ]; then MIXINS="$(echo $tokens)"; break; fi
            done
        else
            MIXINS="$(profile_mixins "$choice")"
        fi
        PROFILE="$choice"
    fi

    # 3. Kit source
    SOURCE="$(read_option "Kit source" "$SOURCE" \
        "git|fetch kits from github.com/${REPO} (recommended)" \
        "local|use the local clone of this repository")"
    if [ "$SOURCE" = "git" ]; then
        REF="$(read_default "Pin kits to a branch/tag/commit (blank = default branch)" "$REF")"
    else
        [ -n "$REPO_DIR" ] || REPO_DIR="$REPO_ROOT"
        REPO_DIR="$(read_default "Local repository directory" "$REPO_DIR")"
        REPO_DIR="$(cd "$REPO_DIR" && pwd)"
    fi

    # 4. Sandbox name (with attach handling for existing names)
    default_name="opencode-node-dotnet-$(basename "$WORKSPACE")"
    NAME="$(read_default "Sandbox name" "${NAME:-$default_name}")"
    while sbx ls -q 2>/dev/null | grep -qxF "$NAME"; do
        choice="$(read_option "Sandbox '$NAME' already exists" "attach" \
            "attach|re-attach to it (kits are ignored)" \
            "rename|pick a different name" \
            "cancel|exit without doing anything")"
        case "$choice" in
            attach) exec sbx run --name "$NAME" ;;
            cancel) exit 0 ;;
        esac
        NAME="$(read_default "Sandbox name" "$default_name")"
    done

    # 5. Launch mode
    mode="$(read_option "Launch mode" "attach" \
        "attach|create and attach (interactive)" \
        "detach|create only (no attach)")"
    [ "$mode" = "detach" ] && DETACH=1

    # 6. Summary + confirm
    echo
    echo "==> Sandbox configuration"
    echo "    workspace   $WORKSPACE"
    echo "    name        $NAME"
    if [ "$SOURCE" = "git" ]; then
        echo "    source      git: github.com/$REPO${REF:+ @ $REF}"
    else
        echo "    source      local: $REPO_DIR"
    fi
    echo "    mixins      ${MIXINS:-<none>}"
    echo "    launch      $(if [ -n "$DETACH" ]; then echo "create only"; else echo "create and attach"; fi)"
    if ! read_confirm "Proceed" y; then
        exit 0
    fi
    echo
else
    # Scripted mode: fill unset values from defaults.
    [ -n "$PROFILE" ] || PROFILE="full"
    if ! profile_mixins "$PROFILE" >/dev/null; then
        echo "Unknown profile '$PROFILE'. Known: $KNOWN_PROFILES (or pass --mixins)." >&2
        exit 1
    fi
    if [ -z "$WORKSPACE" ]; then WORKSPACE="."; fi
    WORKSPACE="$(cd "$WORKSPACE" && pwd)"
    if [ -z "$MIXINS" ]; then
        MIXINS="$(profile_mixins "$PROFILE")"
    fi
    if [ -z "$NAME" ]; then
        NAME="opencode-node-dotnet-$(basename "$WORKSPACE")"
    fi
    # Re-attach when the sandbox already exists (kits only apply at creation).
    if sbx ls -q 2>/dev/null | grep -qxF "$NAME"; then
        echo "==> Attaching to existing sandbox '$NAME'"
        exec sbx run --name "$NAME"
    fi
fi

# Build the kit references.
KITS=()
if [ "$SOURCE" = "git" ]; then
    git_base="git+https://github.com/${REPO}.git"
    if [ -n "$REF" ]; then
        KITS+=("${git_base}#ref=${REF}&dir=kit-published-node-dotnet")
    else
        KITS+=("${git_base}#dir=kit-published-node-dotnet")
    fi
    if [ -n "$MIXINS" ]; then
        for m in $MIXINS; do
            if [ -n "$REF" ]; then
                KITS+=("${git_base}#ref=${REF}&dir=mixins/${m}")
            else
                KITS+=("${git_base}#dir=mixins/${m}")
            fi
        done
    fi
else
    KITS+=("$REPO_DIR/kit-node-dotnet")
    if [ -n "$MIXINS" ]; then
        for m in $MIXINS; do
            KITS+=("$REPO_DIR/mixins/$m")
        done
    fi
fi

KIT_ARGS=()
for k in "${KITS[@]}"; do KIT_ARGS+=(--kit "$k"); done

if [ "$WIZARD" != "1" ]; then
    echo "==> Creating sandbox '$NAME' (profile: $PROFILE, source: $SOURCE)"
fi
echo "    kits: ${KITS[*]}"

if [ -n "$DETACH" ]; then
    sbx create -q --name "$NAME" "${KIT_ARGS[@]}" opencode-node-dotnet "$WORKSPACE"
else
    sbx run --name "$NAME" "${KIT_ARGS[@]}" opencode-node-dotnet "$WORKSPACE"
fi
