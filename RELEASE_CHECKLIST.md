# Release Checklist

This document provides guidance for creating releases of rgeometry.

## Version Number Selection

RGeometry follows [Semantic Versioning](https://semver.org/). Given a version number MAJOR.MINOR.PATCH:

### Patch Release (0.10.X)

Use a patch release for:
- Bug fixes that don't change the public API
- Documentation updates
- Internal refactoring that doesn't affect the API
- Performance improvements without API changes

### Minor Release (0.X.0)

Use a minor release for:
- **New algorithms or features** (backwards-compatible additions)
- **Dependency version bumps** (even if they could theoretically break downstream builds)
- **MSRV (Minimum Supported Rust Version) changes**
- New public functions, methods, or types
- Deprecating functionality (but not removing it)
- Bug fixes that could be considered behavioral changes

### Major Release (X.0.0)

Use a major release for:
- **Any changes to existing public types** (field additions, removals, or modifications)
- **Breaking changes to existing public APIs**
- Removing deprecated functionality
- Changing function signatures
- Changing trait bounds or implementations
- Changing error types or error behavior in significant ways

### Examples

- Adding `Bentley-Ottmann` algorithm → **Minor release** (new feature)
- Bumping `rug` from 1.16.0 to 1.28.0 → **Minor release** (dependency bump)
- Updating Rust edition from 2021 to 2024 → **Minor release** (MSRV change)
- Adding a field to `Point<T>` struct → **Major release** (type change)
- Changing `Polygon::new()` signature → **Major release** (API change)
- Fixing a bug in `melkman` algorithm → **Patch release** (bug fix)

## Release Process

### 1. Prepare the Release

- [ ] Create a branch named `release/X.Y.Z` from `origin/main`
- [ ] Review commits since the last release: `git log vX.Y.Z..HEAD --oneline`
- [ ] Update `CHANGELOG.md`:
  - Move unreleased changes to new version section
  - Add release date
  - Include only user-visible changes (omit CI, build system changes)
  - Format: `## [X.Y.Z] YYYY-MM-DD`
- [ ] Update version in `Cargo.toml`
- [ ] Update `Cargo.lock` to reflect version change: `nix develop -c cargo metadata --format-version=1 > /dev/null`
- [ ] Commit changes: `git commit -m "release: X.Y.Z"`

### 2. Create Pull Request

- [ ] Create a draft PR with title: `release: X.Y.Z`
- [ ] PR description should be brief and note the release type (patch/minor/major)
- [ ] Ensure all CI checks pass
- [ ] Mark PR as ready for review
- [ ] Get approval and merge

### 3. Create Release (Manual - Done by Maintainer)

After the release PR is merged:

- [ ] Tag the release: `git tag vX.Y.Z`
- [ ] Push the tag: `git push origin vX.Y.Z`
- [ ] The CI workflow will automatically publish to crates.io
- [ ] Create GitHub release from the tag with CHANGELOG excerpt

## Notes

- **DO NOT** manually publish to crates.io - the CI workflow handles this
- **DO NOT** add "Co-Authored-By" or "Generated with Claude Code" footers
- The pre-commit hook will automatically validate all changes
- All commits must pass the same checks as CI
