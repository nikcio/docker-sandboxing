#!/usr/bin/env bash
#
# sbx sandbox setup example (macOS / Linux)
#
# A guided, copy-and-tweak script that:
#   1. Installs the `sbx` CLI if it isn't already installed
#   2. Signs you in with `sbx login`
#   3. Registers your secrets:
#        - zeldoc  (global  — the model provider key)
#        - github  (sandbox — the GitHub token so the agent can push)
#   4. Prints how to run the sandbox
#
# === How to customize ======================================================
# Everything you are likely to change lives in the CONFIGURATION section
# below. Add your own sandbox-scoped secrets by extending SECRET_HELPERS
# and calling `secret_set` where marked.
# ===========================================================================

set -euo pipefail

# --------------------------------------------------------------------------
# CONFIGURATION — tweak these to fit your project
# --------------------------------------------------------------------------

# Sandbox name: must match the `name:` in your sbxenv.yaml. Secrets set with
# `--sandbox <name>` are scoped to this sandbox only.
SANDBOX_NAME="${SANDBOX_NAME:-my-project}"

# Example file to copy into your project (see examples/ in this repo).
# Set SANDBOX_EXAMPLE to match your stack: dotnet, go, node, python, rust.
SANDBOX_EXAMPLE="${SANDBOX_EXAMPLE:-go}"

# Install command for sbx per platform. Override if you prefer another
# method (see https://docs.docker.com/ai/sandboxes/install/).
SBX_INSTALL_MACOS="brew install docker/tap/sbx"
SBX_INSTALL_LINUX="curl -fsSL https://get.docker.com | sudo SBX=1 sh"

# --------------------------------------------------------------------------
# SECRET HELPERS — extend this section for your own secrets
#
#   secret_set <secret-name> <scope> <docs-url>
#     scope: "global" (available to every sandbox) or "sandbox" (scoped to
#     $SANDBOX_NAME only).
#
#   Example — add a private npm token scoped to the sandbox:
#     secret_set npm_token sandbox "https://docs.npmjs.com/creating-and-viewing-access-tokens"
#   ...then reference it from sbxenv.yaml bindings.
# --------------------------------------------------------------------------

secret_set() {
  local name="$1" scope="$2" docs="$3"
  local scope_args=()
  if [[ "$scope" == "sandbox" ]]; then
    scope_args=(--sandbox "$SANDBOX_NAME")
  fi

  echo
  echo "--- Secret: ${name} (${scope}) ----------------------------------"
  echo "Docs: ${docs}"
  if sbx secret get "$name" "${scope_args[@]}" >/dev/null 2>&1; then
    read -r -p "A ${name} secret already exists. Replace it? [y/N] " replace
    if [[ ! "${replace:-}" =~ ^[Yy]$ ]]; then
      echo "Keeping existing ${name} secret."
      return 0
    fi
  fi
  echo "You will be prompted for the value; it is stored in your OS keychain,"
  echo "never in the sandbox."
  sbx secret set "$name" "${scope_args[@]}"
}

# --------------------------------------------------------------------------
# Helpers
# --------------------------------------------------------------------------

step() { printf '\n=== %s ===\n' "$*"; }
info() { printf '  %s\n' "$*"; }

have() { command -v "$1" >/dev/null 2>&1; }

# --------------------------------------------------------------------------
# 1. Install sbx if missing
# --------------------------------------------------------------------------

step "Step 1/4: sbx CLI"

if have sbx; then
  info "sbx already installed: $(sbx version 2>/dev/null || sbx --version 2>/dev/null || echo 'unknown version')"
else
  info "sbx not found. Installing..."
  case "$(uname -s)" in
    Darwin)
      if ! have brew; then
        echo "ERROR: Homebrew is required on macOS. Install it from https://brew.sh and rerun." >&2
        exit 1
      fi
      # shellcheck disable=SC2086
      $SBX_INSTALL_MACOS
      ;;
    Linux)
      # shellcheck disable=SC2086
      $SBX_INSTALL_LINUX
      ;;
    *)
      echo "ERROR: unsupported platform '$(uname -s)'. Install sbx manually:" >&2
      echo "  https://docs.docker.com/ai/sandboxes/install/" >&2
      exit 1
      ;;
  esac
  have sbx || { echo "ERROR: sbx still not found after install; open a new shell or install manually." >&2; exit 1; }
  info "sbx installed."
fi

# --------------------------------------------------------------------------
# 2. Sign in
# --------------------------------------------------------------------------

step "Step 2/4: sign in"

if sbx whoami >/dev/null 2>&1; then
  info "Already signed in as: $(sbx whoami 2>/dev/null || true)"
else
  info "Opening a browser for Docker OAuth..."
  sbx login
fi

# --------------------------------------------------------------------------
# 3. Set up secrets
# --------------------------------------------------------------------------

step "Step 3/4: secrets"

# zeldoc: GLOBAL — model provider key, available to all your sandboxes.
# Get/register/approve your key: https://docs.zeldoc.ai/connect-opencode
# Repo docs: docs/zeldoc-api-key.md
secret_set zeldoc global "https://docs.zeldoc.ai/connect-opencode"

# github: SANDBOX-SCOPED — fine-grained PAT so the agent can push over HTTPS.
# Create one at GitHub: Settings -> Developer settings -> Personal access
# tokens -> Fine-grained tokens (repo guide: docs/github-pat.md).
# Skip this if you only clone public repos / use git over SSH.
read -r -p "Set a GitHub token for sandbox '${SANDBOX_NAME}'? [Y/n] " want_github
if [[ ! "${want_github:-}" =~ ^[Nn]$ ]]; then
  secret_set github sandbox "https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-fine-grained-personal-access-tokens"
else
  info "Skipped github secret (pushing over HTTPS won't work without it)."
fi

# >>> Add more secrets here, e.g.:
# secret_set npm_token sandbox "https://docs.npmjs.com/creating-and-viewing-access-tokens"

# --------------------------------------------------------------------------
# 4. Host settings (one-time)
# --------------------------------------------------------------------------

step "Step 4/4: host settings"

# sbx only fetches kits from allowed sources. The set call REPLACES the whole
# list, so merge with existing entries first.
info "Allowing the kit sources for this repo's kits..."
existing="$(sbx settings get kit.allowedSources 2>/dev/null || echo '[]')"
info "Current kit.allowedSources: ${existing}"
sbx settings set kit.allowedSources '["docker.io/","github.com/nikcio/"]'
info "Done. If you had other entries in the list, re-add them (rerun this step after checking the output above)."

# Optional: let the sandboxed agent read images you paste
# sbx settings set clipboard.imagePaste true

# --------------------------------------------------------------------------
# Done — how to run
# --------------------------------------------------------------------------

step "All set"

cat <<EOF
Next steps:

  1. Copy the example env file for your stack into your project root
     and rename it sbxenv.yaml (commit it so teammates get the same sandbox):

       cp examples/opencode-${SANDBOX_EXAMPLE}.sbxenv.yaml /path/to/your/project/sbxenv.yaml

     In the copy, set \`name: ${SANDBOX_NAME}\` (or update SANDBOX_NAME in
     this script) and drop the \`kits:\` lines you don't need.

  2. Run the sandbox from your project root:

       sbx env run

     The sandbox starts and launches OpenCode automatically. Quitting
     OpenCode exits the sandbox; rerun the command to start it again.

  3. First run only: sbx asks you to approve credential bindings
     (which hosts may receive which secret). Approve the prompts.

Useful commands:

  sbx env rm      # remove the sandbox (and its scoped secrets)
  sbx secret set github --sandbox ${SANDBOX_NAME}   # rotate a secret

Full guide: docs/getting-started.md in this repo, or
https://docs.docker.com/ai/sandboxes/
EOF
