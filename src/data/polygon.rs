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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PositionId(usize);

impl From<PositionId> for usize {
  fn from(pid: PositionId) -> usize {
    pid.0
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RingId(usize);

impl From<RingId> for usize {
  fn from(rid: RingId) -> usize {
    rid.0
  }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PointId(usize);

impl From<PointId> for usize {
  fn from(pid: PointId) -> usize {
    pid.0
  }
}

impl std::fmt::Debug for PointId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    f.write_fmt(format_args!("{}", self.0))
  }
}

impl PointId {
  pub const INVALID: PointId = PointId(usize::MAX);
  pub fn usize(self) -> usize {
    self.0
  }
}

#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexEdge {
  pub min: PointId,
  pub max: PointId,
}

impl std::fmt::Debug for IndexEdge {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    f.debug_tuple("IndexEdge")
      .field(&self.min)
      .field(&self.max)
      .finish()
  }
}

// Undirected Indexed Edge
impl IndexEdge {
  pub fn new(a: PointId, b: PointId) -> IndexEdge {
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
  pub src: PointId,
  pub dst: PointId,
}

// A     C
// B     D

// A*D   C*B

// intersections : &[impl Into<LineSegment>] -> impl Iterator<Item =(&LineSegment,&LineSegment, ILineSegment)>
// cut_holes: Polygon -> PolygonSimple
// cut_holes: Polygon -> Vec<PointId>
// triangulate: Vec<Point<T,2>> + Vec<PointId> -> Vec<(PointId,PointId,PointId)>
// triangulate: Polygon -> Vec<Polygon>
// triangulate: Polygon -> Vec<(PointId, PointId, PointId)>

#[derive(Debug, Clone)]
pub struct Polygon<T> {
  // Key: PointId
  pub(crate) points: Vec<Point<T, 2>>,
  pub(crate) ring_index: Vec<RingId>,         // Key: PointId
  pub(crate) position_index: Vec<PositionId>, // Key: PointId
  // Ring 0 is boundary and ccw
  // Ring n+1 is a hole and cw.
  pub(crate) rings: Vec<Vec<PointId>>,
  // pub(crate) vertices: Vec<Point<T, 2>>, // VertexId -> &Point<T,2>
  // pub(crate) order: Vec<VertexId>,       // Position -> VertexId
  // pub(crate) positions: Vec<PositionId>, // VertexId -> Position
  // pub(crate) boundary: usize,
  // pub(crate) holes: Vec<usize>,
}
// loops: Vec<Vec<VertexId>>
// vertex_loop: Vec<RingId>
// vertex_position: Vec<PositionId>

impl<T> Polygon<T> {
  pub fn new_unchecked(vertices: Vec<Point<T, 2>>) -> Polygon<T>
  where
    T: PolygonScalar,
  {
    let len = vertices.len();
    Polygon {
      points: vertices,
      ring_index: vec![RingId(0); len],
      position_index: (0..len).map(PositionId).collect(),
      rings: vec![(0..len).map(PointId).collect()],
      // vertices,
      // order: (0..len).map(VertexId).collect(),
      // positions: (0..len).map(PositionId).collect(),
      // boundary: len,
      // holes: vec![],
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
    assert!(!self.rings.is_empty());
    // Has at least three points.
    if self.rings[0].len() < 3 {
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

  pub fn point(&self, idx: PointId) -> &Point<T, 2> {
    &self.points[idx.0]
  }

  /// O(k) where k is the number of holes.
  pub fn cursor(&self, idx: PointId) -> Cursor<'_, T> {
    let ring_id = self.ring_index[idx.0];
    let position_id = self.position_index[idx.0];
    Cursor {
      polygon: &self,
      position: Position {
        ring_id,
        position_id,
        size: self.rings[ring_id.0].len(),
      },
    }
  }

  pub fn boundary_slice(&self) -> &[PointId] {
    &self.rings[0]
  }

  pub fn iter_boundary(&self) -> CursorIter<'_, T> {
    let root_cursor = Cursor {
      polygon: &self,
      position: Position {
        ring_id: RingId(0),
        position_id: PositionId(0),
        size: self.rings[0].len(),
      },
    };
    CursorIter {
      cursor_head: root_cursor,
      cursor_tail: root_cursor.prev(),
      exhausted: false,
    }
  }

  pub fn iter_boundary_edges(&self) -> EdgeIter<'_, T> {
    EdgeIter {
      iter: self.iter_boundary(),
    }
  }

  pub fn map_points<F>(mut self, f: F) -> Polygon<T>
  where
    T: Clone,
    F: Fn(Point<T, 2>) -> Point<T, 2>,
  {
    for pt in self.iter_mut() {
      *pt = f(pt.clone())
    }
    self
  }

  pub fn iter(&self) -> Iter<'_, T> {
    Iter {
      iter: self.points.iter(),
    }
  }

  pub fn iter_mut(&mut self) -> IterMut<'_, T> {
    IterMut {
      points: self.points.iter_mut(),
    }
  }

  pub fn cast<U, F>(self, f: F) -> Polygon<U>
  where
    T: Clone,
    U: Clone,
    F: Fn(T) -> U + Clone,
  {
    let pts = self.points.into_iter().map(|p| p.cast(f.clone())).collect();
    Polygon {
      points: pts,
      ring_index: self.ring_index,
      position_index: self.position_index,
      rings: self.rings,
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
      let root_position = Position {
        ring_id: RingId(0),
        position_id: PositionId(0),
        size: self.rings[0].len(),
      };
      self.vertices_reverse(root_position, root_position.prev())
    }
  }

  fn position_to_point_id(&self, position: Position) -> PointId {
    self.rings[position.ring_id.0][position.position_id.0]
  }

  fn swap_positions(&mut self, pa: Position, pb: Position) {
    assert_eq!(pa.ring_id, pb.ring_id);

    let ring = &mut self.rings[pa.ring_id.0];

    let pa_point_id: PointId = ring[pa.position_id.0];
    let pb_point_id: PointId = ring[pb.position_id.0];
    ring.swap(pa.position_id.0, pb.position_id.0);
    self.position_index.swap(pa_point_id.0, pb_point_id.0);
  }
}

impl From<Polygon<BigRational>> for Polygon<f64> {
  fn from(p: Polygon<BigRational>) -> Polygon<f64> {
    p.cast(|r| r.to_f64().unwrap())
  }
}
impl<'a> From<&'a Polygon<BigRational>> for Polygon<f64> {
  fn from(p: &Polygon<BigRational>) -> Polygon<f64> {
    p.clone().into()
  }
}

impl From<Polygon<f64>> for Polygon<BigRational> {
  fn from(p: Polygon<f64>) -> Polygon<BigRational> {
    p.cast(|f| BigRational::from_float(f).unwrap())
  }
}

impl<'a> From<&'a Polygon<f64>> for Polygon<BigRational> {
  fn from(p: &Polygon<f64>) -> Polygon<BigRational> {
    p.clone().into()
  }
}

#[derive(Debug)]
pub struct Cursor<'a, T> {
  polygon: &'a Polygon<T>,
  pub(crate) position: Position,
}

impl<'a, T> PartialEq for Cursor<'a, T> {
  fn eq(&self, other: &Cursor<'a, T>) -> bool {
    self.position == other.position
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
  pub fn point_id(self) -> PointId {
    self.polygon.position_to_point_id(self.position)
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
    &self.polygon.points[self.point_id().0]
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
  ring_id: RingId,         // Ring identifier
  size: usize,             // Ring size
  position_id: PositionId, // Position in ring. 0 <= position_id < size
}

impl Position {
  pub fn prev(mut self) -> Position {
    self.move_prev();
    self
  }

  pub fn next(mut self) -> Position {
    self.move_next();
    self
  }

  pub fn move_next(&mut self) {
    if self.position_id.0 == self.size - 1 {
      self.position_id.0 = 0;
    } else {
      self.position_id.0 += 1;
    }
  }

  pub fn move_prev(&mut self) {
    if self.position_id.0 == 0 {
      self.position_id.0 = self.size - 1;
    } else {
      self.position_id.0 -= 1;
    }
  }
}

// pub trait Triangulate {
//   type Iter: Iterator<Item = (VertexId, VertexId, VertexId)>;
//   fn triangulate(&self) -> Self::Iter;
// }
