# Use GitHub Copilot in the sandbox

The `copilot` mixin makes the sandboxed agent use your GitHub Copilot
subscription as its model provider — instead of (or alongside) Zeldoc.ai.
Unlike Zeldoc there is **no host-side key**: opencode signs in with
GitHub's OAuth device flow, and the token stays inside the sandbox VM
(`~/.local/share/opencode/auth.json`).

## 1. Add the mixin

Add the `copilot` mixin to the `kits:` list in your `.sbxenv.yaml` (pin
the same `&ref=` as the other mixin lines). The default examples use
Zeldoc — swap the `zeldoc` line for `copilot`, or keep both:

```yaml
kits:
  # ...kit + base + opencode-runtime + stack mixins...
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/copilot&ref=<same ref as the other mixins>
```

## 2. Create the sandbox and sign in

```bash
sbx env run
```

The opencode TUI starts with GitHub Copilot as the provider. If Copilot
isn't signed in yet (fresh sandbox = fresh sign-in):

1. Run `/connect` in the TUI and pick **GitHub Copilot**.
2. opencode shows a code — open [github.com/login/device](https://github.com/login/device)
   **in your host browser** and enter it.
3. Run `/models` to pick a model.

No default model is picked for you — set it in a project-level
`opencode.jsonc` (see [project-kit.md](project-kit.md)) or pick per
session with `/models`; some models (e.g. the flagship Claude/GPT ones)
need a higher Copilot plan
([plans](https://github.com/features/copilot/plans)) — if a model
errors, pick another with `/models`.

## Zeldoc and Copilot together

Compose both mixins and both providers stay enabled (the sandbox merges
each provider's config fragment; `enabled_providers` lists are unioned).
The default model comes from your project-level `opencode.jsonc` —
commit one in the repo root with `"model": "github-copilot/<model-id>"`
(or a Zeldoc model), or just switch models per session with `/models`.
Details: [mixins.md](mixins.md).

## Good to know

- **The OAuth token lives in the sandbox VM** — it grants Copilot access
  only, and the sandbox egress policy limits where it can be used. This
  is opencode's built-in sign-in flow; there is no proxy-managed Copilot
  credential.
- Signing in again is needed per sandbox (recreate = sign in again).
- Network egress added by this mixin: `github.com` (device-flow sign-in +
  token refresh), `api.github.com` (Copilot session token), and
  `api.githubcopilot.com` (inference; the wildcard covers
  business/enterprise endpoints).

## Troubleshooting

- **Sign-in or Copilot requests fail while a GitHub PAT is stored** —
  the PAT proxy injection (see [github-pat.md](github-pat.md)) rewrites
  auth headers to `github.com`/`api.github.com` and can clobber
  opencode's own Copilot auth headers. Remove the stored PAT
  (`sbx secret rm github`), sign in again, and store the PAT afterwards
  only if git/`gh` auth still works.
- **A model reports an error** — plan limitation or model rename; pick
  another with `/models`.
- **Blocked hosts** — check `sbx policy log` and see
  [troubleshooting.md](troubleshooting.md).
