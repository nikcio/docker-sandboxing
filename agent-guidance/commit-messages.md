# Commit Messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/) style:

```
type(scope): description
```

## Types

| Type | Use |
|------|-----|
| `feat` | New feature (minor release) |
| `fix` | Bug fix (patch release) |
| `docs` | Documentation only |
| `chore` | Maintenance, deps, release |
| `ci` | CI/CD changes |
| `test` | Test changes |

`feat`, `fix`, and `deps` trigger a release-please release; other types land
without one. Add `!` after the type (or a `BREAKING CHANGE:` footer in the
body) for a major release:

```
feat(kit)!: change entrypoint contract
```

## Scopes

Common scopes: `kit`, `template`, `mixins`, `scripts`, `release`, `docs`, `deps`. Omit the scope when the change doesn't fit one.

## Examples

```
feat(kit): auto-bump published image tag with the release
fix(template): pin NVM install to a verified checksum
docs: update README consumption flow
ci: verify kit references the pushed image tag
```
