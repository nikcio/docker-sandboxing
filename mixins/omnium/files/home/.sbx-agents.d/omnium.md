## Omnium (omnium.no)

- The Omnium API token is proxy-managed: the sandbox only sees an
  `OMNIUM_API_KEY=proxy-managed` placeholder. The host proxy swaps the real
  JWT into the `Authorization` header on requests to the API hosts
  (`api.omnium.no`, `apitest.omnium.no`, `apidev.omnium.no`). Never write a
  real token, ClientId, or ClientSecret into workspace files.
- Omnium uses JWT bearer auth: send `Authorization: Bearer $OMNIUM_API_KEY`
  on every API call. Tokens are minted host-side from an API user's
  ClientId + ClientSecret (`POST /api/token`) and live 10 days — they
  cannot be minted in-sandbox. On 401 the stored token expired or the API
  user was deactivated: tell the user to re-mint and re-register it
  (`sbx secret set omnium`), then retry the call once.
- Environments: Test (`https://apitest.omnium.no`) mirrors the upcoming
  release and shares data + API users with Dev (`https://apidev.omnium.no`);
  Production (`https://api.omnium.no`) is fully separate. Default to Test
  unless the task explicitly targets production. Swagger per environment:
  `https://<api-host>/documentation`.
- Docs for research (no token needed): https://docs.omnium.no/docs, user
  docs https://help.omnium.no, system status https://status.omnium.no.
- Searches are POSTs (`POST /api/orders/Search` with `{ "take": n,
  "page": n }`, max 100 per page; use `/api/{resource}/Scroll` beyond
  10,000 hits). Handle 429 with the `Retry-After` header (exponential
  backoff) and reuse the token — don't re-authenticate.
