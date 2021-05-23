// use claim::debug_assert_ok;
use num_rational::BigRational;
use std::borrow::Borrow;
use std::ops::*;

use crate::array::Orientation;
use crate::data::Point;
use crate::data::Vector;
use crate::Error;
use crate::PolygonScalar;

mod iter;
pub use iter::*;

mod convex;
pub use convex::*;

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
  {
    // debug_assert_ok!(self.validate());
    let p1 = self.vertex(idx - 1);
    let p2 = self.vertex(idx);
    let p3 = self.vertex(idx + 1);
    p1.orientation(p2, p3)
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

  pub fn iter(&self) -> Iter<'_, T, P> {
    Iter {
      iter: self.points.iter().zip(self.meta.iter()),
    }
  }

  pub fn iter_mut(&mut self) -> IterMut<'_, T, P> {
    IterMut {
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
