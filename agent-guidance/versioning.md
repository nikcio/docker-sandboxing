# Versioning

A single SemVer version for the whole repo, managed by
[release-please](https://github.com/googleapis/release-please):

- `.release-please-manifest.json` — current version
- - `.release-please-config.json` — release config (`extra-files` lists
  everything a release rewrites)
- `CHANGELOG.md` — generated in the release PR

## Release flow

1. 1. Conventional commits land on `main` (see
  [commit-messages.md](commit-messages.md)).
2. The release-please workflow opens a release PR that bumps:
   - `CHANGELOG.md`
   - `version` in `kit-<stack>/kit-<stack>.yaml`
   - the `docker.io/nikcio/...:vX.Y.Z` refs in `examples/*.sbxenv.yaml`
     (workload `agent:` + mixin `kits:` lines)
3. Merging the release PR tags `vX.Y.Z` and publishes the GitHub release.
4. The `publish-image.yml` workflow builds every image in `images.json` and
   pushes `docker.io/nikcio/<name>:vX.Y.Z` (plus `:latest`) as public
   images on Docker Hub — every one from its v3 descriptor (the
   `kit-*/kit-*.dockerfile` recipes bake the stack toolchain directly).

## Version bumps

| Commit | Bump |
|--------|------|
| `fix:`, `deps:` | patch |
| `feat:` | minor |
| `feat!:` / `BREAKING CHANGE:` footer | major |
| `chore:`, `docs:`, `ci:`, `test:` | no release |

## Rules

- Never bump pinned versions by hand — the release PR owns every
  `x-release-please` block.
- `sbxenv.yaml` (dev env) is never bumped.
- `kit-<stack>/` all share the same shape — the only intended differences
  are the stack toolchain in the Dockerfile and the `displayName:` (see
  the header of `kit-node/kit-node.yaml`).

## Adding an image

1. A workload kit directory (e.g. `kit-python/`): descriptor + Dockerfile
   building on `docker/sandbox-templates:shell` (copy an existing kit).
2. An entry in `images.json`, the kit files in
   `.release-please-config.json` → `extra-files`, and the check names in
   `docs/repo-setup.md` → required status checks.
