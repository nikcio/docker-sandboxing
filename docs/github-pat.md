# Create a GitHub PAT and store it for a sandbox

With a GitHub token the sandboxed agent can use `gh` and push over HTTPS.
The token lives only in sbx's secret store (the OS keychain), scoped to one
sandbox — it never enters the sandbox VM. A proxy injects it into requests
to GitHub hosts.

Use a **fine-grained PAT scoped to just the repositories the agent should
touch**, not your host `gh` login (that one carries broad scopes like
`repo`, `workflow`, `read:org`).

## 1. Create the token

1. On GitHub: **Settings → Developer settings → Personal access tokens →
   Fine-grained tokens → Generate new token**.
2. Resource owner: you (or the org that owns the repos). Expiration per your
   policy (e.g. 90 days).
3. **Repository access → Only select repositories** — pick the repositories
   the agent works on. Everything not listed is invisible to the agent.
4. Repository permissions — this minimum:
   - **Metadata: Read** (mandatory, set automatically)
   - **Contents: Read and write** — clone, commit, push, tags
   - **Pull requests: Read and write**
   - **Issues: Read and write** *(optional)*
   - **Actions: Read** *(optional — view CI status)*
5. Deliberately leave out anything the agent doesn't need:
   - **Workflows: leave unset** — without it the agent cannot alter your CI.
   - No **Administration**, no **Secrets**, no org-wide permissions.

## 2. Store it for the sandbox

```bash
sbx secret set github --sandbox <sandbox-name>
```

sbx prompts for the token ("Enter secret:") and stores it in its secret
store (the OS keychain) at that sandbox's scope. `<sandbox-name>` is the
environment's `name:` from your `.sbxenv.yaml`.

## 3. Approve the credential binding

For environments created from a `.sbxenv.yaml`, the example file already
declares the `bindings.github` block that approves injection for the GitHub
hosts. Keep it in your copy (and in sync with `mixins/git/spec.yaml`). If
you change `bindings:`, recreate the environment.

## Rotate / remove

- Rotation is the same command as storing: `sbx secret set github --sandbox <name>`.
- `sbx env rm` removes the environment's scoped secret; for `--sandbox`-scoped
  secrets use `sbx secret rm github`.
- Revoke the token from GitHub anytime.

## Without a token

The sandbox still works for **public** repositories and for **git over SSH**
(your host SSH agent is forwarded; private keys stay on the host).

If the sandbox starts and prints a `[git-auth]` note, GitHub auth is not
working — run the store command above on your host; it takes effect
immediately, no restart needed.

## Vault alternative

```bash
sbx secret set github --ref 'op://Private/GitHub/pat'
```
