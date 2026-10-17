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
   - `version` in `kit-<stack>/spec.yaml`
   - the `&ref=vX.Y.Z` git pins in `examples/*.sbxenv.yaml`
3. Merging the release PR tags `vX.Y.Z` and publishes the GitHub release.

No images are published from this repo: the kits run on Docker's built-in
`docker/sandbox-templates:opencode-docker` image, and stack toolchains are
installed by the mixins at sandbox creation.

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
  are the `name:`/`displayName:` and the version block (see the header of
  `kit-node/spec.yaml`).

## Adding a kit

1. A directory with a `spec.yaml` (copy `kit-node/`) running on the
   built-in `docker/sandbox-templates:opencode-docker` image.
2. An entry in `.release-please-config.json` → `extra-files` so the
   release bumps its `x-release-please` version block.
3. An example in `examples/` composing the kit plus the stack mixin.
