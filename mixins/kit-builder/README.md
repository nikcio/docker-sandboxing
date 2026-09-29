# kit-builder

Turns the sandbox into a Docker **build sandbox**: guarantees the Docker
CLI, buildx, and a running engine so the agent can build, run, and push
container images from inside the sandbox — including the sandbox template
images this repo publishes (`template-*/Dockerfile`).

## Usage

Add the mixin to the `kits:` list in your project's `sbxenv.yaml`
(pin `&ref=<tag>` to a release, as the [examples](../../examples) do):

```yaml
kits:
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/kit-builder   # Docker + buildx build sandbox
```

For local development, point `--kit` at the directory instead:
`sbx run --kit ./mixins/kit-builder <agent> .`

Compose it with the kit-authoring mixins to let the agent work on sandbox
kits end to end:

```yaml
kits:
  # ...the other stock mixins...
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/sbx          # the sbx CLI (validate/inspect/pack)
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/kit-builder  # Docker + buildx
  - git+https://github.com/nikcio/docker-sandboxing.git#dir=mixins/ghcr         # registry egress (pick yours)
```

## How it works

- **The engine is already there on the stock templates.** The
  `docker/sandbox-templates:opencode-*` images carry the
  `com.docker.sandboxes.start-docker=true` label, so the runtime starts
  `dockerd` inside the sandbox automatically and mounts the engine store
  at `/var/lib/docker` (sized by the host's `sandbox.disk.dockerVolume`
  setting, default 10g). Images built from those templates keep the label.
- **Check-and-install** — on a template without the Docker CLI or buildx,
  the install step pulls `docker-ce-cli` + `docker-buildx-plugin` from
  Docker's apt repo (isolated apt list directory, same pattern as the
  other mixins). Skipped when both are already present. The apt repo
  stays configured so `apt upgrade` keeps them current.
- **Engine guard** — a background startup step starts `dockerd` when the
  runtime has not (base images without the start-docker label). It is a
  no-op on the stock templates. Logs land in `/var/log/dockerd.log`.
- **Registry egress included** — Docker Hub (registry API, auth, blob
  CDN) is allowed out of the box so `docker pull`/`build`/`push` works
  against Docker Hub without composing `docker-hub`. For other
  registries, compose `ghcr`, `gcr`, or `mcr`.

## What the agent can do

```console
docker build -t my-image .                      # build inside the sandbox
docker run --rm my-image                        # run containers
docker buildx build --platform linux/amd64 .    # buildx builds
docker push my-image                            # push (needs registry auth — see below)
```

## Notes & limits

- **Registry auth** is not part of the mixin: `docker login` state is
  host-side and is not copied into sandboxes. Pushes to authenticated
  registries need a token — e.g. store a PAT with `sbx secret set
  github` for GHCR flows that read it, or have the agent use a
  short-lived token you provide another way.
- **No nested sandbox VMs.** The sandbox can build and run *containers*,
  not new sandbox VMs. Creating sandboxes stays a host-side (`sbx env
  run`) or cloud-side (`sbx --cloud`, requires `sbx login`) operation —
  the sandboxed agent can, however, build and push the *images* a kit's
  `sandbox.image` points at, then (via the `sbx` mixin) validate and
  pack kits that reference them.
- The engine store under `/var/lib/docker` is a per-sandbox volume: it
  persists across sandbox stop/start but is removed with the sandbox.
- Build cache lives in the same store; `docker builder prune` reclaims
  it when the disk gets tight.

## Network domains

| Domain | Why |
| ------ | --- |
| `download.docker.com:443` | Docker apt repo: CLI/buildx packages + keyring, and upgrades |
| `docker.io`, `*.docker.io` | Registry API + image manifests (pull/push) |
| `registry-1.docker.io:443` | Docker Hub registry endpoint used by the engine |
| `auth.docker.io:443` | Registry auth/token endpoint |
| `*.docker.com` | Registry auth/token endpoints |
| `production.cloudflare.docker.com` | Blob/layer downloads (CDN) |
| `production.cloudfront.docker.com` | Blob/layer downloads (CDN fallback) |
