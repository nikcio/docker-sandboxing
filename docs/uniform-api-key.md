# Set your Uniform API key

With the `uniform` mixin the sandboxed agent can read the
[Uniform docs](https://docs.uniform.app/docs) and call the Uniform platform
APIs (Management API and Edge Delivery API) from a frontend integration.
The key is registered on your host only — the sandbox never sees it. A proxy
injects it into requests to `uniform.app` / `uniform.global` (and the EU
hosts) as the `x-api-key` header the Uniform SDK and CLI use.

## 1. Create the key

1. Open your team at [uniform.app](https://uniform.app) (or
   [eu.uniform.app](https://eu.uniform.app) for EU teams).
2. **Security → Service Accounts → Add Service Account** (personal access
   tokens work too).
3. Name it (e.g. `sandbox`) and assign the roles the agent needs for your
   project — delivery/read roles unless the agent should push content or
   run CLI commands that write.
4. Create it and copy the **Key**. From the same screen, also copy the
   project ID ("Copy as Project ID"). Skip "Copy as .env" — the key must
   not land in a project `.env` (see step 3).

## 2. Register the key with sbx

```bash
sbx secret set uniform
```

sbx prompts for the key and stores it in its secret store (the OS keychain).

## 3. Add the project ID to your .sbxenv.yaml

The project ID is not a secret. Pass it to the sandbox in the `env:` block —
never as a project `.env` file, which the kit's env guard removes:

```yaml
env:
  UNIFORM_PROJECT_ID: <project-id>
```

## 4. Approve the credential binding

The first time you create a sandbox, sbx asks you to approve that the
`uniform` key may be injected for the `uniform.app` / `uniform.global` hosts
— approve the prompt and you're done.

## Verify

In the sandbox, run `npx uniform whoami`. It reports the masked API key, the
project, and the API hosts the configuration resolves to. Frontend apps
authenticated via the SDK (`UNIFORM_API_KEY` + `UNIFORM_PROJECT_ID`) now
work without the real key ever entering the sandbox.

## Without a key

The docs ([docs.uniform.app](https://docs.uniform.app/docs)) stay reachable
for research; API calls fail with 401 until you register a key.

## EU region teams

Also set these in your `.sbxenv.yaml` `env:` block:

```yaml
env:
  UNIFORM_PROJECT_ID: <project-id>
  UNIFORM_CLI_BASE_URL: https://eu.uniform.app
  UNIFORM_CLI_BASE_EDGE_URL: https://eu.uniform.global
```
