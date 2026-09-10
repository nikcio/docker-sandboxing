# OpenCode

- OpenCode was updated to the newest npm release when the sandbox was created (the template image's
  baked version may be older). Update it yourself with
  `sudo npm install -g --prefix "$(dirname "$(dirname "$(command -v opencode)")")" opencode-ai@latest`
  (runs as agent via sudo, like the create-time update).
- Do NOT use `opencode upgrade`: it resolves "latest" via api.github.com, which the sandbox proxy
  blocks (HTTP 403), and can hang on an invisible stdin prompt.
