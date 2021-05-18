#![allow(unused_imports)]
// #![allow(incomplete_features)]
// #![feature(const_generics)]
// #![feature(const_evaluatable_checked)]
// use nalgebra::geometry::Point;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::identities::One;
use num_traits::FromPrimitive;
use num_traits::Num;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::iter::Sum;
use std::iter::Zip;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Div;
use std::ops::Index;
use std::ops::Mul;
use std::ops::Sub;

mod array;
mod linesegment;
mod matrix;
mod point;
mod transformation;
mod vector;

pub use array::Turn;
pub use linesegment::*;
pub use point::Point;
pub use transformation::*;
pub use vector::{Vector, VectorView};

pub trait PolygonScalar: Clone + Num + Sum + AddAssign + FromPrimitive {}
impl<T> PolygonScalar for T where T: Clone + Num + Sum + AddAssign + FromPrimitive {}

#[derive(Debug, Clone)]
pub struct Polygon<T, P = ()> {
  pub points: Vec<Point<T, 2>>,
  pub boundary: usize,
  pub holes: Vec<usize>,
  pub meta: Vec<P>,
}

impl<T> Polygon<T> {
  pub fn new(points: Vec<Point<T, 2>>) -> Polygon<T> {
    let len = points.len();
    let mut meta = Vec::with_capacity(len);
    meta.resize(len, ());
    Polygon {
      points,
      boundary: len,
      holes: vec![],
      meta,
    }
  }
}

impl<T, P> Polygon<T, P> {
  pub fn centroid(&self) -> Point<T, 2>
  where
    T: PolygonScalar,
  {
    let xs: Vector<T, 2> = self
      .iter_boundary_edges()
      .map(|edge| {
        let p = edge.0.inner().0.as_vec_ref();
        let q = edge.1.inner().0.as_vec_ref();
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

  pub fn iter_boundary_edges(&self) -> EdgeIter<'_, T, P, 2> {
    // let mut iter = self.iter();
    // let (this_point, this_meta) = iter.next().unwrap();
    EdgeIter {
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

  pub fn iter(&self) -> Zip<std::slice::Iter<'_, Point<T, 2>>, std::slice::Iter<'_, P>> {
    self.points.iter().zip(self.meta.iter())
  }

  pub fn iter_mut(&mut self) -> IterMut<'_, T, P> {
    IterMut {
      points: self.points.iter_mut(),
      meta: self.meta.iter_mut(),
    }
  }
}

pub struct EdgeIter<'a, T: 'a, P: 'a, const N: usize> {
  at: usize,
  points: &'a [Point<T, N>],
  meta: &'a [P],
}

impl<'a, T, P, const N: usize> Iterator for EdgeIter<'a, T, P, N> {
  type Item = LineSegmentView<'a, T, P, N>;
  fn next(&mut self) -> Option<LineSegmentView<'a, T, P, N>> {
    if self.at >= self.points.len() {
      return None;
    }
    let this_point = self.points.index(self.at);
    let this_meta = self.meta.index(self.at);
    let next_point = self.points.index((self.at + 1) % self.points.len());
    let next_meta = self.meta.index((self.at + 1) % self.points.len());
    self.at += 1;
    Some(LineSegmentView(
      EndPoint::Open((this_point, this_meta)),
      EndPoint::Closed((next_point, next_meta)),
    ))
  }
}

pub struct IterMut<'a, T: 'a, P: 'a> {
  points: std::slice::IterMut<'a, Point<T, 2>>,
  meta: std::slice::IterMut<'a, P>,
}

impl<'a, T, P> Iterator for IterMut<'a, T, P> {
  type Item = (&'a mut Point<T, 2>, &'a mut P);
  fn next(&mut self) -> Option<(&'a mut Point<T, 2>, &'a mut P)> {
    Some((self.points.next()?, self.meta.next()?))
  }
}

#[derive(Debug)]
pub struct ConvexPolygon<T, P = ()>(Polygon<T, P>);

// Property: random_between(n, max, &mut rng).iter().sum::<usize>() == max
pub fn random_between<R>(n: usize, max: usize, rng: &mut R) -> Vec<usize>
where
  R: Rng + ?Sized,
{
  if max < n + 1 {
    return vec![1; max];
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
pub fn random_between_zero<R>(n: usize, max: usize, rng: &mut R) -> Vec<BigInt>
where
  R: Rng + ?Sized,
{
  random_between(n, max, rng)
    .into_iter()
    .map(BigInt::from)
    .zip(random_between(n, max, rng).into_iter().map(BigInt::from))
    .map(|(a, b)| a - b)
    .collect()
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

impl<T, P> ConvexPolygon<T, P> {
  // data PointLocationResult = Inside | OnBoundary | Outside deriving (Show,Read,Eq)
  pub fn locate(self, _pt: &Point<T, 2>) -> PointLocation {
    let ConvexPolygon(_p) = self;
    unimplemented!();
  }
}

impl ConvexPolygon<BigRational> {
  pub fn random<R>(n: usize, max: usize, rng: &mut R) -> ConvexPolygon<BigRational>
  where
    R: Rng + ?Sized,
  {
    let vs = {
      let mut vs = random_vectors(n, max, rng);
      Vector::sort_around(&mut vs);
      vs
    };
    let vertices: Vec<point::Point<BigRational, 2>> = vs
      .into_iter()
      .scan(Point::zero(), |st, pt| {
        *st += pt;
        Some(st.clone())
      })
      .collect();
    let p = Polygon::new(vertices);
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
