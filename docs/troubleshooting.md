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

## The sandbox prints a `[git-auth]` note at startup

GitHub auth is not working — `git push` / `gh` over HTTPS will fail. Store a
fine-grained PAT on your host (see [github-pat.md](github-pat.md)); it takes
effect immediately, no restart needed. Public repos and git over SSH keep
working without a token.

`gh auth status` showing "not logged in" inside the sandbox is expected —
the sandbox only sees a placeholder token; the real one is injected by the
proxy.

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
  `kits:` lines at local directories (e.g. `./kit`) instead of git refs.
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
