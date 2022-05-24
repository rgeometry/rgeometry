// use claim::debug_assert_ok;
use num_traits::*;
use ordered_float::OrderedFloat;
use std::iter::Sum;
use std::ops::Bound::*;
use std::ops::*;

use crate::data::{
  DirectedEdge, HalfLineSoS, IHalfLineLineSegmentSoS::*, Point, PointLocation, TriangleView, Vector,
};
use crate::intersection::*;
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
  pub(crate) points: Vec<Point<T>>,
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
    // Verify minimum length before calling 'ensure_ccw'.
    if points.len() < 3 {
      return Err(Error::InsufficientVertices);
    }
    let mut p = Self::new_unchecked(points);
    p.ensure_ccw()?;
    p.validate()?;
    Ok(p)
  }

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
    // Rings is never allowed to be empty under any circumstances, even when
    // using 'Polygon::unchecked_new'.
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
    let edges: Vec<DirectedEdge<'_, T, 2>> = self.iter_boundary_edges().collect();
    let mut isects = crate::algorithms::segment_intersections(&edges);
    if isects.next().is_some() {
      return Err(Error::SelfIntersections);
    }
    Ok(())
  }

  pub fn locate(&self, origin: &Point<T, 2>) -> PointLocation
  where
    T: PolygonScalar,
  {
    // FIXME: Support polygons with holes.
    assert_eq!(
      self.rings.len(),
      1,
      "FIXME: Polygon::locate should support polygons with holes."
    );

    let direction = Vector::unit_right();
    let ray = HalfLineSoS::new_directed(origin, &direction);
    let mut intersections = 0;
    for edge in self.iter_boundary_edges() {
      if edge.contains(origin) {
        return PointLocation::OnBoundary;
      }
      if let Some(Crossing(lean)) = ray.intersect(edge) {
        // Only count crossing that aren't leaning to the right.
        if !lean.is_cw() {
          intersections += 1;
        }
      }
    }
    if intersections % 2 == 0 {
      PointLocation::Outside
    } else {
      PointLocation::Inside
    }
  }

  pub fn triangulate(
    &self,
  ) -> impl Iterator<Item = (Cursor<'_, T>, Cursor<'_, T>, Cursor<'_, T>)> + '_
  where
    T: PolygonScalar,
  {
    crate::algorithms::triangulation::earclip::earclip(self)
      .map(move |(p1, p2, p3)| (self.cursor(p1), self.cursor(p2), self.cursor(p3)))
  }

  //
  // # Panics
  //
  // May panic for bounded types (i8, isize, etc). Maybe produce inaccurate results for
  // floating point types (f32, f64).
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
    let three = T::from_constant(3);
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

  /// Computes the area of a polygon. If the polygon winds counter-clockwise,
  /// the area will be a positive number. If the polygon winds clockwise, the area will
  /// be negative.
  ///
  /// # Return type
  ///
  /// This function is polymorphic for the same reason as [`signed_area_2x`](Polygon::signed_area_2x).
  ///
  /// # Time complexity
  ///
  /// $O(n)$
  ///
  /// # Examples
  /// ```rust
  /// // The area of this polygon is 1/2 and won't fit in an `i32`
  /// # use rgeometry::data::*;
  /// # fn main() -> Result<(), rgeometry::Error> {
  /// let p = Polygon::new(vec![
  ///   Point::new([0, 0]),
  ///   Point::new([1, 0]),
  ///   Point::new([0, 1]),
  ///  ])?;
  /// assert_eq!(p.signed_area::<i32>(), 0);
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// ```rust
  /// // The area of this polygon is 1/2 and will fit in a `Ratio<i32>`.
  /// # use rgeometry::data::*;
  /// # use num_rational::Ratio;
  /// # fn main() -> Result<(), rgeometry::Error> {
  /// let p = Polygon::new(vec![
  ///   Point::new([0, 0]),
  ///   Point::new([1, 0]),
  ///   Point::new([0, 1]),
  ///  ])?;
  /// assert_eq!(p.signed_area::<Ratio<i32>>(), Ratio::from((1,2)));
  /// # Ok(())
  /// # }
  /// ```
  pub fn signed_area<F>(&self) -> F
  where
    T: PolygonScalar + Into<F>,
    F: NumOps<F, F> + Sum + FromPrimitive,
  {
    self.signed_area_2x::<F>() / F::from_usize(2).unwrap()
  }

  /// Compute double the area of a polygon. If the polygon winds counter-clockwise,
  /// the area will be a positive number. If the polygon winds clockwise, the area will
  /// be negative.
  ///
  /// Why compute double the area? If you are using integer coordinates then the doubled
  /// area will also be an integer value. Computing the exact requires a division which
  /// may leave you with a non-integer result.
  ///
  /// # Return type
  ///
  /// Storing the area of a polygon may require more bits of precision than are used
  /// for the coordinates. For example, if 32bit integers are used for point coordinates
  /// then you might need 65 bits to store the area doubled. For this reason, the return
  /// type is polymorphic and should be selected with care. If you are unsure which type
  /// is appropriate, use [`BigInt`](num::BigInt) or [`BigRational`](num::BigRational).
  ///
  /// # Time complexity
  ///
  /// $O(n)$
  ///
  /// # Examples:
  /// ```rust
  /// // The area of this polygon is 1/2 and cannot fit in an `i32` variable
  /// // so lets compute twice the area.
  /// # use rgeometry::data::*;
  /// # fn main() -> Result<(), rgeometry::Error> {
  /// let p = Polygon::new(vec![
  ///   Point::new([0, 0]),
  ///   Point::new([1, 0]),
  ///   Point::new([0, 1]),
  ///  ])?;
  /// assert_eq!(p.signed_area_2x::<i32>(), 1);
  /// # Ok(())
  /// # }
  /// ```
  ///
  /// ```rust
  /// // The area of a polygon may require more bits of precision than are
  /// // used for the coordinates.
  /// # use num::BigInt;
  /// # use rgeometry::data::*;
  /// # fn main() -> Result<(), rgeometry::Error> {
  /// let p = Polygon::new(vec![
  ///   Point::new([0, 0]),
  ///   Point::new([i32::MAX, 0]),
  ///   Point::new([0, i32::MAX]),
  ///  ])?;
  /// assert_eq!(p.signed_area_2x::<i64>(), 4611686014132420609_i64);
  /// # Ok(())
  /// # }
  /// ```
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

  /// Access point of a given vertex.
  /// # Time complexity
  /// $O(1)$
  pub fn point(&self, idx: PointId) -> &Point<T, 2> {
    &self.points[idx.0]
  }

  // FIXME: The PointId may not be valid. Return Option<Cursor>.
  /// Access cursor of a given vertex.
  /// # Time complexity
  /// $O(1)$
  pub fn cursor(&self, idx: PointId) -> Cursor<'_, T> {
    let ring_id = self.ring_index[idx.0];
    let position_id = self.position_index[idx.0];
    Cursor {
      polygon: self,
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
      polygon: self,
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

  #[must_use]
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

  pub fn map<U, F>(self, f: F) -> Polygon<U>
  where
    T: Clone,
    U: Clone,
    F: Fn(T) -> U + Clone,
  {
    let pts = self.points.into_iter().map(|p| p.map(f.clone())).collect();
    Polygon {
      points: pts,
      ring_index: self.ring_index,
      position_index: self.position_index,
      rings: self.rings,
    }
  }

  pub fn cast<U>(self) -> Polygon<U>
  where
    T: Clone + Into<U>,
  {
    let pts = self.points.into_iter().map(|p| p.cast()).collect();
    Polygon {
      points: pts,
      ring_index: self.ring_index,
      position_index: self.position_index,
      rings: self.rings,
    }
  }

  pub fn float(self) -> Polygon<OrderedFloat<f64>>
  where
    T: Clone + Into<f64>,
  {
    self.map(|v| OrderedFloat(v.into()))
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

  pub fn ensure_ccw(&mut self) -> Result<(), Error>
  where
    T: PolygonScalar,
  {
    match self.orientation() {
      Orientation::CounterClockWise => Ok(()),
      Orientation::CoLinear => Err(Error::CoLinearViolation),
      Orientation::ClockWise => {
        let root_position = Position {
          ring_id: RingId(0),
          position_id: PositionId(0),
          size: self.rings[0].len(),
        };
        self.vertices_reverse(root_position, root_position.prev());
        Ok(())
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

impl Polygon<OrderedFloat<f64>> {
  #[must_use]
  // Center on <0,0>. Scale size such that max(width,height) = 1.
  pub fn normalize(&self) -> Polygon<OrderedFloat<f64>> {
    let (min, max) = self.bounding_box();
    let [min_x, min_y] = min.array;
    let [max_x, max_y] = max.array;
    let width = max_x - min_x;
    let height = max_y - min_y;
    let ratio = std::cmp::max(width, height);
    let centroid = self.centroid();
    let t = Transform::translate(-Vector::from(centroid));
    let s = Transform::uniform_scale(ratio.recip());
    s * t * self
  }
}

#[derive(Debug)]
pub struct Cursor<'a, T> {
  polygon: &'a Polygon<T>,
  pub(crate) position: Position,
}

impl<'a, T> Deref for Cursor<'a, T> {
  type Target = Point<T, 2>;
  fn deref(&self) -> &Self::Target {
    self.point()
  }
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

  #[must_use]
  pub fn prev(mut self) -> Cursor<'a, T> {
    self.move_prev();
    self
  }

  #[must_use]
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
      for pt in self.next().next().to(Excluded(self.prev())) {
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

  pub fn to(self: Cursor<'a, T>, end: Bound<Cursor<'a, T>>) -> CursorIter<'a, T> {
    // FIXME: Check that self and end point to the same polygon.
    match end {
      Bound::Unbounded => CursorIter {
        cursor_head: self,
        cursor_tail: self.prev(),
        exhausted: false,
      },
      Bound::Included(end) => CursorIter {
        cursor_head: self,
        cursor_tail: end,
        exhausted: false,
      },
      Bound::Excluded(end) => CursorIter {
        cursor_head: self,
        cursor_tail: end.prev(),
        exhausted: false,
      },
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
  #[must_use]
  pub fn prev(mut self) -> Position {
    self.move_prev();
    self
  }

  #[must_use]
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

#[cfg(test)]
pub mod tests {
  use super::*;

  use crate::testing::*;
  use ordered_float::NotNan;
  use proptest::prelude::*;
  use proptest::proptest as proptest_block;
  use test_strategy::proptest;

  #[proptest]
  fn random_polygon(poly: Polygon<i8>) {
    prop_assert_eq!(poly.validate().err(), None);
  }

  #[proptest]
  fn fuzz_new_polygon(pts: Vec<Point<i8>>) {
    // make sure there's no input that can cause a panic. Err is okay, panic is not.
    Polygon::new(pts).ok();
  }

  #[proptest]
  fn fuzz_validate(pts: Vec<Point<i8>>) {
    // make sure there's no input that can cause a panic. Err is okay, panic is not.
    Polygon::new_unchecked(pts).validate().ok();
  }

  #[proptest]
  fn fuzz_validate_weakly(pts: Vec<Point<i8>>) {
    // make sure there's no input that can cause a panic. Err is okay, panic is not.
    Polygon::new_unchecked(pts).validate_weakly().ok();
  }

  proptest_block! {
    #[test]
    fn fuzz_centroid(poly in polygon_nn()) {
      poly.centroid();
    }
  }

  // // #[cfg(not(debug_assertions))] // proxy for release builds.
  // #[proptest]
  // fn normalize_props(poly: Polygon<i8>) {
  //   // Debug builds are ~50x slower than release builds. Sigh.
  //   let norm = poly.float().normalize();
  //   // prop_assert_eq!(norm.centroid(), Point::zero());
  //   let (min, max) = norm.bounding_box();
  //   let width = max.x_coord() - min.x_coord();
  //   let height = max.y_coord() - min.y_coord();
  //   // prop_assert!(width == OrderedFloat(1.0) || height == OrderedFloat(1.0));
  // }

  #[test]
  #[should_panic]
  fn locate_feature_fixme() {
    let mut poly: Polygon<i8> = Polygon::new(vec![
      Point { array: [0, 0] },
      Point { array: [-1, 127] },
      Point { array: [-50, 48] },
    ])
    .expect("valid polygon");
    // Add an hole (happens to be topologically invalid but that doesn't matter)
    poly.rings.push(vec![PointId(0), PointId(1), PointId(2)]);
    let origin = Point { array: [79, 108] };
    poly.locate(&origin);
  }

  #[test]
  fn locate_unit_1() {
    let poly: Polygon<i8> = Polygon::new(vec![
      Point { array: [0, 0] },
      Point { array: [-1, 127] },
      Point { array: [-50, 48] },
    ])
    .expect("valid polygon");
    let origin = Point { array: [79, 108] };

    assert_eq!(poly.locate(&origin), PointLocation::Outside);
  }

  #[test]
  fn locate_unit_2() {
    let poly: Polygon<i8> = Polygon::new(vec![
      Point { array: [-9, 83] },
      Point { array: [-28, 88] },
      Point { array: [-9, -75] },
      Point { array: [-2, -6] },
      Point { array: [-2, 11] },
    ])
    .expect("valid polygon");
    let origin = Point { array: [-9, 0] };

    assert_eq!(
      locate_by_triangulation(&poly, &origin),
      poly.locate(&origin)
    );
  }

  // Locate a point relative to a polygon. Should be identical to
  // Polygon::locate but slower.
  fn locate_by_triangulation<T>(poly: &Polygon<T>, origin: &Point<T, 2>) -> PointLocation
  where
    T: PolygonScalar,
  {
    for edge in poly.iter_boundary_edges() {
      if edge.contains(origin) {
        return PointLocation::OnBoundary;
      }
    }
    for (a, b, c) in poly.triangulate() {
      let trig = TriangleView::new([&a, &b, &c]).expect("valid triangle");
      if trig.locate(origin) != PointLocation::Outside {
        return PointLocation::Inside;
      }
    }
    return PointLocation::Outside;
  }

  #[proptest]
  fn locate_id_prop(poly: Polygon<i8>, origin: Point<i8, 2>) {
    prop_assert_eq!(
      locate_by_triangulation(&poly, &origin),
      poly.locate(&origin)
    )
  }
}
