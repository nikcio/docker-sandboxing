# env-guard (v3)

The workspace `.env` guard as a `com.docker.sandbox/lifecycle@1` startup
hook. v3 port of [`../env-guard`](../env-guard) — read that for the full
behavior story (what counts as a .env, clone vs direct mode); see
[`../README.md`](../README.md) for what the v3 format changes.

Declaration-only: no image content, no build args, no network rules.
