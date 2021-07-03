// use claim::debug_assert_ok;
use num::BigInt;
use num::BigRational;
use num_traits::*;
use std::iter::Sum;
use std::ops::*;

use crate::data::{DirectedEdge, Point, PointLocation, TriangleView, Vector};
use crate::{Error, Orientation, PolygonScalar};

mod iter;
pub use iter::*;

mod convex;
pub use convex::*;

use super::Transform;

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

// DONE: intersections : &[impl Into<LineSegment>] -> impl Iterator<Item =(&LineSegment,&LineSegment, ILineSegment)>
// cut_holes: Polygon -> PolygonSimple
// cut_holes: Polygon -> Vec<PointId>
// triangulate: Vec<Point<T,2>> + Vec<PointId> -> Vec<(PointId,PointId,PointId)>
// triangulate: Polygon -> Vec<Polygon>
// triangulate: Polygon -> Vec<(PointId, PointId, PointId)>

#[derive(Debug, Clone)]
pub struct Polygon<T> {
  // Use points: Arc<Vec<Point<T, 2>>>, ?
  // Key: PointId
  pub(crate) points: Vec<Point<T, 2>>,
  // Key: PointId
  pub(crate) ring_index: Vec<RingId>,
  // Key: PointId
  pub(crate) position_index: Vec<PositionId>,
  // Ring 0 is boundary and ccw
  // Ring n+1 is a hole and cw.
  // Outer key: RingId
  // Inner key: PositionId
  pub(crate) rings: Vec<Vec<PointId>>,
}

impl<T> Default for Polygon<T> {
  fn default() -> Polygon<T> {
    Polygon {
      points: Vec::default(),
      ring_index: Vec::default(),
      position_index: Vec::default(),
      rings: Vec::default(),
    }
  }
}

impl<T> Polygon<T> {
  /// $O(1)$
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
    }
  }

  /// $O(n^2)$
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
    if self.orientation() != Orientation::CounterClockWise {
      return Err(Error::ClockWiseViolation);
    }
    // Has no self intersections.
    // XXX: Hm, allow overlapping (but not crossing) edges in the weakly check?
    let edges: Vec<DirectedEdge<T, 2>> = self.iter_boundary_edges().collect();
    let isects = crate::algorithms::segment_intersections(&edges).next();
    if isects.is_some() {
      return Err(Error::SelfIntersections);
    }
    Ok(())
  }

  pub fn centroid(&self) -> Point<T, 2>
  where
    T: PolygonScalar,
  {
    let xs: Vector<T, 2> = self
      .iter_boundary_edges()
      .map(|edge| {
        let p = &edge.src.as_vec().cast(|v| v);
        let q = &edge.dst.as_vec().cast(|v| v);
        (p + q) * (p.0[0].clone() * q.0[1].clone() - q.0[0].clone() * p.0[1].clone())
      })
      .sum();
    let three = T::from_usize(3).unwrap();
    Point::from(xs / (three * self.signed_area_2x()))
  }

  pub fn bounding_box(&self) -> (Point<T, 2>, Point<T, 2>)
  where
    T: PolygonScalar,
  {
    assert!(!self.rings[0].is_empty());
    let mut min_x = self.points[self.rings[0][0].usize()].x_coord().clone();
    let mut max_x = self.points[self.rings[0][0].usize()].x_coord().clone();
    let mut min_y = self.points[self.rings[0][0].usize()].y_coord().clone();
    let mut max_y = self.points[self.rings[0][0].usize()].y_coord().clone();
    for pt in self.iter_boundary() {
      if pt.point().x_coord() < &min_x {
        min_x = pt.point().x_coord().clone();
      }
      if pt.point().x_coord() > &max_x {
        max_x = pt.point().x_coord().clone();
      }
      if pt.point().y_coord() < &min_y {
        min_y = pt.point().y_coord().clone();
      }
      if pt.point().y_coord() > &max_y {
        max_y = pt.point().y_coord().clone();
      }
    }
    (Point::new([min_x, min_y]), Point::new([max_x, max_y]))
  }

  // Convert to BigRational. Center on <0,0>. Scale size such that max(width,height) = 1.
  pub fn normalize(&self) -> Polygon<BigRational>
  where
    T: PolygonScalar + Into<BigInt>,
    T::ExtendedSigned: Into<BigInt>,
  {
    let (min, max) = self.bounding_box();
    let [min_x, min_y] = min.array;
    let [max_x, max_y] = max.array;
    let width = max_x.extend_signed() - min_x.extend_signed();
    let height = max_y.extend_signed() - min_y.extend_signed();
    let ratio = std::cmp::max(width, height);
    let p = self.clone().cast(|t| BigRational::from(t.into()));
    let centroid = p.centroid();
    let t = Transform::translate(-Vector::from(centroid));
    let s = Transform::uniform_scale(BigRational::new(One::one(), ratio.into()));
    s * t * p
  }

  pub fn signed_area<F>(&self) -> F
  where
    T: PolygonScalar + Into<F>,
    F: NumOps<F, F> + Sum + FromPrimitive,
  {
    self.signed_area_2x::<F>() / F::from_usize(2).unwrap()
  }

  pub fn signed_area_2x<F>(&self) -> F
  where
    T: PolygonScalar + Into<F>,
    F: NumOps<F, F> + Sum,
  {
    self
      .iter_boundary_edges()
      .map(|edge| {
        let p = edge.src;
        let q = edge.dst;
        p.array[0].clone().into() * q.array[1].clone().into()
          - q.array[0].clone().into() * p.array[1].clone().into()
      })
      .sum()
  }

  pub fn orientation(&self) -> Orientation
  where
    T: PolygonScalar,
  {
    match self.iter_boundary().min_by(|a, b| a.point().cmp(b.point())) {
      None => Orientation::CoLinear,
      Some(cursor) => cursor.orientation(),
    }
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
    match self.orientation() {
      Orientation::CounterClockWise => (),
      Orientation::CoLinear => panic!("Polygon is non-orientable."),
      Orientation::ClockWise => {
        let root_position = Position {
          ring_id: RingId(0),
          position_id: PositionId(0),
          size: self.rings[0].len(),
        };
        self.vertices_reverse(root_position, root_position.prev())
      }
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

  // O(n)
  pub fn is_ear(&self) -> bool
  where
    T: PolygonScalar,
  {
    let trig =
      TriangleView::new_unchecked([self.prev().point(), self.point(), self.next().point()]);
    if trig.orientation() == Orientation::CounterClockWise {
      for pt in self.next().next().to(self.prev()) {
        if trig.locate(pt.point()) != PointLocation::Outside {
          return false;
        }
      }
      true
    } else {
      false
    }
  }

  pub fn orientation(&self) -> Orientation
  where
    T: PolygonScalar,
  {
    let p1 = self.prev().point();
    let p2 = self.point();
    let p3 = self.next().point();
    Point::orient(p1, p2, p3)
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

#[cfg(test)]
pub mod tests {
  use super::*;

  use proptest::prelude::*;
  use test_strategy::proptest;

  #[proptest]
  fn random_polygon(poly: Polygon<i8>) {
    prop_assert_eq!(poly.validate().err(), None);
  }

  #[cfg(not(debug_assertions))] // proxy for release builds.
  #[proptest]
  fn normalize_props(poly: Polygon<i8>) {
    // Debug builds are ~50x slower than release builds. Sigh.
    let norm = poly.normalize();
    prop_assert_eq!(norm.centroid(), Point::zero());
    let (min, max) = norm.bounding_box();
    let width = max.x_coord() - min.x_coord();
    let height = max.y_coord() - min.y_coord();
    prop_assert!(width == BigRational::one() || height == BigRational::one());
  }
}
