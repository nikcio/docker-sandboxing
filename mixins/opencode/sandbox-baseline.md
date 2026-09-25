# Sandbox environment

OpenCode sandbox: outbound traffic goes through a deny-by-default proxy firewall, and GitHub
credentials are injected at the network level — push and open PRs directly from inside the sandbox.

## Environment facts

- Outbound network is deny-by-default; blocked requests return HTTP 403 with a structured body
  naming the case. Ask the user to run `sbx policy allow network <domain>` on the host (or
  inspect with `sbx policy log`); details in the workspace AGENTS.md's network section.
- `/etc/sandbox-persistent.sh` is sourced before every bash command — persist tool env setup
  there, never shell completions (they break the bash tool).
- HTTPS Git auth is proxy-injected. If `git push` fails with auth errors, ask the user to run
  `sbx secret set github --sandbox <sandbox-name>` on the host. Push and open PRs from inside
  the sandbox — never ask the user to push from their host terminal.
- Workspace mode check: `if [ -d /run/sandbox/source ]; then echo "clone mode"; else echo "direct mode"; fi`.
  Direct mode: the host tree is mounted — edits appear on the host immediately. Clone mode: a
  standalone clone at start; commits reach the host via `git fetch sandbox-<name>` while the
  sandbox is up.
- sudo is available for installing packages.
- OpenCode config: the global layer (`OPENCODE_CONFIG`) is generated — set project defaults in
  the repo-root `opencode.jsonc` (e.g. `"model"`), not in `~/.config/opencode/`.
