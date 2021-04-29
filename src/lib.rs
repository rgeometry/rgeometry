use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::collections::BTreeSet;

mod array;
mod point;
mod vector;

pub use point::Point;
pub use vector::Vector;

pub struct Polygon<T, P = ()> {
  pub points: Vec<Point<T, 2>>,
  pub boundary: u32,
  pub holes: Vec<u32>,
  pub meta: Vec<P>,
}

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
pub fn random_between_zero<R>(n: usize, max: usize, rng: &mut R) -> Vec<isize>
where
  R: Rng + ?Sized,
{
  random_between(n, max, rng)
    .iter()
    .zip(random_between(n, max, rng).iter())
    .map(|(a, b)| *a as isize - *b as isize)
    .collect()
}

// Random vectors that sum to zero.
pub fn random_vectors<R>(n: usize, max: usize, rng: &mut R) -> Vec<Vector<isize, 2>>
where
  R: Rng + ?Sized,
{
  random_between_zero(n, max, rng)
    .iter()
    .zip(random_between_zero(n, max, rng).iter())
    .map(|(a, b)| Vector([*a, *b]))
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
  pub fn random<R>(n: usize, max: usize, rng: &mut R) -> ConvexPolygon<isize>
  where
    R: Rng + ?Sized,
  {
    let _vs = {
      let mut vs = random_vectors(n, max, rng);
      Vector::sort_around(&mut vs);
      vs
    };
    unimplemented!();
    // ~(v:vs) <- coerce . sortAround origin . coerce <$> randomEdges n vMax
    // let vertices = fmap ((/ realToFrac vMax) . realToFrac) <$> scanl (.+^) (Point v) vs
    //     pRational = unsafeFromPoints $ map ext vertices
    //     Point c = centroid pRational
    //     pFinal = pRational & unsafeOuterBoundaryVector %~ CV.map (over core (.-^ c))
    // pure $ ConvexPolygon pFinal
  }
}

impl Distribution<ConvexPolygon<isize>> for Standard {
  fn sample<R: Rng + ?Sized>(&self, _rng: &mut R) -> ConvexPolygon<isize> {
    unimplemented!();
  }
}

#[cfg(test)]
mod tests;
