# Versioning

A single SemVer version for the whole repo, managed by [release-please](https://github.com/googleapis/release-please):

- `.release-please-manifest.json`, the current version
- `.release-please-config.json`, the release config (`extra-files` lists everything a release rewrites)
- `CHANGELOG.md`, generated in the release PR

## Release flow

1. Conventional commits land on `main` (see [commit-messages.md](commit-messages.md)).
2. The release-please workflow opens a release PR that bumps:
   - `CHANGELOG.md`
   - the `&ref=vX.Y.Z` git pins in `examples/*.sbxenv.yaml`
3. Merging the release PR tags `vX.Y.Z` and publishes the GitHub release.

## Preview releases

Pre-release versions (`v3.1.0-beta001`, GitHub prereleases) cut on demand from
`main`, for testing the examples before a stable release. Nothing runs
automatically: preview release PRs are only opened when you run the workflow.

1. Actions → **Preview Release** → **Run workflow**. This opens a preview
   release PR (same files as a stable release PR, plus a `-beta001` version)
   and queues it for auto-merge (needs the repo setting "Allow auto-merge").
2. Merging tags the prerelease and publishes the GitHub
   prerelease (not marked latest).

Previews count up zero-padded: `beta001` → `beta002` → … Repeated dispatches
before merging just update the same release PR; a merged preview bumps the
number only on the next dispatch after the previous preview tagged.

Files (kept apart from the stable flow — own manifest, component, labels, and
`release-please--branches--main--components--preview` PR branch):

- `.release-please-preview-config.json`, `.release-please-preview-manifest.json`
- `.github/workflows/preview-release.yml`

Preview releases don't consume conventional commits: the next stable release
PR releases the same commits again as a full version. A `feat!:` after a
preview still bumps the stable major from the last stable tag.

Caveats:

- Both release PRs rewrite the same `x-release-please` lines, so don't cut a
  preview while a stable release PR is open. If one landed anyway, click
  **Update branch** on the stale PR and resolve the pins to the full version
  (and keep both CHANGELOG sections).
- Never hand-edit the preview manifest
  (`.release-please-preview-manifest.json`); preview release PRs own it, same
  as the stable one.

No images are published from this repo. The kit runs on Docker's built-in
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

- Never bump pinned versions by hand. The release PR owns every `x-release-please` block.
- `kit/` (the base kit) and `sbxenv.yaml` (dev env) are never bumped.

## Adding a stack

No kit needed, because stacks are mixins:

1. A mixin directory `mixins/<stack>/` with a `spec.yaml` (copy an
   existing stack mixin, e.g. `mixins/node/`).
2. An example in `examples/` composing the base kit plus the stack mixin.
