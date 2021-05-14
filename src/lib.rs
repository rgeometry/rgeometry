use num_bigint::BigInt;
use num_rational::BigRational;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::collections::BTreeSet;

mod array;
mod point;
mod vector;

pub use array::Turn;
pub use point::Point;
pub use vector::Vector;

#[derive(Debug)]
pub struct Polygon<T, P = ()> {
  pub points: Vec<Point<T, 2>>,
  pub boundary: usize,
  pub holes: Vec<usize>,
  pub meta: Vec<P>,
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
    .map(|(a, b)| Vector([BigRational::from_integer(a), BigRational::from_integer(b)]))
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
  let len = (&vertices).len();
  ConvexPolygon(Polygon {
    points: vertices,
    boundary: len,
    holes: vec![],
    meta: vec![],
  })
}

impl Distribution<ConvexPolygon<BigRational>> for Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ConvexPolygon<BigRational> {
    random(100, usize::MAX, rng)
  }
}

#[cfg(test)]
mod tests;
