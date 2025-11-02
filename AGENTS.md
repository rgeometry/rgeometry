# AGENTS.md - Guide for Agentic Coding

## Build, Test & Lint

**Primary workflow** (automatically validates on commit):
```bash
git add . && git commit -m "feat: message"  # Pre-commit hook runs all checks
```

**Manual commands** (for development):
```bash
cargo test                           # Run all tests
cargo test <test_name>               # Run specific test
cargo test <module_name> -- --test-threads=1  # Run single module
cargo clippy -- -D warnings          # Lint (no warnings allowed)
cargo fmt --all                      # Format code
nix flake check                      # Full CI validation locally
```

## Code Style Guidelines

**Formatting & Imports**
- 2-space indentation (rustfmt.toml: `tab_spaces = 2`)
- Use `use` statements at top; organize standard library, dependencies, then relative imports
- Imports from `std` and `num_traits` typically come first
- All code must pass `cargo fmt --all --check`

**Types & Generics**
- Leverage generic `PolygonScalar` trait for numeric types (i8, i16, i32, i64, f32, f64, BigInt, BigRational)
- Use `TotalOrd` trait for total ordering (handles floating-point edge cases)
- Type parameters constrained by trait bounds (e.g., `T: PolygonScalar + TotalOrd`)

**Naming Conventions**
- Types: `PascalCase` (Point, Vector, Polygon, ConvexPolygon)
- Functions/methods: `snake_case` (calculate_area, is_convex, get_intersection)
- Module names: `snake_case` (data, algorithms, orientation)
- Constants: `SCREAMING_SNAKE_CASE`

**Error Handling**
- Return `Result<T, Error>` where `Error` enum is defined in src/lib.rs (InsufficientVertices, SelfIntersections, DuplicatePoints, ConvexViolation, ClockWiseViolation, CoLinearViolation)
- Use `?` operator for propagating errors in Result chains
- Validate geometry invariants (convexity, self-intersections) at construction boundaries

**Testing & Documentation**
- Write doctests in doc comments (format: `/// # Example` followed by `/// ```rust` blocks)
- Property-based tests use `proptest` crate; regressions stored in `proptest-regressions/`
- All public items require documentation
- Use `#[cfg(test)]` for test-only code; add `#[tarpaulin_include]` to cover test helpers

**Clippy & Strict Checks**
- Zero clippy warnings allowed: `cargo clippy -- -D warnings`
- Deny lossy float literals: `#![deny(clippy::lossy_float_literal)]`
- Deny float precision loss: `#![deny(clippy::cast_precision_loss)]`
- WASM compatibility: `cargo check --target wasm32-unknown-unknown` must pass

**Commit Messages**
- Use Conventional Commits: `feat:`, `fix:`, `test:`, `refactor:`, `docs:`, `ci:`, `chore:`
- Example: `feat: add visibility polygon algorithm`
- Do NOT add "Co-Authored-By" trailers or "Generated with Claude Code" footers

**MSRV**: Minimum Supported Rust Version is 1.59

## Feature Development Workflow

**For new features:**
1. Create a feature branch from `origin/main`: `git checkout -b feat/your-feature-name origin/main`
2. Implement your changes and commit: `git add . && git commit -m "feat: description"`
3. Open a PR using `gh` CLI: `gh pr create --title "feat: description" --body "Brief description"`
4. Push to remote when ready: `git push -u origin feat/your-feature-name`

**GitHub CLI commands:**
```bash
gh pr create --title "feat: description" --body "PR description"  # Create PR
gh pr ready                                                        # Mark PR ready for review
gh pr view                                                         # View current PR
```
