#!/usr/bin/env bash
# Launcher for `sbx env run` that provisions the sandbox's GitHub PAT first,
# using sbx's own prompt (POSIX variant of scripts/sbx-env.ps1 - keep in sync).
#
# sbx can't prompt while resolving a `secrets.*.command` entry (stdout is
# captured as the secret), so this launcher checks `sbx secret ls` for a
# github entry scoped to the environment file's `name:` and, when missing,
# runs `sbx secret set github --sandbox <name>` before `sbx env run`. Empty
# input skips the token — public repos and git over SSH keep working.
# Rotation is the same command; prompts go to stderr so command substitution
# only captures the sandbox output.

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
