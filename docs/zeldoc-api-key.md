# Set your Zeldoc API key

The sandbox's model provider is [Zeldoc.ai](https://zeldoc.ai). The API key is registered on your host only — the sandbox never sees it. A proxy injects it into requests to `api.zeldoc.ai`.

## 1. Get a key

Follow the [Zeldoc connect guide](https://docs.zeldoc.ai/connect-opencode) to create an API key.

## 2. Register it with sbx

```bash
sbx secret set zeldoc
```

sbx prompts for the key and stores it in its secret store (the OS keychain).

## 3. Approve the credential binding

The first time you create a sandbox, sbx asks you to approve that the `zeldoc` key may be injected for `api.zeldoc.ai` — approve the prompt and you're done.

## Verify

Start the sandbox (`sbx env run`); opencode starts with the Zeldoc provider enabled. No default model is picked for you — set `"model": "zeldoc/zdev"` in a project-level `opencode.jsonc` (see [project-kit.md](project-kit.md)) or pick it with `/models`.

The model picker lists exactly the models your key can use: the [`opencode-zeldoc` plugin](https://github.com/martinmose/opencode-zeldoc) asks the API which models your key has at every start (limits, prices, and capabilities come from Zeldoc.ai, so nothing needs manual updating). If Zeldoc.ai can't be reached at start, opencode falls back to its built-in catalog until the next successful start.

Prefer your GitHub Copilot subscription instead? See [copilot-setup.md](copilot-setup.md) — the two mixins compose (both providers stay enabled).
