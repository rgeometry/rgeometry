// use claim::debug_assert_ok;
use std::iter::Zip;
use std::ops::*;

use crate::data::line_segment::*;
use crate::data::DirectedEdge;
use crate::data::Point;

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

pub struct EdgeIter<'a, T: 'a, const N: usize> {
  pub(crate) at: usize,
  pub(crate) points: &'a [Point<T, N>],
}

impl<'a, T: Clone, const N: usize> Iterator for EdgeIter<'a, T, N> {
  type Item = DirectedEdge<T, N>;
  fn next(&mut self) -> Option<DirectedEdge<T, N>> {
    if self.at >= self.points.len() {
      return None;
    }
    let this_point = self.points.index(self.at).clone();
    let next_point = self.points.index((self.at + 1) % self.points.len()).clone();
    self.at += 1;
    Some(DirectedEdge {
      src: this_point,
      dst: next_point,
    })
  }
}
