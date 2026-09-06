# Troubleshooting

## A download fails inside the sandbox

The network policy is deny-by-default; the host is probably not allowed.
Check what was blocked, then allow it in the owning mixin:

```bash
sbx policy log          # on your host — shows blocked hosts
```

Add the host to the mixin's `permissions.network.allow` in
`mixins/<area>/spec.yaml`, then recreate the sandbox (kit changes never
apply to running sandboxes):

```bash
sbx rm <sandbox-name>
sbx env run
```

## Git push / `gh` fails with auth errors

GitHub auth is not configured for this sandbox — store a fine-grained PAT
on your host (see [github-pat.md](github-pat.md)); it takes effect
immediately, no restart needed. Public repos and git over SSH keep working
without a token.

`gh auth status` showing "not logged in" inside the sandbox is expected —
the sandbox only sees a placeholder token; the real one is injected by the
proxy.

## GitHub Copilot sign-in fails

Copilot signs in with the OAuth device flow (`/connect` in the TUI — see
[copilot-setup.md](copilot-setup.md)). If it fails while a GitHub PAT is
stored for the sandbox, the PAT injection into `github.com`/`api.github.com`
requests can override opencode's own Copilot auth headers — remove the
stored PAT (`sbx secret rm github`), sign in again, and re-store the PAT
only if git/`gh` auth still works.

## The agent uses the wrong model provider

Model providers are mixins (`zeldoc`, `copilot`); their configs are
merged fragments — see [mixins.md](mixins.md). With both composed,
`zeldoc/zdev-2` is the default model; switch with `/models` or set
`model` in a project-level `opencode.jsonc`. Provider changes only apply
to new sandboxes: recreate with `sbx rm <name>` + `sbx env run`.

## The sandbox refuses to start (.env files)

The workspace must not contain `.env` files (secrets stay out of the
sandbox):

- **Direct mode** (default): the sandbox refuses to start. Delete the `.env`
  file(s) in your project on the host, then recreate.
- **Clone mode** (`clone: true`): the sandbox deletes them from its local
  copy and warns after a 5-second pause.

## My changes to a kit/mixin/template are not picked up

- **Kit/mixin spec changes** apply only to new sandboxes — recreate with
  `sbx rm <name>` + `sbx env run`. While iterating on a clone, point the
  `kits:` lines at local directories (e.g. `./kit-node-dotnet`) instead of
  git refs.
- **Template (Dockerfile) changes** need a rebuild and reload, then a
  sandbox recreation. From a clone of this repo:
  `./scripts/bootstrap.sh` (or `bootstrap.ps1`).

## The agent's dev server is not reachable from my host

Publish the port in your `.sbxenv.yaml`:

```yaml
ports:
  - sandbox: 3000
    host: 3000
```

Services inside the sandbox must listen on `0.0.0.0` (not `127.0.0.1`).

To reach a service running on your host from inside the sandbox, use
`host.docker.internal` (e.g. `curl http://host.docker.internal:3000`).
