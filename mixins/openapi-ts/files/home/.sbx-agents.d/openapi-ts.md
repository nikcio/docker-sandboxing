## openapi-typescript / openapi-fetch

- Docs: openapi-ts.dev (reachable via this mixin) — openapi-typescript
  generates TypeScript types from OpenAPI 3.x schemas; openapi-fetch is
  the tiny typed fetch client built on those types
- Codegen: `npx openapi-typescript <schema file or URL> -o <out>.ts` —
  regenerate whenever the schema changes; never hand-edit the output
- openapi-fetch: `createClient<paths>()` gets full request/response
  typing from the generated types
- Packages install from the npm registry (node mixin); fetching a remote
  schema needs its host allowed (in-project kit)
