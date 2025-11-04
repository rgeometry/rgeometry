use num_traits::*;
use ordered_float::OrderedFloat;
use rand::Rng;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use std::ops::*;

use crate::data::{Point, PointLocation, TriangleView, Vector};
use crate::{Error, Orientation, PolygonScalar, TotalOrd};

use super::Polygon;

#[derive(Debug, Clone, Hash)]
pub struct PolygonConvex<T>(Polygon<T>);

///////////////////////////////////////////////////////////////////////////////
// PolygonConvex

impl<T> PolygonConvex<T>
where
  T: PolygonScalar,
{
  /// Assume that a polygon is convex.
  ///
  /// # Safety
  /// The input polygon has to be strictly convex, ie. no vertices are allowed to
  /// be concave or colinear.
  ///
  /// # Time complexity
  /// $O(1)$
  pub fn new_unchecked(poly: Polygon<T>) -> PolygonConvex<T> {
    PolygonConvex(poly)
  }

  /// Locate a point relative to a convex polygon.
  ///
  /// # Time complexity
  /// $O(\log n)$
  ///
  /// # Examples
  /// <iframe src="https://web.rgeometry.org/wasm/gist/2cb9ff5bd6ce24f395a5ea30280aabee"></iframe>
  ///
  pub fn locate(&self, pt: &Point<T, 2>) -> PointLocation {
    let poly = &self.0;
    let vertices = self.boundary_slice();
    let p0 = poly.point(vertices[0]);
    let mut lower = 1;
    let mut upper = vertices.len() - 1;
    while lower + 1 < upper {
      let middle = (lower + upper) / 2;
      if Point::orient(p0, poly.point(vertices[middle]), pt) == Orientation::CounterClockWise {
        lower = middle;
      } else {
        upper = middle;
      }
    }
    let p1 = poly.point(vertices[lower]);
    let p2 = poly.point(vertices[upper]);
    let triangle = TriangleView::new_unchecked([p0, p1, p2]);
    triangle.locate(pt)
  }

  /// Compute the extreme point of the convex polygon in the direction of a vector.
  ///
  /// Returns the point on the polygon that is furthest in the direction of the given vector.
  ///
  /// # Time complexity
  /// $O(n)$ in the current implementation. This can be optimized to $O(\log n)$ using binary search
  /// on the unimodal sequence formed by the vertex projections along the direction.
  ///
  /// # Arguments
  ///
  /// * `direction` - The direction vector along which to find the extreme point
  ///
  /// # Examples
  ///
  /// ```rust
  /// # use rgeometry::data::{Polygon, Vector, Point, PolygonConvex};
  /// # use rgeometry::algorithms::convex_hull;
  /// let vertices = vec![
  ///   Point::new([0, 0]),
  ///   Point::new([2, 0]),
  ///   Point::new([2, 2]),
  ///   Point::new([0, 2]),
  /// ];
  /// let poly = convex_hull(vertices).unwrap();
  /// let direction = Vector([1, 0]); // Point to the right
  /// let extreme = poly.extreme_point(&direction);
  /// assert_eq!(extreme.x_coord(), &2);
  /// ```
  ///
  pub fn extreme_point(&self, direction: &Vector<T, 2>) -> &Point<T, 2> {
    let poly = &self.0;
    let vertices = self.boundary_slice();
    let n = vertices.len();

    if n <= 1 {
      return poly.point(vertices[0]);
    }

    // Find the vertex that maximizes the projection along direction.
    // We perform a linear scan to find the global maximum.
    //
    // For a convex polygon with vertices in counter-clockwise order,
    // there is exactly one local maximum for any direction, and it is the global maximum.
    let mut best_point = poly.point(vertices[0]);

    for &idx in vertices.iter() {
      let point = poly.point(idx);
      if direction.cmp_along(best_point, point) == std::cmp::Ordering::Less {
        best_point = point;
      }
    }

    best_point
  }

  /// Validates the following properties:
  ///  * Each vertex is convex, ie. not concave or colinear.
  ///  * All generate polygon properties hold true (eg. no duplicate points, no self-intersections).
  ///
  /// # Time complexity
  /// $O(n \log n)$
  pub fn validate(&self) -> Result<(), Error> {
    for cursor in self.0.iter_boundary() {
      if cursor.orientation() != Orientation::CounterClockWise {
        return Err(Error::ConvexViolation);
      }
    }
    self.0.validate()
  }

  pub fn cast<U>(self) -> PolygonConvex<U>
  where
    T: Clone + Into<U>,
    U: PolygonScalar,
  {
    PolygonConvex::new_unchecked(self.0.cast())
  }

  pub fn float(self) -> PolygonConvex<OrderedFloat<f64>>
  where
    T: Clone + Into<f64>,
  {
    PolygonConvex::new_unchecked(self.0.float())
  }

  /// $O(1)$
  pub fn polygon(&self) -> &Polygon<T> {
    self.into()
  }

  /// Uniformly sample a random convex polygon.
  ///
  /// The output polygon is rooted in `(0,0)`, grows upwards, and has a height and width of [`T::max_value()`](Bounded::max_value).
  ///
  /// # Time complexity
  /// $O(n \log n)$
  ///
  // # Examples
  // ```no_run
  // # use rgeometry_wasm::playground::*;
  // # use rgeometry::data::*;
  // # clear_screen();
  // # set_viewport(2.0, 2.0);
  // # let convex: PolygonConvex<i8> = {
  // PolygonConvex::random(3, &mut rand::thread_rng())
  // # };
  // # render_polygon(&convex.float().normalize());
  // ```
  // <iframe src="https://web.rgeometry.org/wasm/gist/9abc54a5e2e3d33e3dd1785a71e812d2"></iframe>
  pub fn random<R>(n: usize, rng: &mut R) -> PolygonConvex<T>
  where
    T: Bounded + PolygonScalar + SampleUniform,
    R: Rng + ?Sized,
  {
    let n = n.max(3);
    loop {
      let vs = {
        let mut vs = random_vectors(n, rng);
        Vector::sort_around(&mut vs);
        vs
      };
      let vertices: Vec<Point<T, 2>> = vs
        .into_iter()
        .scan(Point::zero(), |st, vec| {
          *st += vec;
          Some((*st).clone())
        })
        .collect();
      let n_vertices = (*vertices).len();
      debug_assert_eq!(n_vertices, n);
      // FIXME: Use the convex hull algorithm for polygons rather than point sets.
      //        It runs in O(n) rather than O(n log n). Hasn't been implemented, yet, though.
      // If the vertices are all colinear then give up and try again.
      // FIXME: If the RNG always returns zero then we might loop forever.
      //        Maybe limit the number of recursions.
      if let Ok(p) = crate::algorithms::convex_hull(vertices) {
        return p;
      }
    }
  }
}

impl PolygonConvex<OrderedFloat<f64>> {
  #[must_use]
  pub fn normalize(&self) -> PolygonConvex<OrderedFloat<f64>> {
    PolygonConvex::new_unchecked(self.0.normalize())
  }
}

impl PolygonConvex<f64> {
  #[must_use]
  pub fn normalize(&self) -> PolygonConvex<f64> {
    PolygonConvex::new_unchecked(self.0.normalize())
  }
}

impl PolygonConvex<f32> {
  #[must_use]
  pub fn normalize(&self) -> PolygonConvex<f32> {
    PolygonConvex::new_unchecked(self.0.normalize())
  }
}

///////////////////////////////////////////////////////////////////////////////
// Trait Implementations

impl<T> Deref for PolygonConvex<T> {
  type Target = Polygon<T>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> From<PolygonConvex<T>> for Polygon<T> {
  fn from(convex: PolygonConvex<T>) -> Polygon<T> {
    convex.0
  }
}

impl<'a, T> From<&'a PolygonConvex<T>> for &'a Polygon<T> {
  fn from(convex: &'a PolygonConvex<T>) -> &'a Polygon<T> {
    &convex.0
  }
}

impl Distribution<PolygonConvex<isize>> for Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PolygonConvex<isize> {
    PolygonConvex::random(100, rng)
  }
}

///////////////////////////////////////////////////////////////////////////////
// Helper functions

// Property: random_between(n, max, &mut rng).sum::<usize>() == max
fn random_between_iter<T, R>(n: usize, rng: &mut R) -> impl Iterator<Item = T> + use<T, R>
where
  T: PolygonScalar + Bounded + SampleUniform,
  R: Rng + ?Sized,
{
  let zero: T = T::from_constant(0);
  let max: T = Bounded::max_value();
  assert!(n > 0);
  let mut pts = Vec::with_capacity(n);
  while pts.len() < n - 1 {
    pts.push(rng.gen_range(zero.clone()..max.clone()));
  }
  pts.sort_unstable_by(TotalOrd::total_cmp);
  pts.push(max);
  pts.into_iter().scan(zero, |from, x| {
    let out = x.clone() - (*from).clone();
    *from = x;
    Some(out)
  })
}

// Property: random_between_zero(10, 100, &mut rng).iter().sum::<isize>() == 0
fn random_between_zero<T, R>(n: usize, rng: &mut R) -> Vec<T>
where
  T: Bounded + PolygonScalar + SampleUniform,
  R: Rng + ?Sized,
{
  assert!(n >= 2);
  let n_positive = rng.gen_range(1..n); // [1;n[
  let n_negative = n - n_positive;
  assert!(n_positive + n_negative == n);
  let positive = random_between_iter(n_positive, rng);
  let negative = random_between_iter(n_negative, rng).map(|i: T| -i);
  let mut result: Vec<T> = positive.chain(negative).collect();
  result.shuffle(rng);
  result
}

// Random vectors that sum to zero.
fn random_vectors<T, R>(n: usize, rng: &mut R) -> Vec<Vector<T, 2>>
where
  T: Bounded + PolygonScalar + SampleUniform,
  R: Rng + ?Sized,
{
  random_between_zero(n, rng)
    .into_iter()
    .zip(random_between_zero(n, rng))
    .map(|(a, b)| Vector([a, b]))
    .collect()
}

///////////////////////////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
  use super::*;
  use rand::SeedableRng;
  use rand::rngs::SmallRng;

  use proptest::prelude::*;
  use proptest::proptest as proptest_block;

  proptest_block! {
    // These traits are usually derived but let's not rely on that.
    #[test]
    fn fuzz_convex_traits(poly: PolygonConvex<i8>) {
      let _ = format!("{:?}", &poly);
      let _ = poly.clone();
    }

    #[test]
    fn fuzz_validate(poly: Polygon<i8>) {
      let convex = PolygonConvex::new_unchecked(poly);
      let _ = convex.validate();
    }

    #[test]
    fn all_random_convex_polygons_are_valid_i8(poly: PolygonConvex<i8>) {
      prop_assert_eq!(poly.validate().err(), None)
    }

    #[test]
    fn random_convex_prop(poly: PolygonConvex<i8>) {
      let (min, max) = poly.bounding_box();
      prop_assert_eq!(min.y_coord(), &0);
      let width = max.x_coord() - min.x_coord();
      let height = max.y_coord() - min.y_coord();
      prop_assert_eq!(width, i8::MAX);
      prop_assert_eq!(height, i8::MAX);
    }

    #[test]
    fn all_random_convex_polygons_are_valid_i64(poly: PolygonConvex<i64>) {
      prop_assert_eq!(poly.validate().err(), None)
    }

    #[test]
    fn sum_to_max(n in 1..1000, seed: u64) {
      let mut rng = SmallRng::seed_from_u64(seed);
      let vecs = random_between_iter::<i8, _>(n as usize, &mut rng);
      prop_assert_eq!(vecs.sum::<i8>(), i8::MAX);

      let vecs = random_between_iter::<i64, _>(n as usize, &mut rng);
      prop_assert_eq!(vecs.sum::<i64>(), i64::MAX);
    }

    #[test]
    fn random_between_zero_properties(n in 2..1000, seed: u64) {
      let mut rng = SmallRng::seed_from_u64(seed);
      let vecs: Vec<i8> = random_between_zero(n as usize, &mut rng);
      prop_assert_eq!(vecs.iter().sum::<i8>(), 0);
      prop_assert_eq!(vecs.len(), n as usize);

      let vecs: Vec<i64> = random_between_zero(n as usize, &mut rng);
      prop_assert_eq!(vecs.iter().sum::<i64>(), 0);
      prop_assert_eq!(vecs.len(), n as usize);
    }

    #[test]
    fn sum_to_zero_vector(n in 2..1000, seed: u64) {
      let mut rng = SmallRng::seed_from_u64(seed);
      let vecs: Vec<Vector<i8, 2>> = random_vectors(n as usize, &mut rng);
      prop_assert_eq!(vecs.into_iter().sum::<Vector<i8, 2>>(), Vector([0, 0]))
    }

    #[test]
    fn extreme_point_right_direction(poly: PolygonConvex<i8>) {
      let direction = Vector([1i8, 0]);
      let extreme = poly.extreme_point(&direction);
      // The extreme point should have x-coordinate equal to or very close to max
      let (_, max) = poly.bounding_box();
      // Just verify it's a valid vertex on the polygon
      let vertices = poly.boundary_slice();
      let found = vertices.iter().any(|&idx| poly.polygon().point(idx) == extreme);
      prop_assert!(found, "Extreme point should be a vertex of the polygon");
    }

    #[test]
    fn extreme_point_up_direction(poly: PolygonConvex<i8>) {
      let direction = Vector([0i8, 1]);
      let extreme = poly.extreme_point(&direction);
      // Just verify it's a valid vertex on the polygon
      let vertices = poly.boundary_slice();
      let found = vertices.iter().any(|&idx| poly.polygon().point(idx) == extreme);
      prop_assert!(found, "Extreme point should be a vertex of the polygon");
    }

    #[test]
    fn extreme_point_is_on_polygon(poly: PolygonConvex<i8>) {
      let direction = Vector([1i8, 1]);
      let extreme = poly.extreme_point(&direction);
      let vertices = poly.boundary_slice();
      let poly_ref = poly.polygon();

      let found = vertices.iter().any(|&idx| poly_ref.point(idx) == extreme);
      prop_assert!(found, "Extreme point should be one of the polygon vertices");
    }

    #[test]
    fn extreme_point_is_optimal(poly: PolygonConvex<i8>) {
      let direction = Vector([1i8, 1]);
      let extreme = poly.extreme_point(&direction);
      let vertices = poly.boundary_slice();
      let poly_ref = poly.polygon();

      // Check that no other vertex is further in the direction
      for &idx in vertices {
        let other_point = poly_ref.point(idx);
        if direction.cmp_along(other_point, extreme) == std::cmp::Ordering::Greater {
          prop_assert!(
            false,
            "Found a better point than the extreme point: {:?} > {:?}",
            other_point,
            extreme
          );
        }
      }
    }

    #[test]
    fn extreme_point_consistency_across_directions(poly: PolygonConvex<i8>) {
      let directions = vec![
        Vector([1i8, 0]),
        Vector([0i8, 1]),
        Vector([-1i8, 0]),
        Vector([0i8, -1]),
        Vector([1i8, 1]),
        Vector([-1i8, 1]),
      ];

      for dir in directions {
        let extreme = poly.extreme_point(&dir);
        let vertices = poly.boundary_slice();
        let poly_ref = poly.polygon();

        // Verify it's a valid vertex
        let found = vertices.iter().any(|&idx| poly_ref.point(idx) == extreme);
        prop_assert!(found, "Extreme point should be a valid vertex for direction {:?}", dir);
      }
    }
  }

  #[test]
  fn extreme_point_unit_square_right() {
    let vertices = vec![
      Point::new([0i32, 0]),
      Point::new([1, 0]),
      Point::new([1, 1]),
      Point::new([0, 1]),
    ];
    let poly = crate::algorithms::convex_hull(vertices).unwrap();
    let direction = Vector([1i32, 0]);
    let extreme = poly.extreme_point(&direction);
    assert_eq!(extreme.x_coord(), &1);
  }

  #[test]
  fn extreme_point_unit_square_up() {
    let vertices = vec![
      Point::new([0i32, 0]),
      Point::new([1, 0]),
      Point::new([1, 1]),
      Point::new([0, 1]),
    ];
    let poly = crate::algorithms::convex_hull(vertices).unwrap();
    let direction = Vector([0i32, 1]);
    let extreme = poly.extreme_point(&direction);
    assert_eq!(extreme.y_coord(), &1);
  }

  #[test]
  fn extreme_point_triangle() {
    let vertices = vec![
      Point::new([0i32, 0]),
      Point::new([4, 0]),
      Point::new([2, 3]),
    ];
    let poly = crate::algorithms::convex_hull(vertices).unwrap();
    let direction = Vector([1i32, 1]);
    let extreme = poly.extreme_point(&direction);
    // The extreme point should be (4, 0) or (2, 3), but (4, 0) is furthest to the right
    // Actually, we need to check which is furthest in the [1, 1] direction
    // (4, 0) projects to 4*1 + 0*1 = 4
    // (2, 3) projects to 2*1 + 3*1 = 5
    // So (2, 3) should be the extreme point
    assert_eq!(extreme.x_coord(), &2);
    assert_eq!(extreme.y_coord(), &3);
  }

  #[test]
  fn extreme_point_triangle_diagonal() {
    let vertices = vec![
      Point::new([0i32, 0]),
      Point::new([4, 0]),
      Point::new([2, 4]),
    ];
    let poly = crate::algorithms::convex_hull(vertices).unwrap();
    let direction = Vector([1i32, 1]);
    let extreme = poly.extreme_point(&direction);
    // (4, 0) projects to 4*1 + 0*1 = 4
    // (2, 4) projects to 2*1 + 4*1 = 6
    // So (2, 4) should be the extreme point
    assert_eq!(extreme.x_coord(), &2);
    assert_eq!(extreme.y_coord(), &4);
  }
}
