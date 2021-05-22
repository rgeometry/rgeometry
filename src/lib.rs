#![allow(unused_imports)]
// #![allow(incomplete_features)]
// #![feature(const_generics)]
// #![feature(const_evaluatable_checked)]
#![doc(html_playground_url = "https://rgeometry.org/rgeometry-playground/")]
// use nalgebra::geometry::Point;
use claim::debug_assert_ok;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::*;
use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use rand::Rng;
use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::iter::Sum;
use std::iter::Zip;
use std::ops::*;

pub mod algorithms;
mod array;
pub mod data;
mod intersection;
mod matrix;
mod point;
mod transformation;
mod vector;

pub use array::Orientation;
use data::polygon;
use data::*;
use point::Point;
use transformation::*;
use vector::{Vector, VectorView};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
  InsufficientVertices,
  SelfIntersections,
  /// Two consecutive line segments are either colinear or oriented clockwise.
  ConvexViolation,
  ClockWiseViolation,
}
pub trait PolygonScalar<T = Self, Output = Self>:
  PolygonScalarRef<T, Output>
  + AddAssign<Output>
  + MulAssign<Output>
  + FromPrimitive
  + One
  + Zero
  + Sum
  + Ord
  + Neg<Output = Output>
{
}
impl<T, Rhs, Output> PolygonScalar<Rhs, Output> for T where
  T: PolygonScalarRef<Rhs, Output>
    + AddAssign<Output>
    + MulAssign<Output>
    + FromPrimitive
    + One
    + Zero
    + Sum
    + Ord
    + Neg<Output = Output>
{
}

pub trait PolygonScalarRef<T = Self, Output = Self>: Clone + NumOps<T, Output> {}
impl<T, Rhs, Output> PolygonScalarRef<Rhs, Output> for T where T: Clone + NumOps<Rhs, Output> {}

#[derive(Debug, Clone)]
pub struct Polygon<T, P = ()> {
  pub(crate) points: Vec<Point<T, 2>>,
  pub(crate) boundary: usize,
  pub(crate) holes: Vec<usize>,
  pub(crate) meta: Vec<P>,
}

impl<T> Polygon<T> {
  pub fn new(points: Vec<Point<T, 2>>) -> Result<Polygon<T>, Error>
  where
    T: PolygonScalar,
  {
    let len = points.len();
    let mut meta = Vec::with_capacity(len);
    meta.resize(len, ());
    let p = Polygon {
      points,
      boundary: len,
      holes: vec![],
      meta,
    };
    p.validate()?;
    Ok(p)
  }
}

impl<T, P> Polygon<T, P> {
  // Validate that a polygon is simple.
  // https://en.wikipedia.org/wiki/Simple_polygon
  pub fn validate(&self) -> Result<(), Error>
  where
    T: PolygonScalar,
  {
    // Has no duplicate points.
    // TODO. Hm, finding duplicates is difficult when using IEEE floats.
    // There are two crates for dealing with this: noisy_float and ordered-float.
    // Unfortunately, both libraries only implement a subset of the traits that
    // are implemented by f64 and are required by rgeometry.
    // For now, we'll just not look for duplicate points. :(

    self.validate_weakly()
  }

  pub fn validate_weakly(&self) -> Result<(), Error>
  where
    T: PolygonScalar,
  {
    // Has at least three points.
    if self.points.len() < 3 {
      return Err(Error::InsufficientVertices);
    }
    // Is counter-clockwise
    if self.signed_area_2x() < T::zero() {
      return Err(Error::ClockWiseViolation);
    }
    // Has no self intersections.
    // TODO. Only check line intersections. Overlapping vertices are OK.
    Ok(())
  }

  pub fn centroid(&self) -> Point<T, 2>
  where
    T: PolygonScalar,
    for<'a> &'a T: PolygonScalarRef<&'a T, T>,
  {
    let xs: Vector<T, 2> = self
      .iter_boundary_edges()
      .map(|edge| {
        let p = edge.0.inner().0.as_vec();
        let q = edge.1.inner().0.as_vec();
        (p + q) * (p.0[0].clone() * q.0[1].clone() - q.0[0].clone() * p.0[1].clone())
      })
      .sum();
    let three = T::from_usize(3).unwrap();
    Point::from(xs / (three * self.signed_area_2x()))
  }

  pub fn signed_area(&self) -> T
  where
    T: PolygonScalar,
  {
    self.signed_area_2x() / T::from_usize(2).unwrap()
  }

  pub fn signed_area_2x(&self) -> T
  where
    T: PolygonScalar,
  {
    self
      .iter_boundary_edges()
      .map(|edge| {
        let p = edge.0.inner().0;
        let q = edge.1.inner().0;
        p.array[0].clone() * q.array[1].clone() - q.array[0].clone() * p.array[1].clone()
      })
      .sum()
  }

  pub fn vertex(&self, idx: isize) -> &Point<T, 2> {
    self
      .points
      .index(idx.rem_euclid(self.points.len() as isize) as usize)
  }

  pub fn vertex_orientation(&self, idx: isize) -> Orientation
  where
    T: PolygonScalar,
    for<'a> &'a T: PolygonScalarRef<&'a T, T>,
  {
    // debug_assert_ok!(self.validate());
    let p1 = self.vertex(idx - 1);
    let p2 = self.vertex(idx);
    let p3 = self.vertex(idx + 1);
    p1.orientation(p2, p3)
  }

  pub fn iter_boundary_edges(&self) -> polygon::EdgeIter<'_, T, P, 2> {
    // let mut iter = self.iter();
    // let (this_point, this_meta) = iter.next().unwrap();
    polygon::EdgeIter {
      at: 0,
      points: self.points.borrow(),
      meta: self.meta.borrow(),
    }
  }

  pub fn map_points<F>(self, f: F) -> Polygon<T, P>
  where
    F: Fn(Point<T, 2>) -> Point<T, 2>,
  {
    let pts = self.points.into_iter().map(f).collect();
    Polygon {
      points: pts,
      boundary: self.boundary,
      holes: self.holes,
      meta: self.meta,
    }
  }

  pub fn iter(&self) -> polygon::Iter<'_, T, P> {
    polygon::Iter {
      iter: self.points.iter().zip(self.meta.iter()),
    }
  }

  pub fn iter_mut(&mut self) -> polygon::IterMut<'_, T, P> {
    polygon::IterMut {
      points: self.points.iter_mut(),
      meta: self.meta.iter_mut(),
    }
  }

  pub fn cast<U, F>(self, f: F) -> Polygon<U, P>
  where
    T: PolygonScalar,
    F: Fn(T) -> U + Clone,
  {
    let pts = self.points.into_iter().map(|p| p.cast(f.clone())).collect();
    Polygon {
      points: pts,
      boundary: self.boundary,
      holes: self.holes,
      meta: self.meta,
    }
  }
}

impl<P> From<Polygon<BigRational, P>> for Polygon<f64, P> {
  fn from(p: Polygon<BigRational, P>) -> Polygon<f64, P> {
    let pts = p.points.into_iter().map(|p| Point::from(&p)).collect();
    Polygon {
      points: pts,
      boundary: p.boundary,
      holes: p.holes,
      meta: p.meta,
    }
  }
}
impl<'a, P: Clone> From<&'a Polygon<BigRational, P>> for Polygon<f64, P> {
  fn from(p: &Polygon<BigRational, P>) -> Polygon<f64, P> {
    let pts = p.points.iter().map(Point::from).collect();
    Polygon {
      points: pts,
      boundary: p.boundary,
      holes: p.holes.clone(),
      meta: p.meta.clone(),
    }
  }
}

impl<P> From<Polygon<f64, P>> for Polygon<BigRational, P> {
  fn from(p: Polygon<f64, P>) -> Polygon<BigRational, P> {
    let pts = p.points.into_iter().map(|p| Point::from(&p)).collect();
    Polygon {
      points: pts,
      boundary: p.boundary,
      holes: p.holes,
      meta: p.meta,
    }
  }
}

impl<'a, P: Clone> From<&'a Polygon<f64, P>> for Polygon<BigRational, P> {
  fn from(p: &Polygon<f64, P>) -> Polygon<BigRational, P> {
    let pts = p.points.iter().map(Point::from).collect();
    Polygon {
      points: pts,
      boundary: p.boundary,
      holes: p.holes.clone(),
      meta: p.meta.clone(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct ConvexPolygon<T, P = ()>(Polygon<T, P>);

// Property: random_between(n, max, &mut rng).iter().sum::<usize>() == max
pub fn random_between<R>(n: usize, max: usize, rng: &mut R) -> Vec<usize>
where
  R: Rng + ?Sized,
{
  debug_assert!(n > 0);
  assert!(n <= max);
  if max == n {
    return vec![1; n];
  }
  let mut pts = BTreeSet::new();
  while pts.len() < n - 1 {
    pts.insert(rng.gen_range(1..max));
  }
  let mut from = 0;
  let mut out = Vec::new();
  for x in pts.iter() {
    out.push(x - from);
    from = *x;
  }
  out.push(max - from);
  out
}

// Property: random_between_zero(10, 100, &mut rng).iter().sum::<isize>() == 0
// Property: random_between_zero(10, 100, &mut rng).iter().all(|v| !v.is_zero())
pub fn random_between_zero<R>(n: usize, max: usize, rng: &mut R) -> Vec<BigInt>
where
  R: Rng + ?Sized,
{
  assert!(n >= 2);
  let n_positive = rng.gen_range(1..n); // [1;n[
  dbg!(n_positive);
  let positive = random_between(n_positive, max, rng)
    .into_iter()
    .map(BigInt::from)
    .collect();
  let n_negative = n - n_positive;
  let negative: Vec<BigInt> = random_between(n_negative, max, rng)
    .into_iter()
    .map(BigInt::from)
    .map(Neg::neg)
    .collect();
  let mut result = [positive, negative].concat();
  result.shuffle(rng);
  result
  // random_between(n, max, rng)
  //   .into_iter()
  //   .map(BigInt::from)
  //   .zip(random_between(n, max, rng).into_iter().map(BigInt::from))
  //   .map(|(a, b)| a - b)
  //   .collect()
}

// Random vectors that sum to zero.
pub fn random_vectors<R>(n: usize, max: usize, rng: &mut R) -> Vec<Vector<BigRational, 2>>
where
  R: Rng + ?Sized,
{
  random_between_zero(n, max, rng)
    .into_iter()
    .zip(random_between_zero(n, max, rng).into_iter())
    .map(|(a, b)| Vector([BigRational::from(a), BigRational::from(b)]))
    .collect()
}

pub enum PointLocation {
  Inside,
  OnBoundary,
  Outside,
}

impl<T, P> ConvexPolygon<T, P>
where
  T: PolygonScalar,
  for<'a> &'a T: PolygonScalarRef<&'a T, T>,
{
  // O(log n)
  pub fn locate(&self, pt: &Point<T, 2>) -> PointLocation {
    debug_assert_ok!(self.validate());
    let poly = &self.0;
    let p0 = poly.vertex(0);
    let mut lower = 1;
    let mut upper = poly.points.len() as isize - 1;
    while lower + 1 < upper {
      let middle = (lower + upper) / 2;
      if p0.orientation(poly.vertex(middle), pt) == Orientation::CounterClockWise {
        lower = middle;
      } else {
        upper = middle;
      }
    }
    let p1 = poly.vertex(lower);
    let p2 = poly.vertex(upper);
    let triangle = data::TriangleView::new([p0, p1, p2]);
    triangle.locate(pt)
  }

  pub fn validate(&self) -> Result<(), Error> {
    let len = self.0.points.len() as isize;
    for i in 0..len {
      if self.0.vertex_orientation(i) != Orientation::CounterClockWise {
        return Err(Error::ConvexViolation);
      }
    }
    self.0.validate()
  }

  pub fn polygon(&self) -> &Polygon<T, P> {
    self.into()
  }
}

impl ConvexPolygon<BigRational> {
  /// ```rust
  /// # use rgeometry::*;
  /// # let convex = {
  /// let mut rng = rand::thread_rng();
  /// ConvexPolygon::random(3, 1000, &mut rng)
  /// # };
  /// # #[cfg(feature = "wasm")]
  /// # render_convex_polygon(convex);
  /// # return ()
  /// ```
  /// <iframe src="https://reanimate.clozecards.com:20443/loader.html?hash=36XCQBE0Yok="></iframe>
  pub fn random<R>(n: usize, max: usize, rng: &mut R) -> ConvexPolygon<BigRational>
  where
    R: Rng + ?Sized,
  {
    if n < 3 {
      // Return Result<P, Error> instead?
      return ConvexPolygon::random(3, max, rng);
    }
    let vs = {
      let mut vs = random_vectors(n, max, rng);
      Vector::sort_around(&mut vs);
      vs
    };
    let vertices: Vec<point::Point<BigRational, 2>> = vs
      .into_iter()
      .scan(Point::zero(), |st, vec| {
        *st += vec;
        Some(st.clone())
      })
      .collect();
    let n_vertices = (*vertices).len();
    debug_assert_eq!(n_vertices, n);
    let p = Polygon::new(vertices).unwrap();
    for i in 0..n {
      if p.vertex_orientation(i as isize) != Orientation::CounterClockWise {
        return Self::random(n, max, rng);
      }
    }
    let centroid = p.centroid();
    let t = Transform::translate(-Vector::from(centroid));
    let s = Transform::uniform_scale(BigRational::new(
      One::one(),
      BigInt::from_usize(max).unwrap(),
    ));
    ConvexPolygon(s * t * p)
  }
}

impl<T, P> From<ConvexPolygon<T, P>> for Polygon<T, P> {
  fn from(convex: ConvexPolygon<T, P>) -> Polygon<T, P> {
    convex.0
  }
}

impl<'a, T, P> From<&'a ConvexPolygon<T, P>> for &'a Polygon<T, P> {
  fn from(convex: &'a ConvexPolygon<T, P>) -> &'a Polygon<T, P> {
    &convex.0
  }
}

impl Distribution<ConvexPolygon<BigRational>> for Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ConvexPolygon<BigRational> {
    ConvexPolygon::random(100, usize::MAX, rng)
  }
}

#[cfg(test)]
mod tests;
