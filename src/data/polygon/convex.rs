use claim::debug_assert_ok;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::*;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
use std::collections::BTreeSet;
use std::ops::*;

use crate::array::Orientation;
use crate::data::Point;
use crate::data::PointLocation;
use crate::data::TriangleView;
use crate::data::Vector;
use crate::transformation::*;
use crate::{Error, PolygonScalar};

use super::Polygon;

#[derive(Debug, Clone)]
pub struct PolygonConvex<T>(Polygon<T>);

///////////////////////////////////////////////////////////////////////////////
// PolygonConvex

impl<T> PolygonConvex<T>
where
  T: PolygonScalar,
{
  /// $O(1)$ Assume that a polygon is convex.
  ///
  /// # Safety
  /// The input polygon has to be strictly convex, ie. no vertices are allowed to
  /// be concave or colinear.
  pub fn new_unchecked(poly: Polygon<T>) -> PolygonConvex<T> {
    let convex = PolygonConvex(poly);
    debug_assert_ok!(convex.validate());
    convex
  }

  /// $O(\log n)$
  ///
  /// <iframe src="https://web.rgeometry.org:20443/loader.html?hash=mV9xetnc_IU="></iframe>
  pub fn locate(&self, pt: &Point<T, 2>) -> PointLocation {
    // debug_assert_ok!(self.validate());
    let poly = &self.0;
    let vertices = self.boundary_slice();
    let p0 = poly.point(vertices[0]);
    let mut lower = 1;
    let mut upper = vertices.len() - 1;
    while lower + 1 < upper {
      let middle = (lower + upper) / 2;
      if p0.orientation(&poly.point(vertices[middle]), pt) == Orientation::CounterClockWise {
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

  /// $O(n \log n)$
  pub fn validate(&self) -> Result<(), Error> {
    for cursor in self.0.iter_boundary() {
      if cursor.orientation() != Orientation::CounterClockWise {
        return Err(Error::ConvexViolation);
      }
    }
    self.0.validate()
  }

  /// $O(1)$
  pub fn polygon(&self) -> &Polygon<T> {
    self.into()
  }

  pub fn normalize(&self) -> PolygonConvex<BigRational>
  where
    T: PolygonScalar + Into<BigInt>,
    T::ExtendedSigned: Into<BigInt>,
  {
    PolygonConvex::new_unchecked(self.0.normalize())
  }
}

///////////////////////////////////////////////////////////////////////////////
// PolygonConvex<BigRational>

impl<T> PolygonConvex<T>
where
  T: Bounded + PolygonScalar + SampleUniform + Copy + Into<BigInt>,
{
  /// $O(n \log n)$ Uniformly sample a random convex polygon.
  ///
  /// The output polygon is rooted in (0,0), grows upwards, and has a height and width of T::MAX.
  ///
  /// ```no_run
  /// # use rgeometry_wasm::playground::*;
  /// # use rgeometry::data::*;
  /// # let convex: PolygonConvex<i8> = {
  /// PolygonConvex::random(3, &mut rand::thread_rng())
  /// # };
  /// # render_polygon(&convex.normalize());
  /// ```
  /// <iframe src="https://web.rgeometry.org:20443/loader.html?gist=037a23f8391390df8560a2043a14121e"></iframe>
  pub fn random<R>(n: usize, rng: &mut R) -> PolygonConvex<T>
  where
    R: Rng + ?Sized,
  {
    let n = n.max(3);
    let vs = {
      let mut vs = random_vectors(n, rng);
      Vector::sort_around(&mut vs);
      vs
    };
    let vertices: Vec<Point<T, 2>> = vs
      .into_iter()
      .scan(Point::zero(), |st, vec| {
        *st += vec;
        Some(*st)
      })
      .collect();
    let n_vertices = (*vertices).len();
    debug_assert_eq!(n_vertices, n);
    // FIXME: Use the convex hull algorithm for polygons rather than point sets.
    //        It runs in O(n) rather than O(n log n). Hasn't been implemented, yet, though.
    match crate::algorithms::convex_hull(vertices).ok() {
      // If the vertices are all colinear then give up and try again.
      None => Self::random(n, rng),
      Some(p) => p,
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Trait Implementations

impl<T: PolygonScalar> Deref for PolygonConvex<T> {
  type Target = Polygon<T>;
  fn deref(&self) -> &Self::Target {
    self.polygon()
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
  T: PolygonScalar + Bounded + SampleUniform + Copy + Into<BigInt>,
  R: Rng + ?Sized,
{
  let zero: T = Zero::zero();
  let max: T = Bounded::max_value();
  assert!(n > 0);
  let mut pts = Vec::with_capacity(n);
  while pts.len() < n - 1 {
    pts.push(rng.gen_range(zero..max));
  }
  pts.sort_unstable();
  pts.push(max);
  pts.into_iter().scan(zero, |from, x| {
    let out = x - *from;
    *from = x;
    Some(out)
  })
}

// Property: random_between_zero(10, 100, &mut rng).iter().sum::<isize>() == 0
fn random_between_zero<T, R>(n: usize, rng: &mut R) -> Vec<T>
where
  T: Bounded + PolygonScalar + SampleUniform + Copy + Into<BigInt>,
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
  T: Bounded + PolygonScalar + SampleUniform + Copy + Into<BigInt>,
  R: Rng + ?Sized,
{
  random_between_zero(n, rng)
    .into_iter()
    .zip(random_between_zero(n, rng).into_iter())
    .map(|(a, b)| Vector([a, b]))
    .collect()
}

///////////////////////////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
  use super::*;

  use crate::testing::*;
  use proptest::prelude::*;

  proptest! {
    #[test]
    fn all_random_convex_polygons_are_valid_i8(poly in any_convex::<i8>()) {
      prop_assert_eq!(poly.validate().err(), None)
    }

    #[test]
    fn random_convex_prop(poly in any_convex::<i8>()) {
      let (min, max) = poly.bounding_box();
      prop_assert_eq!(min.y_coord(), &0);
      let width = max.x_coord() - min.x_coord();
      let height = max.y_coord() - min.y_coord();
      prop_assert_eq!(width, i8::MAX);
      prop_assert_eq!(height, i8::MAX);
    }

    #[test]
    fn all_random_convex_polygons_are_valid_i64(poly in any_convex::<i64>()) {
      prop_assert_eq!(poly.validate().err(), None)
    }

    #[test]
    fn sum_to_max(n in 1..1000) {
      let mut rng = rand::thread_rng();
      let vecs = random_between_iter::<i8, _>(n as usize, &mut rng);
      prop_assert_eq!(vecs.sum::<i8>(), i8::MAX);

      let vecs = random_between_iter::<i64, _>(n as usize, &mut rng);
      prop_assert_eq!(vecs.sum::<i64>(), i64::MAX);
    }

    #[test]
    fn random_between_zero_properties(n in 2..1000) {
      let mut rng = rand::thread_rng();
      let vecs: Vec<i8> = random_between_zero(n as usize, &mut rng);
      prop_assert_eq!(vecs.iter().sum::<i8>(), 0);
      prop_assert_eq!(vecs.len(), n as usize);

      let vecs: Vec<i64> = random_between_zero(n as usize, &mut rng);
      prop_assert_eq!(vecs.iter().sum::<i64>(), 0);
      prop_assert_eq!(vecs.len(), n as usize);
    }

    #[test]
    fn sum_to_zero_vector(n in 2..1000) {
      let mut rng = rand::thread_rng();
      let vecs: Vec<Vector<i8,2>> = random_vectors(n as usize, &mut rng);
      prop_assert_eq!(vecs.into_iter().sum::<Vector<i8,2>>(), Vector::zero())
    }
  }
}
