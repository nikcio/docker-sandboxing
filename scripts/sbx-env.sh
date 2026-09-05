#!/usr/bin/env bash
# Launcher for `sbx env run` that provisions the environment's GitHub PAT
# first, using sbx's own prompt (POSIX variant of scripts/sbx-env.ps1 - keep
# the two in sync).
#
# sbx resolves `secrets.*.command` entries non-interactively (a "host shell
# command whose standard output becomes the secret", per the
# environment-files docs), so an interactive prompt inside a secret command
# can never work: sbx captures stdout as the secret and only surfaces stderr
# on failures. The token therefore lives in sbx's secret store (the OS
# keychain), scoped per sandbox, and this launcher makes sure it exists
# BEFORE `sbx env run` provisions secrets:
#
#   1. Reads `name:` from the environment file (first non-flag argument,
#      else ./.sbxenv.yaml or ./.sbxenv.yml).
#   2. If `sbx secret ls` has no github entry scoped to that sandbox, runs
#      `sbx secret set github --sandbox <name>` - sbx prompts ("Enter
#      secret:") and stores the value in the keychain. Fine-grained PATs
#      only - never the broad-scope host `gh auth token`. Skipping (empty
#      input) is fine: public repos and git over SSH keep working, and the
#      in-sandbox [git-auth] banner will point at this same command.
#   3. Runs `sbx env run` with all arguments.
#
# Rotation: re-run `sbx secret set github --sandbox <name>`, or remove and
# recreate the environment.
#
# Prompts must go to stderr so command substitution only captures the final
# sandbox output.

set -euo pipefail

env_file=""
for arg in "$@"; do
    case "$arg" in
        -*) ;;
        *)
            if [ -f "$arg" ]; then env_file="$arg"; break; fi
            for fname in .sbxenv.yaml .sbxenv.yml; do
                if [ -f "$arg/$fname" ]; then env_file="$arg/$fname"; break 2; fi
            done
            ;;
    esac
done
if [ -z "$env_file" ]; then
    for fname in .sbxenv.yaml .sbxenv.yml; do
        [ -f "$fname" ] && { env_file="$fname"; break; }
    done
fi

sandbox_name=""
if [ -n "$env_file" ]; then
    name_line="$(grep -E '^name:' "$env_file" | head -1 || true)"
    if [ -n "$name_line" ]; then
        sandbox_name="$(printf '%s' "${name_line#name:}" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'\$//")"
        [ -n "$sandbox_name" ] || sandbox_name=""
    fi
fi

if [ -n "$sandbox_name" ]; then
    # Provision the per-sandbox GitHub token through sbx's own prompt when
    # it is not stored yet. sbx captures stdout as the secret, so this must
    # happen outside secret resolution - hence this launcher.
    if ! sbx secret ls 2>/dev/null | grep -Eq "(^|[[:space:]])${sandbox_name}[[:space:]]+service[[:space:]]+github([[:space:]]|$)"; then
        printf 'No GitHub token stored for sandbox "%s" yet.\n' "$sandbox_name" >&2
        printf "sbx will prompt for a fine-grained PAT (stored only in sbx's secret store).\n" >&2
        printf 'Leave it empty to skip - public repos and git over SSH keep working.\n\n' >&2
        sbx secret set github --sandbox "$sandbox_name" || {
            printf 'No token stored - continuing. Public repos and git over SSH still work;\n' >&2
            printf 'store one later with: sbx secret set github --sandbox %s\n' "$sandbox_name" >&2
        }
    fi
else
    printf "Could not read a 'name:' from the environment file - skipping the GitHub token check.\n" >&2
fi

exec sbx env run "$@"
