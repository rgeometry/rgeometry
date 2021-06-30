use crate::data::Cursor;
use crate::data::EndPoint;
use crate::data::IndexEdge;
use crate::data::LineSegmentView;
use crate::data::Point;
use crate::data::PointId;
use crate::data::Polygon;
use crate::data::{IndexIntersection, IndexIntersectionSet};
use crate::Intersects;
use crate::{Error, PolygonScalar};

use rand::Rng;
use std::collections::BTreeSet;

use crate::Orientation;

pub fn resolve_self_intersections<T, R>(poly: &mut Polygon<T>, rng: &mut R) -> Result<(), Error>
where
  T: PolygonScalar + std::fmt::Debug,
  R: Rng + ?Sized,
{
  assert_eq!(poly.rings.len(), 1);
  if poly.iter_boundary().all(|pt| pt.is_colinear()) {
    return Err(Error::InsufficientVertices);
  }
  // if all points are colinear, return error.
  // dbg!(&pts);
  let mut isects = IndexIntersectionSet::new(poly.iter_boundary().len());
  // dbg!(&poly.rings[0]);
  for e1 in edges(&poly) {
    for e2 in edges(&poly) {
      if e1 < e2 {
        if let Some(isect) = intersects(&poly, e1, e2) {
          // eprintln!("Inserting new intersection: {:?} {:?}", e1, e2);
          isects.push(isect)
        }
      }
    }
  }
  sanity_check(&poly, &isects);
  // dbg!(isects.to_vec());
  while let Some(isect) = isects.random(rng) {
    untangle(poly, &mut isects, isect)
  }
  poly.ensure_ccw();
  // poly.validate()?;
  Ok(())
}

// Create list of edges
// Find all intersections
/// $O(n^4)$ Generate a random, valid polygon from a set of points.
pub fn two_opt_moves<T, R>(pts: Vec<Point<T, 2>>, rng: &mut R) -> Result<Polygon<T>, Error>
where
  T: PolygonScalar + std::fmt::Debug,
  R: Rng + ?Sized,
{
  {
    let mut seen = BTreeSet::new();
    for pt in pts.iter() {
      if !seen.insert(pt) {
        return Err(Error::DuplicatePoints);
      }
    }
  }
  if pts.len() < 3 {
    return Err(Error::InsufficientVertices);
  }
  let mut poly = Polygon::new_unchecked(pts);
  resolve_self_intersections(&mut poly, rng)?;
  Ok(poly)
}

fn endpoint<T>(a: PointId, b: PointId, c: PointId, t: T) -> EndPoint<T> {
  if a == b || a == c {
    EndPoint::Exclusive(t)
  } else {
    EndPoint::Inclusive(t)
  }
}

fn intersects<T>(poly: &Polygon<T>, a: IndexEdge, b: IndexEdge) -> Option<IndexIntersection>
where
  T: PolygonScalar + std::fmt::Debug,
{
  let a_min = endpoint(a.min, b.min, b.max, poly.point(a.min));
  let a_max = endpoint(a.max, b.min, b.max, poly.point(a.max));
  let b_min = endpoint(b.min, a.min, a.max, poly.point(b.min));
  let b_max = endpoint(b.max, a.min, a.max, poly.point(b.max));
  let e1 = LineSegmentView::new(a_min, a_max);
  let e2 = LineSegmentView::new(b_min, b_max);
  // dbg!(e1, e2);
  e1.intersect(e2)?; // Returns Some(...) if there exist a point shared by both line segments.
  Some(IndexIntersection::new(a, b))
}

// untangle takes two edges that overlap and finds a solution that reduces the circumference
// of the polygon. The solution is trivial when the two edges are not parallel: Simply uncross
// the edges and the new edge lengths are guaranteed to be smaller than before.
// If the edges are parallel, things get a bit more complicated.
// Two overlapping parallel edges must have at least one vertex that entirely contained by one
// of the edges. For example:
//    a1 -> a2
// b1 -> b2
//
// It's tempting to cut a1 and put it between b1 and b2. This doesn't change the edge lengths from
// b1 to b2 and it may decrease the edge length from a1.prev to a2. However, if a1 is on a straight
// line then we haven't reduced the circumference of the polygon at all. So, we need to find a point
// that doesn't lie on a straight line with its neighbours but still lies on the line given by the
// overlapping edges. Once this point has been found (along with its corresponding edge), it can be
// cut with the guarantee that it'll reduce the polygon circumference. Like this:
//
//     prev
//       |
//       a1 -----> a2
// b1 -------> b2
//
// Cut 'a1' and place it between 'b1' and 'b2':
//
//     prev
//        \------> a2
// b1 -> a1 -> b2
//
// The edge length from 'b1->b2' is identical to 'b1->a1->b2'.
// The edge length from 'prev->a1->a2' is always greater than 'prev -> a2'.
// We therefore know that the total circumference has decreased. QED.
fn untangle<T: PolygonScalar + std::fmt::Debug>(
  poly: &mut Polygon<T>,
  set: &mut IndexIntersectionSet,
  isect: IndexIntersection,
) {
  // dbg!(vertex_list.vertices().collect::<Vec<Vertex>>());
  // dbg!(isect);
  // eprintln!("Poly order: {:?}", poly.order);
  // eprintln!("Poly pos:   {:?}", poly.positions);
  // eprintln!("Untangle: {:?}", isect);
  // let da = poly.direct(isect.min);
  // let db = poly.direct(isect.max);
  let da = poly.cursor(poly.direct(isect.min).src);
  let db = poly.cursor(poly.direct(isect.max).src);

  let inserted_edges;

  if parallel_edges(da, db) {
    let (a_min, a_max) = linear_extremes(da);
    let (b_min, b_max) = linear_extremes(db);

    let mut mergable = None;
    'outer: for edge in a_min.to(a_max).chain(b_min.to(b_max)) {
      for &elt in [a_min, a_max, b_min, b_max].iter() {
        let segment = LineSegmentView::new(
          EndPoint::Exclusive(edge.point()),
          EndPoint::Exclusive(edge.next().point()),
        );
        if segment.contains(elt.point()) {
          mergable = Some((elt, edge));
          break 'outer;
        }
      }
    }
    // elt is not linear. That is, prev -> elt -> next is not a straight line.
    // Therefore, cutting it and adding to a straight line will shorten the polygon
    // circumference. Since there's a lower limit on the circumference, this algorithm
    // is guaranteed to terminate.
    let (elt, edge) = mergable.expect("There must be at least one mergable point");
    // dbg!(elt, edge);
    // let elt_edges = vertex_list.vertex_edges(elt);
    // vertex_list.hoist(elt, edge);
    let del_edge_1 = IndexEdge::new(elt.prev().point_id(), elt.point_id());
    let del_edge_2 = IndexEdge::new(elt.point_id(), elt.next().point_id());
    let del_edge_3 = IndexEdge::new(edge.point_id(), edge.next().point_id());
    // eprintln!(
    //   "Del edges: {:?} {:?} {:?}",
    //   del_edge_1, del_edge_2, del_edge_3
    // );
    set.remove_all(del_edge_1);
    set.remove_all(del_edge_2);
    set.remove_all(del_edge_3);

    inserted_edges = vec![
      IndexEdge::new(elt.prev().point_id(), elt.next().point_id()),
      IndexEdge::new(edge.point_id(), elt.point_id()),
      IndexEdge::new(elt.point_id(), edge.next().point_id()),
    ];

    let p1 = edge.position;
    let p2 = elt.position;
    // eprintln!("Hoist: {:?} {:?}", p1, p2);
    poly.vertices_join(p1, p2);
  } else {
    // vertex_list.uncross(da, db);
    set.remove_all(IndexEdge::new(da.point_id(), da.next().point_id()));
    set.remove_all(IndexEdge::new(db.point_id(), db.next().point_id()));

    inserted_edges = vec![
      IndexEdge::new(da.point_id(), db.point_id()),
      IndexEdge::new(da.next().point_id(), db.next().point_id()),
    ];

    let p1 = da.next().position;
    let p2 = db.position;
    // eprintln!("Uncross: {:?} {:?}", p1, p2);
    poly.vertices_reverse(p1, p2);
    // dbg!(&poly.rings[0]);
  }
  // dbg!(&removed_edges, &inserted_edges);
  // eprintln!("New edges: {:?}", &inserted_edges);
  for &edge in inserted_edges.iter() {
    for e1 in edges(&poly) {
      if e1 != edge {
        if let Some(isect) = intersects(&poly, e1, edge) {
          // eprintln!("Inserting new intersection: {:?} {:?}", e1, edge);
          set.push(isect)
        }
      }
    }
  }
  // sanity_check(&poly, &set);
}

#[allow(dead_code)]
fn sanity_check<T: PolygonScalar>(poly: &Polygon<T>, isects: &IndexIntersectionSet) {
  let naive_set = naive_intersection_set(poly);
  let fast_set = isects.iter().collect();
  let missing: Vec<&IndexIntersection> = naive_set.difference(&fast_set).collect();
  let extra: Vec<&IndexIntersection> = fast_set.difference(&naive_set).collect();
  if !missing.is_empty() {
    panic!("Fast set is too small! {:?}", missing);
  }
  if !extra.is_empty() {
    panic!("Fast set is too large! {:?}", extra);
  }
}
fn naive_intersection_set<T: PolygonScalar>(poly: &Polygon<T>) -> BTreeSet<IndexIntersection> {
  let mut set = BTreeSet::new();
  for e1 in edges(&poly) {
    for e2 in edges(&poly) {
      if e1 < e2 {
        if let Some(isect) = intersects(&poly, e1, e2) {
          set.insert(isect);
        }
      }
    }
  }
  set
}

fn parallel_edges<T>(a: Cursor<'_, T>, b: Cursor<'_, T>) -> bool
where
  T: PolygonScalar,
{
  let a1 = a.point();
  let a2 = a.next().point();
  let b1 = b.point();
  let b2 = b.next().point();
  Orientation::is_colinear(a1, a2, b1) && Orientation::is_colinear(a1, a2, b2)
}

/// Find the leftmost and rightmost vertices that are not linear.
/// Example:
///    a                    f
///     \- b - c - d -> e -/
/// linear_extremes('c') = ('b', 'e').
/// 'c' is linear (there's a straight line from 'b' to 'c' to 'd').
/// 'b' is not linear since the line from 'a' to 'b' to 'c' bends.
/// 'b' is therefore the first leftmost non-linear vertex and 'e'
/// is the first rightmost non-linear vertex.
fn linear_extremes<T>(at: Cursor<'_, T>) -> (Cursor<'_, T>, Cursor<'_, T>)
where
  T: PolygonScalar,
{
  let mut at_min = at;
  let mut at_max = at;
  at_max.move_next();
  while at_min.is_colinear() {
    at_min.move_prev();
  }
  while at_max.is_colinear() {
    at_max.move_next();
  }
  (at_min, at_max)
}

fn edges<T>(poly: &Polygon<T>) -> impl Iterator<Item = IndexEdge> + '_ {
  poly
    .iter_boundary()
    .map(|cursor| IndexEdge::new(cursor.point_id(), cursor.next().point_id()))
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use crate::testing::*;

  use proptest::collection::vec;
  use proptest::prelude::*;
  use rand::rngs::mock::StepRng;
  use rand::SeedableRng;

  #[test]
  fn unit_1() {
    let pts: Vec<Point<i8, 2>> = vec![
      Point { array: [-71, 91] },
      Point { array: [-17, -117] },
      Point { array: [-13, 98] },
      Point { array: [-84, 67] },
      Point { array: [-12, -92] },
      Point { array: [-95, 71] },
      Point { array: [-81, -2] },
      Point { array: [-91, -9] },
      Point { array: [-42, -66] },
      Point { array: [-107, 105] },
      Point { array: [-49, 9] },
      Point { array: [-96, 92] },
      Point { array: [42, 11] },
      Point { array: [-63, 56] },
      Point { array: [122, -53] },
      Point { array: [93, 29] },
      Point { array: [-93, 89] },
      Point { array: [40, -63] },
      Point { array: [-127, -44] },
      Point { array: [-108, 74] },
      Point { array: [96, -5] },
      Point { array: [46, 3] },
      Point { array: [-103, -94] },
      Point { array: [125, 73] },
      Point { array: [104, 60] },
      Point { array: [-55, -55] },
      Point { array: [-112, -42] },
      Point { array: [107, -16] },
      Point { array: [38, -111] },
      Point { array: [57, 123] },
      Point { array: [-107, 108] },
      Point { array: [46, -61] },
      Point { array: [0, -35] },
      Point { array: [35, -115] },
      Point { array: [-120, 31] },
      Point { array: [123, -87] },
      Point { array: [-22, -87] },
      Point { array: [-91, 27] },
      Point { array: [101, -6] },
      Point { array: [43, 6] },
      Point { array: [-31, -73] },
      Point {
        array: [-107, -115],
      },
      Point { array: [-60, -98] },
      Point { array: [-18, -94] },
      Point { array: [52, -22] },
      Point { array: [-71, -128] },
      Point { array: [80, -26] },
      Point { array: [104, -91] },
      Point { array: [-91, 45] },
      Point { array: [-79, -91] },
      Point { array: [-47, -124] },
      Point { array: [14, 101] },
      Point { array: [-21, -69] },
      Point { array: [16, 55] },
      Point { array: [105, -76] },
      Point { array: [-78, 39] },
      Point { array: [80, -114] },
      Point { array: [-6, 9] },
      Point { array: [-65, -104] },
      Point { array: [16, -1] },
      Point { array: [122, -67] },
      Point { array: [-93, -123] },
      Point {
        array: [-121, -120],
      },
      Point { array: [112, 32] },
      Point { array: [-87, -126] },
      Point { array: [-120, -38] },
      Point { array: [90, -111] },
    ];
    let ret = two_opt_moves(pts, &mut rand::rngs::SmallRng::seed_from_u64(0));

    assert_eq!(ret.and_then(|val| val.validate()).err(), None);
  }

  #[test]
  fn unit_2() {
    let pts: Vec<Point<i8, 2>> = vec![
      Point { array: [-17, 35] },
      Point { array: [-43, -87] },
      Point { array: [-61, -9] },
      Point { array: [111, -92] },
      Point {
        array: [-120, -104],
      },
      Point { array: [-33, 105] },
      Point { array: [-91, -50] },
      Point { array: [64, 70] },
      Point { array: [-94, -72] },
      Point { array: [-1, 42] },
      Point { array: [72, -67] },
      Point { array: [9, 12] },
      Point { array: [99, 28] },
      Point { array: [50, -98] },
      Point { array: [-119, -48] },
      Point { array: [-65, 9] },
      Point { array: [107, 28] },
      Point { array: [52, -38] },
      Point { array: [-27, 103] },
      Point { array: [-78, -37] },
      Point { array: [-107, -52] },
      Point { array: [108, -88] },
      Point { array: [29, -114] },
      Point { array: [-101, 69] },
      Point { array: [-23, -70] },
      Point { array: [68, -115] },
      Point { array: [-13, -41] },
      Point { array: [16, -128] },
      Point { array: [45, -91] },
      Point { array: [-16, -72] },
      Point { array: [100, 110] },
      Point { array: [38, -122] },
      Point { array: [-32, -127] },
      Point { array: [-42, 96] },
      Point { array: [124, -118] },
      Point { array: [77, -50] },
      Point { array: [15, 97] },
      Point { array: [23, 14] },
      Point { array: [-69, -12] },
      Point { array: [-27, 53] },
      Point { array: [-58, 91] },
      Point { array: [58, -21] },
      Point { array: [-105, 30] },
      Point { array: [122, -22] },
      Point { array: [109, 0] },
      Point { array: [-2, 42] },
      Point { array: [10, -84] },
      Point { array: [-87, 8] },
      Point { array: [53, 26] },
      Point { array: [112, -27] },
      Point { array: [-61, 9] },
      Point { array: [-58, -6] },
      Point { array: [-76, 8] },
      Point { array: [63, -82] },
      Point { array: [96, 106] },
      Point { array: [72, -123] },
      Point { array: [-78, -85] },
      Point { array: [22, 9] },
      Point { array: [22, 107] },
      Point { array: [40, -16] },
      Point { array: [81, -2] },
      Point { array: [46, 6] },
      Point { array: [-10, 120] },
      Point { array: [83, -51] },
      Point { array: [-54, 27] },
      Point { array: [-60, -3] },
      Point { array: [-81, 71] },
      Point { array: [-3, -80] },
      Point { array: [92, 39] },
      Point { array: [-103, 0] },
      Point { array: [-13, 40] },
      Point { array: [-110, 45] },
      Point { array: [96, 64] },
      Point { array: [-51, 91] },
      Point { array: [-108, 53] },
      Point { array: [44, 81] },
      Point { array: [56, 47] },
    ];
    // let mut rng = StepRng::new(0, 0);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
    let ret = two_opt_moves(pts, &mut rng);

    assert_eq!(ret.and_then(|val| val.validate()).err(), None);
  }

  proptest! {
    #[test]
    fn points_to_polygon(mut pts in vec(any_8(), 3..100)) {
      let mut set = BTreeSet::new();
      pts.retain(|pt| set.insert(pt.clone()));
      if pts.len() >= 3 {
        let mut rng = StepRng::new(0, 0);
        let ret = two_opt_moves(pts, &mut rng);
        prop_assert_eq!(ret.and_then(|val| val.validate()).err(), None);
      }
    }
  }
}
