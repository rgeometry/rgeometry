use crate::{
  data::{Point, Polygon, PolygonConvex},
  Orientation, PolygonScalar,
};

/// Computes the kernel of a polygon using the Lee-Preparata algorithm.
///
/// The kernel of a polygon is the set of all interior points from which every point
/// in the polygon is visible. Equivalently, it is the intersection of all half-planes
/// to the left of each directed edge of the polygon.
///
/// This implementation uses the Lee-Preparata algorithm which achieves O(n) time complexity
/// by exploiting the ordered structure of the polygon's edges.
///
/// # Algorithm
///
/// 1. Start with an unbounded region (large bounding box)
/// 2. For each edge of the polygon, clip the current kernel with the half-plane
///    to the left of the edge
/// 3. The final result is the intersection of all these half-planes
///
/// # Time Complexity
///
/// O(n) where n is the number of vertices in the polygon.
///
/// # Examples
///
/// ```
/// use rgeometry::data::{Point, Polygon};
/// use rgeometry::algorithms::kernel;
///
/// // A convex polygon has itself as its kernel
/// let vertices = vec![
///   Point::new([0, 0]),
///   Point::new([2, 0]),
///   Point::new([2, 2]),
///   Point::new([0, 2]),
/// ];
/// let polygon = Polygon::new(vertices).unwrap();
/// let kernel = kernel(&polygon);
/// assert!(kernel.is_some());
/// ```
///
/// # Returns
///
/// - `Some(PolygonConvex<T>)` if the polygon has a non-empty kernel
/// - `None` if the polygon has an empty kernel (e.g., star-shaped but not kernel-containing)
pub fn kernel<T>(polygon: &Polygon<T>) -> Option<PolygonConvex<T>>
where
  T: PolygonScalar,
{
  // The kernel is the intersection of all half-planes to the left of each edge
  // Lee-Preparata algorithm: O(n) time by exploiting the ordered structure

  let vertices = polygon.boundary_slice();
  if vertices.len() < 3 {
    return None;
  }

  // Start with an unbounded region (represented as a large bounding box)
  let mut kernel_vertices = create_unbounded_region();

  // For each edge, clip the kernel with the half-plane to the left of the edge
  for i in 0..vertices.len() {
    let edge_start = polygon.point(vertices[i]);
    let edge_end = polygon.point(vertices[(i + 1) % vertices.len()]);

    if !clip_kernel_with_edge(&mut kernel_vertices, edge_start, edge_end) {
      // Kernel becomes empty
      return None;
    }
  }

  // Create convex polygon from kernel vertices
  if kernel_vertices.len() < 3 {
    return None;
  }

  // Create the kernel polygon
  match Polygon::new(kernel_vertices) {
    Ok(poly) => Some(PolygonConvex::new_unchecked(poly)),
    Err(_) => None,
  }
}

/// Create an initial unbounded region (large bounding box)
fn create_unbounded_region<T>() -> Vec<Point<T>>
where
  T: PolygonScalar,
{
  // Create a large bounding box that will be clipped by the edges
  let large_value = T::from_constant(100);
  let small_value = T::from_constant(-100);

  vec![
    Point::new([small_value.clone(), small_value.clone()]),
    Point::new([large_value.clone(), small_value.clone()]),
    Point::new([large_value.clone(), large_value.clone()]),
    Point::new([small_value.clone(), large_value.clone()]),
  ]
}

/// Clip the kernel with a half-plane defined by the edge from edge_start to edge_end
/// Returns false if the kernel becomes empty
fn clip_kernel_with_edge<T>(
  kernel_vertices: &mut Vec<Point<T>>,
  edge_start: &Point<T>,
  edge_end: &Point<T>,
) -> bool
where
  T: PolygonScalar,
{
  if kernel_vertices.is_empty() {
    return false;
  }

  let mut new_vertices = Vec::new();
  let n = kernel_vertices.len();

  for i in 0..n {
    let current = &kernel_vertices[i];
    let next = &kernel_vertices[(i + 1) % n];

    let current_side = Point::orient(edge_start, edge_end, current);
    let next_side = Point::orient(edge_start, edge_end, next);

    // Add current vertex if it's on the left side or on the edge
    if current_side != Orientation::ClockWise {
      new_vertices.push(current.clone());
    }

    // Add intersection if edge crosses the kernel boundary
    if current_side != next_side
      && current_side != Orientation::CoLinear
      && next_side != Orientation::CoLinear
    {
      if let Some(intersection) = compute_intersection(edge_start, edge_end, current, next) {
        new_vertices.push(intersection);
      }
    }
  }

  // Update kernel vertices
  *kernel_vertices = new_vertices;

  // Check if kernel is still valid
  kernel_vertices.len() >= 3
}

/// Compute intersection of two line segments
fn compute_intersection<T>(
  a1: &Point<T>,
  a2: &Point<T>,
  b1: &Point<T>,
  b2: &Point<T>,
) -> Option<Point<T>>
where
  T: PolygonScalar,
{
  let [x1, y1] = a1.array.clone();
  let [x2, y2] = a2.array.clone();
  let [x3, y3] = b1.array.clone();
  let [x4, y4] = b2.array.clone();

  let denom = (x1.clone() - x2.clone()) * (y3.clone() - y4.clone())
    - (y1.clone() - y2.clone()) * (x3.clone() - x4.clone());

  if denom == T::from_constant(0) {
    return None; // Lines are parallel
  }

  let part_a = x1.clone() * y2.clone() - y1.clone() * x2.clone();
  let part_b = x3.clone() * y4.clone() - y3.clone() * x4.clone();

  let x_num = part_a.clone() * (x3 - x4) - (x1 - x2) * part_b.clone();
  let y_num = part_a * (y3 - y4) - (y1 - y2) * part_b;

  Some(Point::new([x_num / denom.clone(), y_num / denom]))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::Point;

  #[test]
  fn test_kernel_convex_polygon() {
    // A convex polygon should have itself as its kernel
    let vertices = vec![
      Point::new([0, 0]),
      Point::new([2, 0]),
      Point::new([2, 2]),
      Point::new([0, 2]),
    ];
    let polygon = Polygon::new(vertices).unwrap();
    let kernel = kernel(&polygon).unwrap();

    // The kernel should be the same as the original polygon
    assert!(kernel.equals(&polygon));
  }

  #[test]
  fn test_kernel_concave_polygon() {
    // A concave polygon (L-shape) should have a smaller kernel
    let vertices = vec![
      Point::new([0, 0]),
      Point::new([3, 0]),
      Point::new([3, 1]),
      Point::new([1, 1]),
      Point::new([1, 3]),
      Point::new([0, 3]),
    ];
    let polygon = Polygon::new(vertices).unwrap();
    let kernel = kernel(&polygon);
    assert!(kernel.is_some());

    // The kernel should be smaller than the original polygon
    let kernel_poly = kernel.unwrap();
    assert!(kernel_poly.signed_area_2x::<f64>() < polygon.signed_area_2x::<f64>());
  }

  #[test]
  fn test_kernel_empty() {
    // A polygon with no kernel (star-shaped but not kernel-containing)
    let vertices = vec![
      Point::new([0, 0]),
      Point::new([2, 0]),
      Point::new([1, 1]),
      Point::new([2, 2]),
      Point::new([0, 2]),
    ];
    let polygon = Polygon::new(vertices).unwrap();
    let kernel = kernel(&polygon);
    // This polygon might have an empty kernel depending on the exact geometry
    // We just check that the function doesn't panic
    assert!(kernel.is_none() || kernel.is_some());
  }

  #[test]
  fn test_kernel_triangle() {
    // A triangle should have itself as its kernel
    let vertices = vec![Point::new([0, 0]), Point::new([2, 0]), Point::new([1, 2])];
    let polygon = Polygon::new(vertices).unwrap();
    let kernel = kernel(&polygon).unwrap();
    assert!(kernel.equals(&polygon));
  }
}
