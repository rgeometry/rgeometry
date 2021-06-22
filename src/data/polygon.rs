// use claim::debug_assert_ok;
use num_rational::BigRational;
use num_traits::*;
use ordered_float::{FloatIsNan, NotNan, OrderedFloat};
use std::borrow::Borrow;
use std::iter::ExactSizeIterator;
use std::iter::FromIterator;
use std::iter::Sum;
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

// pub type VertexId = usize;
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VertexId(pub usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PositionId(usize);
// pub type PositionId = usize;

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexEdge {
  pub min: VertexId,
  pub max: VertexId,
}

// Undirected Indexed Edge
impl IndexEdge {
  pub fn new(a: VertexId, b: VertexId) -> IndexEdge {
    IndexEdge {
      min: std::cmp::min(a, b),
      max: std::cmp::max(a, b),
    }
  }
}

impl From<DirectedIndexEdge> for IndexEdge {
  fn from(directed: DirectedIndexEdge) -> IndexEdge {
    IndexEdge::new(directed.src, directed.dst)
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirectedIndexEdge {
  pub src: VertexId,
  pub dst: VertexId,
}

#[derive(Debug, Clone)]
pub struct Polygon<T> {
  pub(crate) vertices: Vec<Point<T, 2>>, // VertexId -> &Point<T,2>
  pub(crate) order: Vec<VertexId>,
  pub(crate) positions: Vec<PositionId>,
  pub(crate) boundary: usize,
  pub(crate) holes: Vec<usize>,
}
// loops: Vec<Vec<VertexId>>
// positions: Vec<(LoopId, PositionId)>

impl<T> Polygon<T> {
  pub fn new_unchecked(vertices: Vec<Point<T, 2>>) -> Polygon<T>
  where
    T: PolygonScalar,
  {
    let len = vertices.len();
    Polygon {
      vertices,
      order: (0..len).map(VertexId).collect(),
      positions: (0..len).map(PositionId).collect(),
      boundary: len,
      holes: vec![],
    }
  }

  pub fn new(points: Vec<Point<T, 2>>) -> Result<Polygon<T>, Error>
  where
    T: PolygonScalar,
  {
    let mut p = Self::new_unchecked(points);
    p.ensure_ccw();
    p.validate()?;
    Ok(p)
  }
}

impl<T> Polygon<T> {
  // Validate that a polygon is simple.
  // https://en.wikipedia.org/wiki/Simple_polygon
  pub fn validate(&self) -> Result<(), Error>
  where
    T: PolygonScalar,
  {
    // Has no duplicate points.
    let mut seen = std::collections::BTreeSet::new();
    for pt in self.iter() {
      if !seen.insert(pt) {
        return Err(Error::DuplicatePoints);
      }
    }

    self.validate_weakly()
  }

  pub fn validate_weakly(&self) -> Result<(), Error>
  where
    T: PolygonScalar,
  {
    // Has at least three points.
    if self.vertices.len() < 3 {
      return Err(Error::InsufficientVertices);
    }
    // Is counter-clockwise
    if !self.signed_area_2x().is_positive() {
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
        let p = edge.src.as_vec();
        let q = edge.dst.as_vec();
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
        let p = edge.src;
        let q = edge.dst;
        p.array[0].clone() * q.array[1].clone() - q.array[0].clone() * p.array[1].clone()
      })
      .sum()
  }

  pub fn point(&self, idx: VertexId) -> &Point<T, 2> {
    &self.vertices[idx.0]
  }

  /// O(k) where k is the number of holes.
  pub fn cursor(&self, idx: VertexId) -> Cursor<'_, T> {
    // FIXME: Support holes.
    Cursor {
      polygon: &self,
      position: Position {
        position_id: self.vertex_to_position(idx),
        start: 0,
        end: self.boundary,
      },
    }
  }

  pub fn boundary_slice(&self) -> &[VertexId] {
    &self.order[0..self.boundary]
  }

  pub fn iter_boundary(&self) -> CursorIter<'_, T> {
    // FIXME: Use position 0 instead of vertex 0.
    CursorIter {
      cursor_head: self.cursor(VertexId(0)),
      cursor_tail: self.cursor(VertexId(0)).prev(),
      exhausted: false,
    }
  }

  pub fn iter_boundary_edges(&self) -> EdgeIter<'_, T> {
    EdgeIter {
      iter: self.iter_boundary(),
    }
  }

  pub fn map_points<F>(self, f: F) -> Polygon<T>
  where
    F: Fn(Point<T, 2>) -> Point<T, 2>,
  {
    let pts = self.vertices.into_iter().map(f).collect();
    Polygon {
      vertices: pts,
      order: self.order,
      positions: self.positions,
      boundary: self.boundary,
      holes: self.holes,
    }
  }

  pub fn iter(&self) -> Iter<'_, T> {
    Iter {
      iter: self.vertices.iter(),
    }
  }

  pub fn iter_mut(&mut self) -> IterMut<'_, T> {
    IterMut {
      points: self.vertices.iter_mut(),
    }
  }

  pub fn cast<U, F>(self, f: F) -> Polygon<U>
  where
    T: Clone,
    U: Clone,
    F: Fn(T) -> U + Clone,
  {
    let pts = self
      .vertices
      .into_iter()
      .map(|p| p.cast(f.clone()))
      .collect();
    Polygon {
      vertices: pts,
      order: self.order,
      positions: self.positions,
      boundary: self.boundary,
      holes: self.holes,
    }
  }

  // Reverse all points between p1 and p2, inclusive of p1 and p2.
  // For example: p1, a, b, c, p2 =>
  //              p2, c, b, a, p1
  pub fn vertices_reverse(&mut self, mut pt1: Position, mut pt2: Position) {
    loop {
      self.swap_positions(pt1, pt2);
      pt1.move_next();
      if pt1 == pt2 {
        break;
      }
      pt2.move_prev();
      if pt1 == pt2 {
        break;
      }
    }
  }

  // Move pt2 back until it is right in front of p1.
  pub fn vertices_join(&mut self, pt1: Position, mut pt2: Position) {
    while pt2.prev() != pt1 {
      let prev = pt2.prev();
      self.swap_positions(prev, pt2);
      pt2.position_id = prev.position_id;
    }
  }

  /// Panics if the edge isn't part of the polygon.
  pub fn direct(&self, edge: IndexEdge) -> DirectedIndexEdge {
    let min_cursor = self.cursor(edge.min);
    let max_cursor = self.cursor(edge.max);
    if min_cursor.next() == max_cursor {
      DirectedIndexEdge {
        src: edge.min,
        dst: edge.max,
      }
    } else if max_cursor.next() == min_cursor {
      DirectedIndexEdge {
        src: edge.max,
        dst: edge.min,
      }
    } else {
      panic!("Edge isn't part of polygon: {:?}", edge)
    }
  }

  pub fn ensure_ccw(&mut self)
  where
    T: PolygonScalar,
  {
    if !self.signed_area_2x().is_positive() {
      self.order.reverse();
      self.positions.reverse();
    }
  }

  fn position_to_vertex(&self, position: PositionId) -> VertexId {
    self.order[position.0]
  }

  fn vertex_to_position(&self, vertex: VertexId) -> PositionId {
    self.positions[vertex.0]
  }

  fn swap_positions(&mut self, pa: Position, pb: Position) {
    let pa_vertex_id: VertexId = self.position_to_vertex(pa.position_id);
    let pb_vertex_id: VertexId = self.position_to_vertex(pb.position_id);
    self.order.swap(pa.position_id.0, pb.position_id.0);
    self.positions.swap(pa_vertex_id.0, pb_vertex_id.0);
  }
}

impl From<Polygon<BigRational>> for Polygon<f64> {
  fn from(p: Polygon<BigRational>) -> Polygon<f64> {
    let pts = p.vertices.into_iter().map(|p| Point::from(&p)).collect();
    Polygon {
      vertices: pts,
      order: p.order,
      positions: p.positions,
      boundary: p.boundary,
      holes: p.holes,
    }
  }
}
impl<'a> From<&'a Polygon<BigRational>> for Polygon<f64> {
  fn from(p: &Polygon<BigRational>) -> Polygon<f64> {
    let pts = p.vertices.iter().map(Point::from).collect();
    Polygon {
      vertices: pts,
      order: p.order.clone(),
      positions: p.positions.clone(),
      boundary: p.boundary,
      holes: p.holes.clone(),
    }
  }
}

impl From<Polygon<f64>> for Polygon<BigRational> {
  fn from(p: Polygon<f64>) -> Polygon<BigRational> {
    let pts = p.vertices.into_iter().map(|p| Point::from(&p)).collect();
    Polygon {
      vertices: pts,
      order: p.order,
      positions: p.positions,
      boundary: p.boundary,
      holes: p.holes,
    }
  }
}

impl<'a> From<&'a Polygon<f64>> for Polygon<BigRational> {
  fn from(p: &Polygon<f64>) -> Polygon<BigRational> {
    let pts = p.vertices.iter().map(Point::from).collect();
    Polygon {
      vertices: pts,
      order: p.order.clone(),
      positions: p.positions.clone(),
      boundary: p.boundary,
      holes: p.holes.clone(),
    }
  }
}

#[derive(Debug)]
pub struct Cursor<'a, T> {
  polygon: &'a Polygon<T>,
  pub(crate) position: Position,
}

impl<'a, T> PartialEq for Cursor<'a, T> {
  fn eq(&self, other: &Cursor<'a, T>) -> bool {
    self.vertex_id() == other.vertex_id()
  }
}

// Can't derive it because T should not be 'Clone'.
impl<'a, T> Clone for Cursor<'a, T> {
  fn clone(&self) -> Self {
    Cursor {
      polygon: self.polygon,
      position: self.position,
    }
  }
}
impl<'a, T> Copy for Cursor<'a, T> {}

impl<'a, T> Cursor<'a, T> {
  pub fn vertex_id(self) -> VertexId {
    self.polygon.position_to_vertex(self.position.position_id)
  }

  pub fn prev(mut self) -> Cursor<'a, T> {
    self.move_prev();
    self
  }

  pub fn next(mut self) -> Cursor<'a, T> {
    self.move_next();
    self
  }

  pub fn point(self: Cursor<'a, T>) -> &'a Point<T, 2> {
    self.polygon.vertices.index(self.vertex_id().0)
  }

  pub fn move_next(&mut self) {
    self.position.move_next()
  }

  pub fn move_prev(&mut self) {
    self.position.move_prev()
  }

  pub fn orientation(&self) -> Orientation
  where
    T: PolygonScalar,
  {
    let p1 = self.prev().point();
    let p2 = self.point();
    let p3 = self.next().point();
    Orientation::new(p1, p2, p3)
  }

  pub fn is_colinear(&self) -> bool
  where
    T: PolygonScalar,
  {
    self.orientation() == Orientation::CoLinear
  }

  pub fn to(self: Cursor<'a, T>, end: Cursor<'a, T>) -> impl Iterator<Item = Cursor<'a, T>> + 'a {
    CursorIter {
      cursor_head: self,
      cursor_tail: end.prev(),
      exhausted: false,
    }
  }

  pub fn to_inclusive(self, end: Cursor<'a, T>) -> CursorIter<'a, T> {
    CursorIter {
      cursor_head: self,
      cursor_tail: end,
      exhausted: false,
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Position {
  position_id: PositionId, // start <= current < end
  start: usize,            // start of range, inclusive.
  end: usize,              // end of range, not inclusive.
}

impl Position {
  fn wrap(&self, index: usize) -> usize {
    if index >= self.end {
      index - self.end + self.start
    } else if index < self.start {
      self.end - (self.start - index)
    } else {
      index
    }
  }

  pub fn prev(mut self) -> Position {
    self.move_prev();
    self
  }

  pub fn next(mut self) -> Position {
    self.move_next();
    self
  }

  pub fn move_next(&mut self) {
    self.position_id.0 = self.wrap(self.position_id.0 + 1);
  }

  pub fn move_prev(&mut self) {
    if self.position_id.0 == 0 {
      self.position_id.0 = self.end - 1;
    } else {
      self.position_id.0 = self.wrap(self.position_id.0 - 1);
    }
  }
}
