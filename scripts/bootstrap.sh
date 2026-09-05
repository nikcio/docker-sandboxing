#!/usr/bin/env bash
# Local development setup for the template, kit, and mixins.
#
#   - Builds the template image and loads it into the sandbox runtime
#     (or pushes it when PUSH_REGISTRY is set).
#   - Registers the Zeldoc.ai API key (proxy-managed — the sandbox never
#     sees it) and pre-creates the credential binding. Already-stored
#     secrets are skipped; an env var always (re)registers.
#   - Validates the kit and mixins.
#
# Usage:
#   ZELDOC_API_KEY=zd-... ./scripts/bootstrap.sh
#   PUSH_REGISTRY=docker.io/myorg ZELDOC_API_KEY=zd-... ./scripts/bootstrap.sh
#   SKIP_BUILD=1 ./scripts/bootstrap.sh                # skip the template build

set -euo pipefail

TEMPLATE_TAG="${TEMPLATE_TAG:-opencode-node-dotnet:v1}"
PUSH_REGISTRY="${PUSH_REGISTRY:-}"
SKIP_BUILD="${SKIP_BUILD:-}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

step() { printf '\033[36m==> %s\033[0m\n' "$1"; }

# secret_stored <service> — true when sbx already holds a secret for the
# service (matched on the TYPE/NAME columns of `sbx secret ls`).
secret_stored() {
    sbx secret ls 2>/dev/null | grep -Eq "(^|[[:space:]])service[[:space:]]+${1}([[:space:]]|$)"
}

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
if [ -n "${ZELDOC_API_KEY:-}" ]; then
    printf '%s\n' "${ZELDOC_API_KEY}" | sbx secret set zeldoc
elif secret_stored zeldoc; then
    echo "    Already registered - skipping (set \$ZELDOC_API_KEY to update it)."
else
    read -r -s -p "ZELDOC_API_KEY: " ZELDOC_API_KEY
    echo
    if [ -z "${ZELDOC_API_KEY}" ]; then
        echo "No Zeldoc API key provided (set \$ZELDOC_API_KEY or re-run)." >&2
        exit 1
    fi
    printf '%s\n' "${ZELDOC_API_KEY}" | sbx secret set zeldoc
fi
unset ZELDOC_API_KEY

# GitHub tokens are provisioned PER SANDBOX with sbx's own prompt
# (`sbx secret set github --sandbox <name>`) — never globally here.
# See docs/github-pat.md.

# Third-party v2 kits need a credential binding approval. The first
# interactive `sbx run` prompts for it; pre-create it for unattended use.
if [ -n "${APPDATA:-}" ] && [ -d "${APPDATA}" ]; then
    bindings="${APPDATA}/sbx/credentials.yaml" # Windows (Git Bash)
else
    bindings="${XDG_CONFIG_HOME:-$HOME/.config}/sbx/credentials.yaml"
fi
if [ ! -f "${bindings}" ]; then
    step "Pre-creating credential bindings for zeldoc (${bindings})"
    mkdir -p "$(dirname "${bindings}")"
    cat > "${bindings}" <<'YAML'
bindings:
  zeldoc:
    apiKey:
      domains:
        - api.zeldoc.ai
YAML
else
    if ! grep -q "zeldoc" "${bindings}"; then
        echo "    Hint: add a 'zeldoc' apiKey binding (domains: api.zeldoc.ai) to ${bindings},"
        echo "    or approve it interactively on the first 'sbx run'."
    fi
fi

step "Validating kit and mixins"
for mixin in "${REPO_ROOT}"/mixins/*/; do
    sbx kit validate "${mixin}"
done
sbx kit validate "${REPO_ROOT}/kit"

cat <<EOF

Done. Develop the kits/mixins in a sandbox:
  sbx env run                    # from this repo root (uses ./.sbxenv.yaml, local kits)
  ./scripts/new-sandbox.sh       # wizard launcher for any workspace

Tip: kit changes only apply to NEW sandboxes. Recreate with:
  sbx rm <sandbox-name> && sbx env run

GitHub PAT for a sandbox (lets the agent push and open PRs): store it with
sbx's own prompt, scoped to the sandbox (see docs/github-pat.md):
  sbx secret set github --sandbox <sandbox-name>
EOF
