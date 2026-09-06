# Set your Omnium API credentials

With the `omnium` mixin the sandboxed agent — and any app it builds or
runs — can call the Omnium REST API (orders, products, inventory,
customers, carts, …) for your tenant, and read the
[Omnium tech docs](https://docs.omnium.no/docs) without credentials.

Omnium auth is client-credentials: an API user's `ClientId` +
`ClientSecret` are exchanged at `POST /api/token` for a JWT that lives
**10 days**. Apps in the sandbox mint their own tokens; a proxy keeps the
ClientSecret out of the sandbox while they do it.

## 1. Create the API user

1. Log in to the Omnium GUI and open
   **Configuration → Authorization → API Users**.
2. **Create API user** with a descriptive name (e.g. `sandbox`).
3. Copy the generated **ClientId** and **ClientSecret** — the secret is
   shown only once.
4. Assign only the roles the agent needs (least privilege): read-only
   research gets `OrderRead` / `ProductRead` / `CustomerRead`, an
   e-commerce frontend gets `CommerceUser`, full administration gets
   `ApiOwner` (use sparingly). Optionally scope the user to specific
   stores/markets.

## 2. Put the ClientId in your .sbxenv.yaml

The ClientId is an identifier, not a secret — pass it in the `env:` block
(never as a project `.env`, which the kit's env guard removes):

```yaml
env:
  OMNIUM_CLIENT_ID: <client-id>
```

## 3. Register the ClientSecret with sbx

Register it as a **custom secret** so the sandbox only sees a placeholder,
and the proxy swaps in the real value on requests to the Omnium API hosts
(including the token request):

```bash
sbx secret set-custom \
  --host apitest.omnium.no \
  --host api.omnium.no \
  --host apidev.omnium.no \
  --env OMNIUM_CLIENT_SECRET
```

sbx prompts for the value and stores it in its secret store (the OS
keychain). To source the value from a host file instead of the prompt:

```bash
sbx secret set-custom --host apitest.omnium.no --host api.omnium.no \
  --host apidev.omnium.no --env OMNIUM_CLIENT_SECRET \
  --command 'cat ~/.omnium-client-secret'
```

Custom secrets are an experimental sbx feature and apply globally by
default; add `--sandbox <name>` to scope one to a specific sandbox.

## Verify

In the sandbox, mint a token against the Test environment and call the
API:

```bash
TOKEN=$(curl -s -X POST \
  "https://apitest.omnium.no/api/token?clientId=$OMNIUM_CLIENT_ID&clientSecret=$OMNIUM_CLIENT_SECRET")
curl -s -X POST \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"take": 5, "page": 0}' \
  https://apitest.omnium.no/api/orders/Search
```

`OMNIUM_CLIENT_SECRET` holds an `sbx-cs-…` placeholder inside the sandbox;
the host proxy swaps in the real value when the token request goes out.
Integration code that reads `OMNIUM_CLIENT_ID` + `OMNIUM_CLIENT_SECRET`
works the same way without the real secret ever entering the sandbox.

If minting fails with 401, the custom secret isn't registered (or doesn't
cover that host), or the API user was deactivated.

## Environments

- Production `https://api.omnium.no` · Test `https://apitest.omnium.no` ·
  Dev `https://apidev.omnium.no` (each also serves its Swagger UI under
  `/documentation`).
- Test and Dev share data and API users; Production is fully separate —
  API users (and credentials) do not transfer between them.
- The token response body is the JWT (plain text); add
  `&returnAsJson=true` for a structured response with an `expiresIn`
  field.

## Rotate / remove

- Rotate: re-run the same `sbx secret set-custom` command with the new
  value.
- Remove: `sbx secret rm --placeholder <placeholder-value>` — sbx prints
  the placeholder when the secret is created; keep it for this.
- Revoke access anytime by deactivating the API user in the Omnium GUI —
  its tokens stop working on the next call.
- Vault alternative: `--ref 'op://Private/Omnium/client-secret'`.

## How it works

Omnium's token endpoint takes the credentials as query parameters, so the
ClientSecret has to ride in the request itself. sbx custom secrets cover
exactly that: the sandbox sees a generated placeholder in
`OMNIUM_CLIENT_SECRET`, and the proxy replaces the placeholder with the
real value wherever it appears in a request to the registered hosts.
Kit-level header injection can't be used here — it would overwrite the
`Authorization` bearer the app mints for itself.
