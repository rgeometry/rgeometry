# https://keepachangelog.com/en/1.0.0/

## [Unreleased]
### Added
### Changed

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
