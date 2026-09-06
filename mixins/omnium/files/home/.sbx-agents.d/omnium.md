## Omnium (omnium.no)

- Omnium auth is client-credentials: mint a token with
  `curl -s -X POST "$HOST/api/token?clientId=$OMNIUM_CLIENT_ID&clientSecret=$OMNIUM_CLIENT_SECRET"`
  — the response body is the JWT (plain text; add `&returnAsJson=true` for
  JSON). Send `Authorization: Bearer <token>` on API calls, reuse it (valid
  10 days), and re-mint on 401.
- `$HOST` is the environment's API host: Test `https://apitest.omnium.no`
  (mirrors the upcoming release), Dev `https://apidev.omnium.no` (shares
  data + API users with Test), Production `https://api.omnium.no` (fully
  separate — API users and tokens don't transfer). Default to Test unless
  the task explicitly targets production. Swagger per environment:
  `https://<api-host>/documentation`.
- `OMNIUM_CLIENT_ID` (not a secret) comes from the project's `.sbxenv.yaml`
  `env:` block. `OMNIUM_CLIENT_SECRET` is proxy-managed as a custom secret:
  the sandbox sees an `sbx-cs-…` placeholder, and the host proxy swaps the
  real value into requests to the Omnium API hosts wherever the placeholder
  appears — including the `/api/token` query string. The real ClientSecret
  never enters the sandbox. Never write credentials or minted JWTs into
  workspace files.
- If minting fails, the custom secret is missing, doesn't cover that host,
  or the API user was deactivated — tell the user to re-register it with
  `sbx secret set-custom` (see docs/omnium-api-key.md in the
  docker-sandboxing repo), then retry once.
- Docs for research (no credentials needed): https://docs.omnium.no/docs,
  user docs https://help.omnium.no, system status https://status.omnium.no.
- Searches are POSTs (`POST /api/orders/Search` with `{ "take": n,
  "page": n }`, max 100 per page; use `/api/{resource}/Scroll` beyond
  10,000 hits). Handle 429 with the `Retry-After` header (exponential
  backoff).
