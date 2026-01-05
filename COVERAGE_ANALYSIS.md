# RGeometry Coverage Analysis & Improvement Plan

## Executive Summary

Current coverage: **51%** (as reported)

The codebase has generally good test coverage with property-based testing (proptest), but has several critical gaps:
- Two entire modules with **zero tests**
- Multiple untested panic/error conditions
- Edge cases not covered
- Some unimplemented code paths

---

## Coverage Gaps by Priority

### 🔴 Critical (No Tests Entirely)

| File | Issue | Impact |
|------|-------|--------|
| `src/algorithms/intersection/naive.rs` | **Entire module untested** | O(n²) intersection algorithm has no validation |
| `src/data/triangle.rs` | **Triangle types untested** | Public API (new, validate, locate, signed_area, rejection_sampling) has no tests |

### 🟠 High Priority (Untested Panic/Error Paths)

| Location | Issue | Test Needed |
|----------|-------|--------------|
| `src/algorithms/triangulation/constrained_delaunay.rs:23` | Panic on holes | Test that panic occurs when rings.len() != 1 |
| `src/algorithms/polygonization/star.rs:23` | Panic on holes | Test that panic occurs when rings.len() != 1 |
| `src/algorithms/polygonization/monotone.rs:15` | Panic on holes | Test that panic occurs when rings.len() != 1 |
| `src/algorithms/polygonization/two_opt.rs:23` | Panic on holes | Test that panic occurs when rings.len() != 1 |
| `src/algorithms/triangulation/earclip.rs:17, 33` | Panic on holes | Test that panic occurs when rings.len() != 1 |
| `src/data/polygon.rs:587` | Panic for invalid edge | Test that panic occurs for edges not in polygon |
| `src/data/triangle.rs:~160` | `rejection_sampling` panic | Test the panic condition |

### 🟡 Medium Priority (Untested Code Paths)

| Module | Issue | Details |
|--------|-------|---------|
| `src/data/line.rs:50, 55` | Unimplemented method | `DirectionView::Vector` in `intersection_point` |
| `src/data/line.rs:81-84` | Unimplemented method | `Line::intersection_point` |
| `src/algorithms/visibility/naive.rs` | Boundary conditions | Point on vertex, point on edge, complex obstacles |
| `src/algorithms/kernel/naive.rs` | Edge cases | Degenerate polygons, overflow scenarios |
| `src/data/polygon.rs` | Edge cases | `centroid` with degenerate polygons, `bounding_box` with single point |
| `src/algorithms/triangulation/earclip.rs:70` | Panic on degenerate | "no valid ears found" panic not tested |

### 🟢 Low Priority (Could Improve)

- Colinear violation errors not explicitly tested
- Very large/small input sizes for property tests
- Edge cases around boundary conditions in orientation predicates
- f64 and u64 zhash implementations less thoroughly tested

---

## Improvement Plan

The goal is to increase coverage by either:
1. Adding tests (preferably property tests)
2. Removing dead code

### Phase 1: Critical Coverage (Target: +15-20%)

**Step 1:** Test `src/algorithms/intersection/naive.rs`
- Add unit tests for basic intersection scenarios
- Add property tests comparing to bentley_ottmann results
- Test edge cases (overlapping segments, shared endpoints, collinear)

**Step 2:** Test `src/data/triangle.rs`
- Add unit tests for Triangle and TriangleView
- Test `new`, `validate`, `locate`, `signed_area`
- Test `rejection_sampling` and its panic condition
- Test `bounding_box` method

**Step 3:** Test multi-ring panic conditions
- Add tests verifying panic occurs when rings.len() != 1 for all affected algorithms

### Phase 2: High Priority Edge Cases (Target: +5-8%)

**Step 4:** Test edge cases in visibility and kernel
- Point on vertex, point on edge for visibility
- Degenerate polygons for kernel
- Complex obstacle scenarios

**Step 5:** Test polygon edge cases
- Centroid with degenerate polygons
- Bounding box with single point
- Edge panic condition

**Step 6:** Test remaining panic conditions
- "no valid ears found" panic in earclip
- Other untested error paths

### Phase 3: Dead Code Removal (Target: +2-5%)

**Step 7:** Remove or implement unimplemented code
- Decide whether to implement `Line::intersection_point` or mark it as `unimplemented!`
- Remove any truly dead code identified

---

## Execution Plan

Each step will:
1. Write the tests
2. Run tests to ensure they pass
3. Measure coverage improvement
4. Commit changes

**Total target coverage improvement: ~22-33% (from 51% to ~73-84%)**

---

## Testing Strategy

### Property Tests (Preferred)
- Use `proptest` for algorithms with mathematical properties
- Test invariants (e.g., hull is convex, triangulation preserves area)

### Unit Tests (Acceptable)
- Use for specific edge cases or error conditions
- Must be readable and well-documented
- Focus on readability and clarity

### Integration Tests
- Test interactions between modules
- Test algorithm correctness with known inputs/outputs
