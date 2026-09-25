# Versioning

A single SemVer version for the whole repo, managed by
[release-please](https://github.com/googleapis/release-please):

- `.release-please-manifest.json` — current version
- `.release-please-config.json` — release config (`extra-files` lists everything a release rewrites)
- `CHANGELOG.md` — generated in the release PR

## Release flow

1. Conventional commits land on `main` (see [commit-messages.md](commit-messages.md)).
2. The release-please workflow opens a release PR that bumps:
   - `CHANGELOG.md`
   - `version` in `kit-<stack>/kit-<stack>.yaml` and the `FROM` image tag
     in `kit-<stack>/kit-<stack>.dockerfile` (the published workload image)
   - the `docker.io/nikcio/...:vX.Y.Z` refs in `examples/*.sbxenv.yaml`
     (workload `agent:` + mixin `kits:` lines)
3. Merging the release PR tags `vX.Y.Z` and publishes the GitHub release.
4. The `publish-image.yml` workflow builds every image in `images.json` and
   pushes `docker.io/nikcio/<name>:vX.Y.Z` (plus `:latest`) as public
   images on Docker Hub — template images from their Dockerfiles, kit and
   mixin images from their v3 descriptors.

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
- `kit/` (the base kit) and `sbxenv.yaml` (dev env) are never bumped.
- `kit-<stack>/` all share the same shape — the only intended differences
  are the stack `FROM` image and the `displayName:` (see the header of
  `kit-node/kit-node.yaml`).

## Adding an image

1. Dockerfile in a new directory (e.g. `template-python/`).
2. A workload kit pinning `docker.io/nikcio/<name>:vX.Y.Z` inside an
   `x-release-please` block (copy `kit-python/` or `kit-node/`).
3. An entry in `images.json`, the kit files in
   `.release-please-config.json` → `extra-files`, and the check names in
   `docs/repo-setup.md` → required status checks.
