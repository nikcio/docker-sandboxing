## openapi-typescript / openapi-fetch

- Docs: openapi-ts.dev
- Codegen: `npx openapi-typescript <schema file or URL> -o <out>.ts` —
  regenerate whenever the schema changes; never hand-edit the output
- openapi-fetch: `createClient<paths>()` gets full request/response
  typing from the generated types
