# https://keepachangelog.com/en/1.0.0/

## [Unreleased]

### Internal
- [#169](https://github.com/rgeometry/rgeometry/pull/169) Improved coverage report formatting with range notation and filtering
- [#168](https://github.com/rgeometry/rgeometry/pull/168) Added Python linting with Ruff to flake checks
- [#167](https://github.com/rgeometry/rgeometry/pull/167) Added Nix linting with statix and deadnix to flake checks

## [0.10.4] 2025-11-03

### Added
- [#137](https://github.com/rgeometry/rgeometry/pull/137) Bentley-Ottmann sweep algorithm for line segment intersection

### Changed
- [#139](https://github.com/rgeometry/rgeometry/pull/139) Rust edition updated to 2024
- [#136](https://github.com/rgeometry/rgeometry/pull/136) Bumped rug from 1.16.0 to 1.28.0

## [0.10.3] 2025-08-16

### Added
- [#116](https://github.com/rgeometry/rgeometry/pull/116) Gift wrapping algorithm for finding convex hull

### Fixed
- [#129](https://github.com/rgeometry/rgeometry/pull/129) Fix bug in melkman algorithm.
- [#128](https://github.com/rgeometry/rgeometry/pull/128) Fix arbitrary impls for f32 and f64 polygons.
- [#120](https://github.com/rgeometry/rgeometry/pull/120) Avoid panic when computing line intersections.

## [0.10.2] 2025-01-03

### Changed
- Support no-std environments.

## [0.10.1] 2024-11-28

### Added
- `Polygon::hash`.
### Changed
- Drop `copy` constraint for random convex polygons.

## [0.10.0] 2024-10-26

### Added
- Conversion from LineSegment to LineSegmentView.
- Convex hull calculation using Melkman's algorithm.
### Changed
- Bumped dependencies.
### Fixed
- Fix crash in `get_visibility_polygon` for `Point<f64>`.

## [0.9.0] 2022-08-13

### Added
- `Polygon::equals`
### Changed
- Re-arranged module layout for polygonization.
- Added support for using `f64` and `f32`.
- Improved fuzzy testing.

## [0.8.1] 2022-05-23
### Changed
- Fixed bug in 'squared_magnitude'.

## [0.8.0] 2022-05-23
### Added
- Generator for star-shaped polygons in O(n log n).
- Naive O(n^2) visibility algorithm.
### Changed
- MSRV is now 1.59
- f32 and f64 now uses exact math.
- Updated ordered-float dependency.
- Fixed panic in Polygon::new.

## [0.7.0] 2021-07-15
### Added
- Cursors can be deref'ed to Points for convenience.
- Add implementation agnostic Polygon::triangulate method.
- Generation of monotone polygons (Thanks to Omar Bayomy).
- Basic code for using the "Simulation of Simplicity" strategy.
- Polygon::locate using SoS to deal with degenerate cases.
- Support rug::Integer (GMP).

### Changed
- Set display defaults for the wasm playground.
- Simplify the range operator for cursors.
- Improve performance when using BigInt and BigRational.
- Prevent overflows in 'cmp_distance_to'.

## [0.6.0] - 2021-07-05
### Added
- Orientation along vector.

### Changed
- Fix bug in two-opt polygonization.
- Simplify proptest infrastructure.
- Improve website performance and caching (cloudflare).
- Improve documentation with more interactive examples.

## [0.5.0] - 2021-07-01
### Added
- Triangulation by earclipping with z-order hashing.

### Changed
- Generate test polygons with fixed precision number types.
- Improve documentation for PolygonConvex::random.

## [0.4.0] - 2021-06-26
### Added
- Two-opt polygonization.
- Naive O(n^2) line-segment intersections.
- Support fixed precision coordinates (i8, i16, i32, i64).

### Changed
- Fix playground pixel density on mobile.
- Detect invalid polygons with duplicate vertices.
- Wrap 'VertexId' in a newtype.
- Fix line intersection bug.

## [0.3.0] - 2021-05-30
### Added
- Playground website.

### Changed
- Switch from dual license to a single license.
- Fix bug in convex hull implementation.
- Improve convex hull documentation.
- Enable Katex in documentation.
- Rename ConvexPolygon to PolygonConvex.

## [0.2.1] - 2021-05-25
### Changed
- Check doctests even when they depend on wasm.

## [0.2.0] - 2021-05-24
### Added
- Triangles.
- Wasm support.
- Property tests.

## [0.1.0] - 2021-05-21
### Added
- Basic project structure
