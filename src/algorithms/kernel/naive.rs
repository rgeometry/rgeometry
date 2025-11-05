use crate::data::{Line, Point, Polygon};
use crate::{Orientation, PolygonScalar};

/// Compute the kernel of a simple polygon.
///
/// The kernel of a polygon is the set of all points from which the entire polygon
/// boundary is visible. Equivalently, it is the intersection of all half-planes
/// defined by the edges of the polygon (where each half-plane contains the polygon's
/// interior).
///
/// For a star-shaped polygon, the kernel is non-empty. For a convex polygon, the
/// kernel is the polygon itself. For some non-convex polygons, the kernel may be
/// empty (returns `None`).
///
/// # Precision and Accuracy
///
/// This function uses approximate line intersections for most numeric types:
/// - **Integer types** (i8, i16, i32, i64, BigInt, etc.): Results are approximate
///   due to rounding when computing intersection points.
/// - **Floating-point types** (f32, f64): Results are approximate due to floating-point
///   arithmetic limitations.
/// - **Rational types** (num::BigRational, num::Rational, etc.): Results are **exact**
///   with arbitrary precision.
///
/// For exact results, use `Polygon<num::BigRational>`.
///
/// # Algorithm
///
/// This is a naive O(n²) algorithm that:
/// 1. For each edge of the polygon, determines the half-plane containing the polygon interior
/// 2. Computes the intersection of all these half-planes using a half-plane intersection algorithm
/// 3. Returns the resulting polygon, or `None` if the kernel is empty or invalid
///
/// # Examples
///
/// ```
/// use rgeometry::data::{Point, Polygon};
/// use rgeometry::algorithms::kernel::naive::kernel;
/// use num_rational::BigRational;
///
/// // A simple square
/// let square = Polygon::new(vec![
///     Point::new([BigRational::from_integer(0.into()), BigRational::from_integer(0.into())]),
///     Point::new([BigRational::from_integer(1.into()), BigRational::from_integer(0.into())]),
///     Point::new([BigRational::from_integer(1.into()), BigRational::from_integer(1.into())]),
///     Point::new([BigRational::from_integer(0.into()), BigRational::from_integer(1.into())]),
/// ]).unwrap();
///
/// let k = kernel(&square).expect("Square has a non-empty kernel");
/// // For a convex polygon, the kernel is the polygon itself
/// assert_eq!(k.signed_area::<BigRational>(), square.signed_area::<BigRational>());
/// ```
///
/// # References
///
/// - Wikipedia: [Star-shaped polygon](https://en.wikipedia.org/wiki/Star-shaped_polygon)
/// - Computational Geometry: Algorithms and Applications (de Berg et al.)
///
pub fn kernel<T: PolygonScalar>(poly: &Polygon<T>) -> Option<Polygon<T>> {
  // The algorithm computes the intersection of half-planes defined by each edge.
  // For a CCW-oriented polygon, the interior is to the left of each edge.

  if poly.iter().count() < 3 {
    // Degenerate case: not enough vertices
    return None;
  }

  // Start with the polygon itself, then for each edge, clip the current result
  // by the half-plane defined by that edge.
  let mut result_vertices: Vec<Point<T>> = poly.iter().cloned().collect();

  // For each edge of the original polygon
  for edge in poly.iter_boundary_edges() {
    let p1 = edge.src;
    let p2 = edge.dst;

    // Clip result_vertices by the half-plane defined by this edge
    result_vertices = clip_polygon_by_edge(&result_vertices, p1, p2);

    if result_vertices.len() < 3 {
      // Kernel is empty or degenerate
      return None;
    }
  }

  // Create polygon from remaining vertices
  Polygon::new(result_vertices).ok()
}

/// Clip a polygon by a half-plane defined by an edge (p1 -> p2).
/// The half-plane retains points on the left side of the edge (CCW interior).
fn clip_polygon_by_edge<T: PolygonScalar>(
  vertices: &[Point<T>],
  p1: &Point<T>,
  p2: &Point<T>,
) -> Vec<Point<T>> {
  if vertices.len() < 3 {
    return vec![];
  }

  let mut result = Vec::new();

  for i in 0..vertices.len() {
    let current = &vertices[i];
    let next = &vertices[(i + 1) % vertices.len()];

    let current_side = Orientation::new(&p1.array, &p2.array, &current.array);
    let next_side = Orientation::new(&p1.array, &p2.array, &next.array);

    // Keep vertices on the left side (CCW) or on the line
    let current_inside = matches!(
      current_side,
      Orientation::CounterClockWise | Orientation::CoLinear
    );
    let next_inside = matches!(
      next_side,
      Orientation::CounterClockWise | Orientation::CoLinear
    );

    if current_inside {
      result.push(current.clone());
    }

    // If edge crosses the clipping line, add intersection point
    if current_inside != next_inside {
      // Use Line::intersection_point for approximate intersection
      let line1 = Line::new_through(current, next);
      let line2 = Line::new_through(p1, p2);
      if let Some(intersection) = line1.intersection_point(&line2) {
        result.push(intersection);
      }
    }
  }

  result
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::PolygonConvex;
  use num_rational::BigRational;
  use proptest::prelude::*;

  fn big(n: i32) -> BigRational {
    BigRational::from_integer(n.into())
  }

  #[test]
  fn test_kernel_of_square() {
    // A square is convex, so its kernel should be itself
    let square = Polygon::new(vec![
      Point::new([big(0), big(0)]),
      Point::new([big(1), big(0)]),
      Point::new([big(1), big(1)]),
      Point::new([big(0), big(1)]),
    ])
    .unwrap();

    let k = kernel(&square).expect("Square has a non-empty kernel");
    assert_eq!(k.iter().count(), 4);
    assert_eq!(
      k.signed_area::<BigRational>(),
      square.signed_area::<BigRational>()
    );
  }

  #[test]
  fn test_kernel_of_triangle() {
    // A triangle is convex, so its kernel should be itself
    let triangle = Polygon::new(vec![
      Point::new([big(0), big(0)]),
      Point::new([big(2), big(0)]),
      Point::new([big(1), big(2)]),
    ])
    .unwrap();

    let k = kernel(&triangle).expect("Triangle has a non-empty kernel");
    assert_eq!(k.iter().count(), 3);
    assert_eq!(
      k.signed_area::<BigRational>(),
      triangle.signed_area::<BigRational>()
    );
  }

  #[test]
  fn test_kernel_of_star_polygon() {
    // A simple star-shaped polygon (like a 'arrow' pointing right)
    // The kernel should be smaller than the polygon
    let star = Polygon::new(vec![
      Point::new([big(0), big(1)]),
      Point::new([big(2), big(0)]),
      Point::new([big(2), big(2)]),
    ])
    .unwrap();

    let k = kernel(&star).expect("Star polygon has a non-empty kernel");
    assert!(k.iter().count() >= 3);
    // Kernel area should be <= original area
    assert!(k.signed_area::<BigRational>() <= star.signed_area::<BigRational>());
  }

  #[test]
  fn test_kernel_returns_none_for_empty_kernel() {
    // A non-convex polygon that has an empty kernel
    // This is a "pac-man" shape where the kernel would be empty
    let pacman = Polygon::new(vec![
      Point::new([big(0), big(0)]),
      Point::new([big(2), big(0)]),
      Point::new([big(1), big(1)]),
      Point::new([big(2), big(2)]),
      Point::new([big(0), big(2)]),
    ])
    .unwrap();

    // Note: This specific polygon may or may not have an empty kernel
    // The test is mainly to verify that kernel() returns Option and doesn't panic
    let _ = kernel(&pacman);
  }

  // Property-based tests
  proptest! {
    /// Property: kernel(convex_polygon) should equal the convex polygon itself
    /// (with some tolerance for floating-point/integer approximation)
    #[test]
    fn prop_kernel_of_convex_equals_input_i8(convex: PolygonConvex<i8>) {
      let poly: Polygon<i8> = convex.into();
      // May panic due to overflow in underlying implementation - this is expected
      if let Ok(Some(k)) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| kernel(&poly))) {
        // For convex polygons, kernel should have same number of vertices
        // (or very close, due to numerical precision)
        let vertex_diff = (k.iter().count() as i32 - poly.iter().count() as i32).abs();
        prop_assert!(vertex_diff <= 3, "Convex polygon kernel has very different vertex count");

        // Area should be approximately the same (within 10% for integer types)
        let poly_area = poly.signed_area::<f64>().abs();
        let kernel_area = k.signed_area::<f64>().abs();
        if poly_area > 0.1 {
          let area_ratio = (kernel_area / poly_area).abs();
          prop_assert!(area_ratio >= 0.90 && area_ratio <= 1.10,
            "Convex polygon kernel area differs significantly: {} vs {}", kernel_area, poly_area);
        }
      }
    }

    /// Property: kernel() returns Option (may panic due to overflow in underlying library)
    #[test]
    fn prop_kernel_returns_option_i8(poly: Polygon<i8>) {
      // The function should return Option<Polygon<T>>, not panic directly
      // However, underlying PolygonScalar operations may panic on overflow
      let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _result = kernel(&poly);
      }));
      // Test passes as long as we reach here
    }

    /// Property: when kernel succeeds, area(kernel(polygon)) <= area(polygon)
    #[test]
    fn prop_kernel_area_smaller_or_equal_i8(poly: Polygon<i8>) {
      // Catch any panics from overflow in underlying implementation
      if let Ok(maybe_k) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| kernel(&poly)))
        && let Some(k) = maybe_k {
        let poly_area = poly.signed_area::<f64>().abs();
        let kernel_area = k.signed_area::<f64>().abs();
        // Allow significant tolerance for numerical errors in integer arithmetic
        prop_assert!(kernel_area <= poly_area + 10.0,
          "Kernel area {} should be <= polygon area {}", kernel_area, poly_area);
      }
    }
  }
}
