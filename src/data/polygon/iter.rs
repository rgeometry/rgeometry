use claim::debug_assert_ok;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::*;
use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use rand::Rng;
use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::iter::Zip;
use std::ops::*;

use crate::array::Orientation;
use crate::data;
use crate::data::line_segment::*;
use crate::data::Point;
use crate::data::PointLocation;
use crate::data::{Vector, VectorView};
use crate::transformation::*;
use crate::{Error, PolygonScalar, PolygonScalarRef};

pub struct Iter<'a, T: 'a, P: 'a> {
  pub(crate) iter: Zip<std::slice::Iter<'a, Point<T, 2>>, std::slice::Iter<'a, P>>,
}
impl<'a, T, P> Iterator for Iter<'a, T, P> {
  type Item = (&'a Point<T, 2>, &'a P);
  fn next(&mut self) -> Option<(&'a Point<T, 2>, &'a P)> {
    self.iter.next()
  }
}

pub struct IterMut<'a, T: 'a, P: 'a> {
  pub(crate) points: std::slice::IterMut<'a, Point<T, 2>>,
  pub(crate) meta: std::slice::IterMut<'a, P>,
}

impl<'a, T, P> Iterator for IterMut<'a, T, P> {
  type Item = (&'a mut Point<T, 2>, &'a mut P);
  fn next(&mut self) -> Option<(&'a mut Point<T, 2>, &'a mut P)> {
    Some((self.points.next()?, self.meta.next()?))
  }
}

pub struct EdgeIter<'a, T: 'a, P: 'a, const N: usize> {
  pub(crate) at: usize,
  pub(crate) points: &'a [Point<T, N>],
  pub(crate) meta: &'a [P],
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
