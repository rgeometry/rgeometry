# AGENTS.md - Guide for Agentic Coding

This file provides comprehensive guidance for AI agents (like Claude Code) when working with code in this repository.

## Project Overview

RGeometry is a computational geometry library in Rust providing data types (points, polygons, lines, segments) and algorithms for manipulating them. The library emphasizes correctness through exact arithmetic and supports multiple numeric types from fixed-precision integers to arbitrary-precision rationals.

## Build, Test & Lint

### Pre-commit Hook (Primary Workflow)

**CRITICAL**: The repository uses a Nix-based pre-commit hook that automatically runs ALL validation checks on commit. This is the primary workflow - all other manual commands are optional.

```bash
# All validation happens automatically when you commit
git add . && git commit -m "feat: your message"
```

**IMPORTANT**: You MUST run validation before committing:
- **ALWAYS** let the pre-commit hook run `nix flake check` automatically
- **NEVER** use `git commit --no-verify` or `--no-verify` flag
- **DO NOT** bypass the pre-commit hook under any circumstances
- If checks fail, fix the issues and commit again - the hook ensures all commits pass CI

The pre-commit hook runs:
- `nix flake check` - Runs tests, clippy, formatting checks, and builds all demos
- `nix run .#pre-commit` - Validates Nix, TOML, and Rust formatting

### Manual Commands (for development/debugging)

These are useful during development but are **not required** - the pre-commit hook handles validation:

```bash
# Run all Nix checks (same as pre-commit hook)
nix flake check

# Run tests manually
cargo test

# Run tests for a specific module
cargo test <module_name>

# Run a specific test
cargo test <test_name>

# Run single module with one thread
cargo test <module_name> -- --test-threads=1

# Format code manually
cargo fmt --all

# Lint code manually
cargo clippy -- -D warnings

# Check WASM compatibility
cargo check --target wasm32-unknown-unknown
```

### Benchmarks
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench convex_polygon
cargo bench --bench graham_scan
cargo bench --bench two_opt
```

## Code Architecture

### Core Type System

The library uses a trait-based approach to support multiple numeric types:

- **`PolygonScalar` trait** (src/lib.rs:91-114): The foundational trait that enables the library to work with different numeric types (i8, i16, i32, i64, f32, f64, BigInt, BigRational, etc.). All geometric computations are generic over this trait.

- **Numeric precision categories**: The library implements `PolygonScalar` via macros for three precision categories:
  - `fixed_precision!` for integer types (i8, i16, i32, i64, isize)
  - `arbitrary_precision!` for BigInt and BigRational
  - `floating_precision!` and `wrapped_floating_precision!` for floating-point types

- **`TotalOrd` trait** (src/lib.rs:51-67): Provides total ordering for types, including special handling for floating-point types.

### Module Structure

- **`src/data/`**: Core geometric data types
  - `point.rs`: Point type with dimension-generic support
  - `vector.rs`: Vector type with arithmetic operations
  - `polygon.rs`: Polygon representation with support for simple and multi-ring polygons
  - `line.rs`, `line_segment.rs`: Line primitives
  - `triangle.rs`: Triangle type and operations
  - `polygon/convex.rs`: Specialized convex polygon type with additional invariants

- **`src/algorithms/`**: Computational geometry algorithms
  - `convex_hull/`: Graham scan, gift wrapping, and Melkman's algorithms
  - `triangulation/`: Ear clipping triangulation
  - `polygonization/`: Converting point sets to polygons (monotone, star, two-opt)
  - `intersection/`: Line segment intersection algorithms
  - `visibility/`: Visibility polygon computation
  - `zhash.rs`: Z-order curve hashing for spatial indexing

- **`src/orientation.rs`**: Robust orientation predicates (left/right/collinear tests)

- **`src/intersection.rs`**: `Intersects` trait for testing geometric intersections

### Key Design Patterns

1. **Indexed geometry**: Polygons store points in a flat `Vec` and reference them via `PointId` indices. This enables efficient storage and manipulation.

2. **Zero-cost abstractions**: The library uses iterators and views (e.g., `VectorView`) to avoid unnecessary allocations.

3. **Exact predicates**: For floating-point types, the library uses the `geometry_predicates` crate for robust orientation tests (see `cmp_slope` implementations in src/lib.rs).

4. **Error handling**: The `Error` enum (src/lib.rs:25-33) represents validation failures like insufficient vertices, self-intersections, or convexity violations.

## Code Style Guidelines

### Formatting & Imports
- 2-space indentation (rustfmt.toml: `tab_spaces = 2`)
- Use `use` statements at top; organize standard library, dependencies, then relative imports
- Imports from `std` and `num_traits` typically come first
- All code must pass `cargo fmt --all --check`

### Types & Generics
- Leverage generic `PolygonScalar` trait for numeric types (i8, i16, i32, i64, f32, f64, BigInt, BigRational)
- Use `TotalOrd` trait for total ordering (handles floating-point edge cases)
- Type parameters constrained by trait bounds (e.g., `T: PolygonScalar + TotalOrd`)

### Naming Conventions
- Types: `PascalCase` (Point, Vector, Polygon, ConvexPolygon)
- Functions/methods: `snake_case` (calculate_area, is_convex, get_intersection)
- Module names: `snake_case` (data, algorithms, orientation)
- Constants: `SCREAMING_SNAKE_CASE`

### Error Handling
- Return `Result<T, Error>` where `Error` enum is defined in src/lib.rs (InsufficientVertices, SelfIntersections, DuplicatePoints, ConvexViolation, ClockWiseViolation, CoLinearViolation)
- Use `?` operator for propagating errors in Result chains
- Validate geometry invariants (convexity, self-intersections) at construction boundaries

### Testing & Documentation
- Write doctests in doc comments (format: `/// # Example` followed by `/// ```rust` blocks)
- Property-based tests use `proptest` crate; regressions stored in `proptest-regressions/`
- All public items require documentation
- Use `#[cfg(test)]` for test-only code; add `#[tarpaulin_include]` to cover test helpers

### Clippy & Strict Checks
- Zero clippy warnings allowed: `cargo clippy -- -D warnings`
- Deny lossy float literals: `#![deny(clippy::lossy_float_literal)]`
- Deny float precision loss: `#![deny(clippy::cast_precision_loss)]`
- WASM compatibility: `cargo check --target wasm32-unknown-unknown` must pass

### MSRV
Minimum Supported Rust Version: 1.90

## Development Workflow

### For New Features

1. **Create a feature branch**: Branch from upstream main (`origin/main`)
   ```bash
   git checkout -b feat/your-feature-name origin/main
   ```

2. **Open a draft PR**: Create a draft PR with a Conventional Commits title and an extremely short description
   ```bash
   gh pr create --draft --title "feat: your feature" --body "Brief description"
   ```

3. **Implement changes**: Make your code changes

   **IMPORTANT**: Keep `CHANGELOG.md` up-to-date as you work. The CHANGELOG refers to PR numbers, so it can only be updated after a PR has been opened. Add entries under the "Unreleased" section documenting any user-visible changes.

4. **Commit changes**: Commit your work - the pre-commit hook automatically validates everything
   ```bash
   git add . && git commit -m "feat: your commit message"
   ```
   - The pre-commit hook runs `nix flake check` which executes tests, clippy, formatting, and all CI checks
   - If checks fail, the commit will be rejected - fix the issues and try again
   - **Do not manually run** `cargo test`, `cargo fmt`, or `cargo clippy` - the hook handles this
   - **NEVER use `--no-verify`** - the hook must always run
   - Iterate until the commit succeeds (all checks pass)

5. **Push to remote when ready**:
   ```bash
   git push -u origin feat/your-feature-name
   ```

6. **Finalize PR**: Write the final PR description and mark as ready for review
   ```bash
   gh pr ready
   ```

### GitHub CLI Commands
```bash
gh pr create --draft --title "feat: description" --body "PR description"  # Create draft PR
gh pr ready                                                                # Mark PR ready for review
gh pr view                                                                 # View current PR
```

## Commit and PR Conventions

### Commit Messages
- Follow the Conventional Commits standard with type prefixes:
  - `feat:` for new features
  - `fix:` for bug fixes
  - `test:` for test-related changes
  - `ci:` for CI/CD changes
  - `chore:` for maintenance tasks
  - `docs:` for documentation
  - `refactor:` for code refactoring
- Example: `feat: add visibility polygon algorithm`
- **Do NOT add** "Co-Authored-By" trailers or "Generated with Claude Code" footers

### Pull Requests
- PR titles must also use Conventional Commits tags (e.g., "feat: add visibility polygon algorithm")
- PR descriptions should be short and to-the-point
- Do NOT add "Co-Authored-By" trailers or "Generated with Claude Code" footers to commits or PRs

## Testing Conventions

- Property-based tests use `proptest` and `test-strategy` crates
- Proptest regressions are stored in `proptest-regressions/`
- The `testing` module (src/testing.rs) provides utilities for tests

## CI Requirements

The pre-commit hook ensures all commits automatically pass the same checks as CI. All PRs must pass:
- `cargo test` - All tests including doc tests
- `cargo clippy -- -D warnings` - No clippy warnings allowed
- `cargo fmt --all -- --check` - Code must be formatted
- TOML and Nix formatting checks
- WASM compatibility check
- All demos must build successfully
