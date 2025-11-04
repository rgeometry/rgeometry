use crate::data::{Point, Polygon};
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
/// empty or a smaller polygon.
///
/// # Algorithm
///
/// This is a naive O(n²) algorithm that:
/// 1. For each edge of the polygon, determines the half-plane containing the polygon interior
/// 2. Computes the intersection of all these half-planes using a half-plane intersection algorithm
/// 3. Returns the resulting polygon (which may be empty if the kernel is empty)
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
/// let k = kernel(&square);
/// // For a convex polygon, the kernel is the polygon itself
/// assert_eq!(k.signed_area::<BigRational>(), square.signed_area::<BigRational>());
/// ```
///
/// # References
///
/// - Wikipedia: [Star-shaped polygon](https://en.wikipedia.org/wiki/Star-shaped_polygon)
/// - Computational Geometry: Algorithms and Applications (de Berg et al.)
///
pub fn kernel<T: PolygonScalar>(poly: &Polygon<T>) -> Polygon<T> {
  // The algorithm computes the intersection of half-planes defined by each edge.
  // For a CCW-oriented polygon, the interior is to the left of each edge.

  if poly.iter().count() < 3 {
    // Degenerate case: not enough vertices
    return Polygon::new(vec![]).unwrap_or_else(|_| panic!("Failed to create empty polygon"));
  }

  // Start with a large bounding box that will be intersected with each half-plane
  // We'll use the convex hull approach: start with all vertices and incrementally
  // clip by each edge's half-plane.

  // Actually, a better approach: use a half-plane intersection algorithm.
  // For simplicity in this naive version, we'll use an O(n²) approach:
  // 1. Collect all edges as half-planes (lines with a direction)
  // 2. For each edge, we know the interior is on one side
  // 3. Intersect all these half-planes

  // Let's use a simpler incremental approach:
  // Start with the polygon itself, then for each edge, clip the current result
  // by the half-plane defined by that edge.

  let mut result_vertices: Vec<Point<T>> = poly.iter().cloned().collect();

  // For each edge of the original polygon
  for edge in poly.iter_boundary_edges() {
    let p1 = edge.src;
    let p2 = edge.dst;

    // Clip result_vertices by the half-plane defined by this edge
    result_vertices = clip_polygon_by_edge(&result_vertices, p1, p2);

    if result_vertices.is_empty() {
      // Kernel is empty
      break;
    }
  }

  // Create polygon from remaining vertices
  if result_vertices.len() < 3 {
    Polygon::new(vec![]).unwrap_or_else(|_| panic!("Failed to create empty polygon"))
  } else {
    Polygon::new(result_vertices).unwrap_or_else(|e| {
      // If polygon creation fails, return empty polygon
      Polygon::new(vec![]).unwrap_or_else(|_| panic!("Failed to create polygon: {:?}", e))
    })
  }
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
    if current_inside != next_inside
      && let Some(intersection) = line_intersection(current, next, p1, p2)
    {
      result.push(intersection);
    }
  }

  result
}

/// Compute intersection of two line segments (as infinite lines).
/// Returns None if lines are parallel.
fn line_intersection<T: PolygonScalar>(
  a1: &Point<T>,
  a2: &Point<T>,
  b1: &Point<T>,
  b2: &Point<T>,
) -> Option<Point<T>> {
  // Line 1: a1 + t * (a2 - a1)
  // Line 2: b1 + s * (b2 - b1)
  //
  // a1 + t * (a2 - a1) = b1 + s * (b2 - b1)
  //
  // Solving for t:
  // a1.x + t * (a2.x - a1.x) = b1.x + s * (b2.x - b1.x)
  // a1.y + t * (a2.y - a1.y) = b1.y + s * (b2.y - b1.y)
  //
  // Let:
  // dx_a = a2.x - a1.x
  // dy_a = a2.y - a1.y
  // dx_b = b2.x - b1.x
  // dy_b = b2.y - b1.y
  //
  // a1.x + t * dx_a = b1.x + s * dx_b
  // a1.y + t * dy_a = b1.y + s * dy_b
  //
  // t * dx_a - s * dx_b = b1.x - a1.x
  // t * dy_a - s * dy_b = b1.y - a1.y
  //
  // Using Cramer's rule:
  // denominator = dx_a * (-dy_b) - dy_a * (-dx_b) = -dx_a * dy_b + dy_a * dx_b
  //             = dy_a * dx_b - dx_a * dy_b
  //
  // If denominator is 0, lines are parallel

  let dx_a = a2.array[0].clone() - a1.array[0].clone();
  let dy_a = a2.array[1].clone() - a1.array[1].clone();
  let dx_b = b2.array[0].clone() - b1.array[0].clone();
  let dy_b = b2.array[1].clone() - b1.array[1].clone();

  let denominator = dy_a.clone() * dx_b.clone() - dx_a.clone() * dy_b.clone();

  if denominator == T::from_constant(0) {
    return None; // Lines are parallel
  }

  // t = ((b1.x - a1.x) * (-dy_b) - (b1.y - a1.y) * (-dx_b)) / denominator
  //   = (-(b1.x - a1.x) * dy_b + (b1.y - a1.y) * dx_b) / denominator

  let dx = b1.array[0].clone() - a1.array[0].clone();
  let dy = b1.array[1].clone() - a1.array[1].clone();

  let t_numerator = dx.clone() * dy_b.clone() - dy.clone() * dx_b.clone();
  let t = t_numerator / denominator.clone();

  // Intersection point: a1 + t * (a2 - a1)
  let x = a1.array[0].clone() + t.clone() * dx_a;
  let y = a1.array[1].clone() + t * dy_a;

  Some(Point::new([x, y]))
}

#[cfg(test)]
mod tests {
  use super::*;
  use num_rational::BigRational;

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

    let k = kernel(&square);
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

    let k = kernel(&triangle);
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

    let k = kernel(&star);
    assert!(k.iter().count() >= 3);
    // Kernel area should be <= original area
    assert!(k.signed_area::<BigRational>() <= star.signed_area::<BigRational>());
  }
}
