# Set your Omnium API token

With the `omnium` mixin the sandboxed agent can read the
[Omnium tech docs](https://docs.omnium.no/docs) and call the Omnium REST
API (orders, products, inventory, customers, carts, …) for your tenant.
The API credentials are registered on your host only — the sandbox never
sees them. A proxy injects the access token into requests to the Omnium
API hosts as the `Authorization: Bearer` header.

Omnium has no static API key. An API user's `ClientId` + `ClientSecret`
are exchanged at `POST /api/token` for a JWT that lives **10 days** — so
the value you register is that minted token, and the ClientSecret stays
host-side.

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

## 2. Mint a token (host-side)

On your host, exchange the credentials on the API host of the environment
you target:

```bash
TOKEN=$(curl -s -X POST \
  "https://apitest.omnium.no/api/token?clientId=YOUR_CLIENT_ID&clientSecret=YOUR_CLIENT_SECRET")
```

- Production: `https://api.omnium.no` · Test: `https://apitest.omnium.no`
  · Dev: `https://apidev.omnium.no`
- Test and Dev share data and API users; Production is fully separate —
  API users (and tokens) do not transfer between them.
- The response body is the JWT (plain text). Add `&returnAsJson=true` for
  a structured response with an `expiresIn` field.

## 3. Register the token with sbx

```bash
sbx secret set omnium
```

sbx prompts for the token (paste the JWT from step 2) and stores it in its
secret store (the OS keychain).

## 4. Approve the credential binding

The first time you create a sandbox, sbx asks you to approve that the
`omnium` token may be injected for the Omnium API hosts — approve the
prompt. To pre-approve it in your `.sbxenv.yaml` instead:

```yaml
bindings:
  omnium:
    apiKey:
      domains:
        - api.omnium.no
        - apitest.omnium.no
        - apidev.omnium.no
```

If you change `bindings:`, recreate the environment.

## Verify

In the sandbox, run a search against the Test environment:

```bash
curl -s -X POST \
  -H "Authorization: Bearer $OMNIUM_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"take": 5, "page": 0}' \
  https://apitest.omnium.no/api/orders/Search
```

`OMNIUM_API_KEY` holds a `proxy-managed` placeholder inside the sandbox;
the host proxy swaps in the real token. Integration code that uses
`OMNIUM_API_KEY` as its bearer token works without the real token ever
entering the sandbox.

## Rotation

Tokens live 10 days: when API calls start failing with 401, mint a fresh
token (steps 2–3) — rotation is the same commands as registering.

- Revoke access anytime by deactivating the API user in the Omnium GUI —
  its tokens stop working on the next call.
- Vault alternative: `sbx secret set omnium --ref 'op://Private/Omnium/token'`.
- Optional auto-refresh: keep the ClientSecret in a host-only file and
  store a command that mints a token on refresh —
  `sbx secret set omnium --command 'curl -s -X POST "https://apitest.omnium.no/api/token?clientId=YOUR_CLIENT_ID&clientSecret=$(cat ~/.omnium-client-secret)"' --refresh 24h`

## Without a token

The docs ([docs.omnium.no/docs](https://docs.omnium.no/docs),
[help.omnium.no](https://help.omnium.no)) and the Swagger UIs stay
reachable for research; API calls fail with 401 until you register a
token.
