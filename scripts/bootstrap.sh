#!/usr/bin/env bash
# One-time host setup for the opencode-node-dotnet Docker Sandboxes kit.
#
#   - Enables clipboard image paste for sandboxes (sbx settings).
#   - Allows the GitHub kit source (kit.allowedSources) so kits/mixins are
#     fetched from github.com/nikcio/docker-sandboxing.
#   - Builds the template image and loads it into the sandbox runtime
#     (or pushes it when PUSH_REGISTRY is set).
#   - Registers the Zeldoc.ai API key as a proxy-managed service secret
#     (the real key never enters the sandbox) and pre-creates the
#     credential binding.
#   - Registers the configurable `sbx-new` shell function (skip with
#     SKIP_ALIAS=1).
#   - Validates the kit and mixins.
#
# Usage:
#   ZELDOC_API_KEY=zd-... ./scripts/bootstrap.sh
#   PUSH_REGISTRY=docker.io/myorg ZELDOC_API_KEY=zd-... ./scripts/bootstrap.sh
#   SKIP_BUILD=1 SKIP_ALIAS=1 ./scripts/bootstrap.sh   # selective runs

set -euo pipefail

TEMPLATE_TAG="${TEMPLATE_TAG:-opencode-node-dotnet:v1}"
PUSH_REGISTRY="${PUSH_REGISTRY:-}"
SKIP_BUILD="${SKIP_BUILD:-}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

step() { printf '\033[36m==> %s\033[0m\n' "$1"; }

step "sbx settings: allow clipboard image paste"
sbx settings set clipboard.imagePaste true

# Kits are fetched from GitHub, so the source must be in the allowlist.
# The setting replaces the whole list — merge, don't overwrite.
KIT_SOURCE="github.com/nikcio/"
step "sbx settings: allow kit source ${KIT_SOURCE}"
current="$(sbx settings get kit.allowedSources 2>/dev/null || echo '[]')"
if ! grep -qF "${KIT_SOURCE}" <<<"${current}"; then
    json="["
    first=1
    for e in "docker.io/" $(grep -oE '"[^"]+"' <<<"${current}" | tr -d '"' | grep -v '^$' || true) "${KIT_SOURCE}"; do
        [ -z "$e" ] && continue
        if printf '%s' "$json" | grep -qF "\"$e\""; then continue; fi
        [ $first -eq 1 ] || json+=","
        json+="\"$e\""
        first=0
    done
    json+="]"
    sbx settings set kit.allowedSources "$json"
fi

# Register the configurable `sbx-new` launcher as a shell function.
# Skip with SKIP_ALIAS=1. Idempotent (marker comment).
if [ -z "${SKIP_ALIAS:-}" ]; then
    marker="sbx-new (docker-sandboxing)"
    launcher="${REPO_ROOT}/scripts/new-sandbox.sh"
    rc_file="${HOME}/.bashrc"
    if ! grep -qF "$marker" "$rc_file" 2>/dev/null; then
        step "Registering sbx-new function in ${rc_file}"
        cat >> "$rc_file" <<EOF

# ${marker} — configurable sandbox launcher (see scripts/new-sandbox.sh)
sbx-new() { bash "${launcher}" "\$@"; }
EOF
    fi
fi

if [ -z "${SKIP_BUILD}" ]; then
    step "Building template image ${TEMPLATE_TAG}"
    docker build -t "${TEMPLATE_TAG}" "${REPO_ROOT}/template"

    if [ -n "${PUSH_REGISTRY}" ]; then
        remote="${PUSH_REGISTRY}/${TEMPLATE_TAG}"
        step "Pushing template to ${remote}"
        docker tag "${TEMPLATE_TAG}" "${remote}"
        docker push "${remote}"
        echo "    Update sandbox.image in kit/spec.yaml to: ${remote}"
    else
        mkdir -p "${REPO_ROOT}/dist"
        tar_path="${REPO_ROOT}/dist/$(echo "${TEMPLATE_TAG}" | tr ':/' '--').tar"
        step "Saving image to ${tar_path}"
        docker image save "${TEMPLATE_TAG}" -o "${tar_path}"
        step "Loading template into the sandbox runtime"
        sbx template load "${tar_path}"
    fi
fi

step "Registering Zeldoc API key (proxy-managed; never enters the sandbox)"
if [ -z "${ZELDOC_API_KEY:-}" ]; then
    read -r -s -p "ZELDOC_API_KEY: " ZELDOC_API_KEY
    echo
fi
if [ -z "${ZELDOC_API_KEY}" ]; then
    echo "No Zeldoc API key provided (set \$ZELDOC_API_KEY or re-run)." >&2
    exit 1
fi
printf '%s\n' "${ZELDOC_API_KEY}" | sbx secret set zeldoc
unset ZELDOC_API_KEY

# Third-party v2 kits need a credential binding approval. The first
# interactive `sbx run` prompts for it; pre-create it for unattended use.
if [ -n "${APPDATA:-}" ] && [ -d "${APPDATA}" ]; then
    bindings="${APPDATA}/sbx/credentials.yaml" # Windows (Git Bash)
else
    bindings="${XDG_CONFIG_HOME:-$HOME/.config}/sbx/credentials.yaml"
fi
if [ ! -f "${bindings}" ]; then
    step "Pre-creating credential binding for zeldoc (${bindings})"
    mkdir -p "$(dirname "${bindings}")"
    cat > "${bindings}" <<'YAML'
bindings:
  zeldoc:
    apiKey:
      domains:
        - api.zeldoc.ai
YAML
elif ! grep -q "zeldoc" "${bindings}"; then
    echo "    Hint: add a 'zeldoc' apiKey binding (domains: api.zeldoc.ai) to ${bindings},"
    echo "    or approve it interactively on the first 'sbx run'."
fi

step "Validating kit and mixins"
for mixin in "${REPO_ROOT}"/mixins/*/; do
    sbx kit validate "${mixin}"
done
sbx kit validate "${REPO_ROOT}/kit"

cat <<EOF

Done. Open a NEW shell so 'sbx-new' is loaded, then launch with:
  sbx-new                                   # guided wizard
  sbx-new <path-to-project>                 # scripted: full stack from GitHub
  sbx-new --profile node <path-to-project>  # node-only mixin set
  sbx-new --list-profiles                   # all profiles

Tip: kit changes only apply to NEW sandboxes. Recreate with:
  sbx rm <sandbox-name> && sbx-new <path-to-project>
EOF
