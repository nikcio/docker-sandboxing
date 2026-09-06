## Uniform (uniform.app)

- The Uniform API key is proxy-managed: the sandbox only sees a
  `UNIFORM_API_KEY=proxy-managed` placeholder. The host proxy swaps the real
  key into `x-api-key` headers on requests to `uniform.app` / `uniform.global`
  (and the `eu.*` hosts). Never write a real Uniform key into workspace files.
- The Uniform SDK and CLI both read `UNIFORM_API_KEY`; the CLI also accepts
  `UNIFORM_CLI_API_KEY`. `UNIFORM_PROJECT_ID` is required for API calls and is
  not a secret — set it in `.sbxenv.yaml`'s `env:` block, never in a project
  `.env` (the kit's env guard removes those).
- Docs for research: https://docs.uniform.app/docs
- EU-region teams must also set `UNIFORM_CLI_BASE_URL=https://eu.uniform.app`
  and `UNIFORM_CLI_BASE_EDGE_URL=https://eu.uniform.global`.
- Verify credentials with `npx uniform whoami` (reports masked key, project,
  and API hosts).
