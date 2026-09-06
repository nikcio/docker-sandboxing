# Set your Zeldoc API key

The sandbox's model provider is [Zeldoc.ai](https://zeldoc.ai). The API key
is registered on your host only — the sandbox never sees it. A proxy injects
it into requests to `api.zeldoc.ai`.

## 1. Get a key

Follow the [Zeldoc connect guide](https://docs.zeldoc.ai/connect-opencode) to
create an API key.

## 2. Register it with sbx

```bash
sbx secret set zeldoc
```

sbx prompts for the key and stores it in its secret store (the OS keychain).

## 3. Approve the credential binding

The first time you create a sandbox, sbx asks you to approve that the
`zeldoc` key may be injected for `api.zeldoc.ai` — approve the prompt and
you're done.

## Verify

Start the sandbox (`sbx env run`); opencode should start with the model
`zeldoc/zdev-2`.

If ZDev ever reports "encountered an error", the model's context limit
likely changed — update `"limit".context` in
`mixins/zeldoc/files/home/.config/opencode/providers.d/20-zeldoc.json`
per the [Zeldoc guide](https://docs.zeldoc.ai/connect-opencode)
(currently `1000000`).

Prefer your GitHub Copilot subscription instead? See
[copilot-setup.md](copilot-setup.md) — the two mixins compose (both
providers stay enabled).
