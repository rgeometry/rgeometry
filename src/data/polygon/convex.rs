use num_traits::*;
use ordered_float::OrderedFloat;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use rand::Rng;
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
fn random_between_iter<T, R>(n: usize, rng: &mut R) -> impl Iterator<Item = T>
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
  use rand::rngs::SmallRng;
  use rand::SeedableRng;

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
  }
}
