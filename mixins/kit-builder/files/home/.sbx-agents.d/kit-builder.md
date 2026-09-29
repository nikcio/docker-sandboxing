## Kit builder (Docker + buildx)

- The sandbox has a working Docker engine (`dockerd`, logs in
  `/var/log/dockerd.log`) plus the Docker CLI and buildx.
- Build/run/push images from the workspace: `docker build -t <img> .`,
  `docker run --rm <img>`, `docker buildx build ...`.
- The engine store is `/var/lib/docker` (per-sandbox volume; also the
  build cache — `docker builder prune` when the disk gets tight).
- Docker Hub egress is allowed; other registries need their registry
  mixin (ghcr/gcr/mcr).
- Registry auth is not injected: `docker push` to authenticated
  registries needs a token provided another way.
- No nested sandbox VMs — build images and (with the `sbx` mixin)
  validate/pack kits; creating sandboxes stays host-side.
