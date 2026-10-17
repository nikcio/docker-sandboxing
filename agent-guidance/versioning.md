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
   - the `&ref=vX.Y.Z` git pins in `examples/*.sbxenv.yaml`
3. Merging the release PR tags `vX.Y.Z` and publishes the GitHub release.

No images are published from this repo: the kit runs on Docker's built-in
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

## Adding a stack

No kit needed — stacks are mixins:

1. A mixin directory `mixins/<stack>/` with a `spec.yaml` (copy an
   existing stack mixin, e.g. `mixins/node/`).
2. An example in `examples/` composing the base kit plus the stack mixin.
