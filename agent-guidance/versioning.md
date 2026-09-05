# Versioning

A single SemVer version for the whole repo, managed by
[release-please](https://github.com/googleapis/release-please):

- `.release-please-manifest.json` — current version
- `.release-please-config.json` — release config (`extra-files` lists everything a release rewrites)
- `CHANGELOG.md` — generated in the release PR

## Release flow

1. Conventional commits land on `main` (see [commit-messages.md](commit-messages.md)).
2. The release-please workflow runs as a GitHub App (repo secrets
   `APP_CLIENT_ID` + `APP_PRIVATE_KEY`; the App needs Contents and Pull
   requests read/write) and opens a release PR that bumps:
   - `CHANGELOG.md`
   - `version` + the `sandbox.image` tag in `kit-published/spec.yaml`
   - the `&ref=vX.Y.Z` git pins in `examples/opencode-node-dotnet.sbxenv.yaml`
3. Merging the release PR tags `vX.Y.Z` and publishes the GitHub release.
4. The `publish-image.yml` workflow builds every image in `images.json` and
   pushes `docker.io/nikcio/<name>:vX.Y.Z` (plus `:latest`) as public
   images on Docker Hub.

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
- `kit/spec.yaml` (dev kit) and `.sbxenv.yaml` (dev env) are never bumped.
- `kit-published/` must stay in sync with `kit/` — the only intended
  difference is the published `sandbox.image` (see the header of
  `kit-published/spec.yaml`).

## Adding an image

1. Dockerfile in a new directory (e.g. `template-python/`).
2. A published kit spec pinning `docker.io/nikcio/<name>:vX.Y.Z` inside an
   `x-release-please` block (copy `kit-published/`).
3. An entry in `images.json` and the spec in
   `.release-please-config.json` → `extra-files`.
