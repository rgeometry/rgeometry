use crate::data::Cursor;
use crate::data::EndPoint;
use crate::data::IndexEdge;
use crate::data::LineSegmentView;
use crate::data::Point;
use crate::data::PointId;
use crate::data::Polygon;
use crate::data::{IndexIntersection, IndexIntersectionSet};
use crate::Intersects;
use crate::{Error, PolygonScalar, TotalOrd};

use rand::Rng;
use std::collections::BTreeSet;
use std::ops::Bound::*;

use crate::Orientation;

pub fn resolve_self_intersections<T, R>(poly: &mut Polygon<T>, rng: &mut R) -> Result<(), Error>
where
  T: PolygonScalar,
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
  for e1 in edges(poly) {
    for e2 in edges(poly) {
      if e1 < e2 {
        if let Some(isect) = intersects(poly, e1, e2) {
          // eprintln!("Inserting new intersection: {:?} {:?}", e1, e2);
          isects.push(isect)
        }
      }
    }
  }
  // sanity_check(&poly, &isects);
  // dbg!(isects.to_vec());
  while let Some(isect) = isects.random(rng) {
    untangle(poly, &mut isects, isect)
  }
  poly.ensure_ccw()?;
  // poly.validate()?;
  Ok(())
}

// Create list of edges
// Find all intersections
/// Generate a valid polygon by connecting a set of points in such a way
/// that there are no self-intersections.
/// # Time complexity
/// $O(n^4)$
/// # Space complexity
/// $O(n^2)$
pub fn two_opt_moves<T, R>(pts: Vec<Point<T>>, rng: &mut R) -> Result<Polygon<T>, Error>
where
  T: PolygonScalar,
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

fn endpoint<T: TotalOrd>(a: PointId, b: PointId, c: PointId, t: T) -> EndPoint<T> {
  if a == b || a == c {
    EndPoint::Exclusive(t)
  } else {
    EndPoint::Inclusive(t)
  }
}

fn intersects<T>(poly: &Polygon<T>, a: IndexEdge, b: IndexEdge) -> Option<IndexIntersection>
where
  T: PolygonScalar,
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
fn untangle<T: PolygonScalar>(
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

    // A kink is a point which would shorten the circumference of the polygon
    // if it was removed.
    // If we can find a kink and an edge which contains it then we can move it
    // and shorten the circumference.
    let mut kinks = Vec::new();
    for elt in a_min.to(Included(a_max)).chain(b_min.to(Included(b_max))) {
      let inner_segment: LineSegmentView<T> = (elt.prev().point()..elt.next().point()).into();
      if elt.orientation() != Orientation::CoLinear || !inner_segment.contains(elt.point()) {
        // We're either at a corner:
        //   prev
        //    \
        //     x -- next
        // Or we're at u-turn:
        //   prev ->  x
        //   next <-  x
        // In both cases, the circumference will be shortened if we can remove 'x'.
        kinks.push(elt);
      }
    }

    let mut mergable = None;
    'outer: for edge in a_min.to(Excluded(a_max)).chain(b_min.to(Excluded(b_max))) {
      let segment = LineSegmentView::new(
        EndPoint::Exclusive(edge.point()),
        EndPoint::Exclusive(edge.next().point()),
      );
      for &kink in kinks.iter() {
        if segment.contains(kink.point()) {
          mergable = Some((kink, edge));
          break 'outer;
        }
      }
    }
    // if mergable.is_none() {
    //   dbg!(&kinks);
    //   dbg!(a_min.point());
    //   dbg!(a_max.point());
    //   dbg!(b_min.point());
    //   dbg!(b_max.point());
    //   for (nth, edge) in a_min.to(a_max).chain(b_min.to(b_max)).enumerate() {
    //     dbg!(nth, edge.point(), edge.next().point());
    //   }
    // }
    // elt is not linear. That is, prev -> elt -> next is not a straight line.
    // Therefore, cutting it and adding to a straight line will shorten the polygon
    // circumference. Since there's a lower limit on the circumference, this algorithm
    // is guaranteed to terminate.
    let (kink, edge) = mergable.expect("There must be at least one mergable edge");
    // dbg!(elt, edge);
    // let elt_edges = vertex_list.vertex_edges(elt);
    // vertex_list.hoist(elt, edge);
    let del_edge_1 = IndexEdge::new(kink.prev().point_id(), kink.point_id());
    let del_edge_2 = IndexEdge::new(kink.point_id(), kink.next().point_id());
    let del_edge_3 = IndexEdge::new(edge.point_id(), edge.next().point_id());
    // eprintln!(
    //   "Del edges: {:?} {:?} {:?}",
    //   del_edge_1, del_edge_2, del_edge_3
    // );
    set.remove_all(del_edge_1);
    set.remove_all(del_edge_2);
    set.remove_all(del_edge_3);

    inserted_edges = vec![
      IndexEdge::new(kink.prev().point_id(), kink.next().point_id()),
      IndexEdge::new(edge.point_id(), kink.point_id()),
      IndexEdge::new(kink.point_id(), edge.next().point_id()),
    ];

    let p1 = edge.position;
    let p2 = kink.position;
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
    for e1 in edges(poly) {
      if e1 != edge {
        if let Some(isect) = intersects(poly, e1, edge) {
          // eprintln!("Inserting new intersection: {:?} {:?}", e1, edge);
          set.push(isect)
        }
      }
    }
  }
  // sanity_check(&poly, &set);
}

#[allow(dead_code)]
#[cfg(not(tarpaulin_include))]
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

#[cfg(not(tarpaulin_include))]
fn naive_intersection_set<T: PolygonScalar>(poly: &Polygon<T>) -> BTreeSet<IndexIntersection> {
  let mut set = BTreeSet::new();
  for e1 in edges(poly) {
    for e2 in edges(poly) {
      if e1 < e2 {
        if let Some(isect) = intersects(poly, e1, e2) {
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
  Point::orient(a1, a2, b1).is_colinear() && Point::orient(a1, a2, b2).is_colinear()
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
#[cfg(not(tarpaulin_include))]
pub mod tests {
  use super::*;

  use crate::testing::any_nn;
  use ordered_float::NotNan;
  use proptest::collection::vec;
  use proptest::prelude::*;
  use rand::prelude::SliceRandom;
  use rand::rngs::mock::StepRng;
  use rand::SeedableRng;
  use test_strategy::proptest;

  #[test]
  fn unit_1() {
    let pts: Vec<Point<i8>> = vec![
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
    let pts: Vec<Point<i8>> = vec![
      Point { array: [-59, -36] },
      Point { array: [-62, 88] },
      Point { array: [8, 124] },
      Point { array: [110, -81] },
      Point { array: [-93, 27] },
      Point { array: [96, 98] },
      Point { array: [66, 87] },
      Point { array: [-80, 20] },
      Point { array: [-21, -17] },
      Point { array: [-8, 21] },
      Point { array: [0, -4] },
      Point { array: [-63, 40] },
      Point { array: [-24, 78] },
      Point { array: [83, 23] },
      Point { array: [0, 93] },
      Point { array: [57, 52] },
      Point { array: [-87, -17] },
      Point { array: [38, 6] },
      Point { array: [0, -118] },
      Point {
        array: [-101, -119],
      },
      Point { array: [-30, 90] },
      Point { array: [0, -83] },
      Point {
        array: [-103, -112],
      },
      Point { array: [6, 75] },
      Point { array: [65, -18] },
      Point { array: [-126, 56] },
      Point { array: [-86, 97] },
      Point { array: [42, 44] },
      Point { array: [-128, 23] },
      Point { array: [-100, -53] },
      Point { array: [-85, 96] },
      Point { array: [120, 24] },
      Point { array: [74, 98] },
      Point { array: [63, -43] },
      Point { array: [-42, 45] },
      Point { array: [-2, 109] },
      Point { array: [-107, -94] },
      Point { array: [-12, 73] },
      Point { array: [99, 86] },
      Point { array: [62, 91] },
      Point { array: [-84, 81] },
      Point { array: [-128, 76] },
      Point { array: [-27, -45] },
      Point { array: [-56, 74] },
      Point { array: [-2, -59] },
      Point { array: [-65, 57] },
      Point { array: [-9, 66] },
      Point { array: [52, 40] },
      Point { array: [13, -70] },
      Point { array: [93, -1] },
      Point { array: [47, 38] },
      Point { array: [-85, -119] },
      Point { array: [-91, 52] },
      Point { array: [-107, 69] },
      Point { array: [31, -97] },
      Point { array: [118, 42] },
      Point { array: [61, -85] },
      Point { array: [0, 45] },
      Point { array: [-128, -48] },
      Point { array: [-94, 28] },
      Point { array: [-86, -56] },
      Point { array: [-128, 55] },
      Point { array: [0, 58] },
      Point { array: [75, -45] },
      Point { array: [-76, -21] },
      Point { array: [-10, -113] },
      Point { array: [-96, -21] },
      Point { array: [-84, 72] },
      Point { array: [100, -26] },
      Point { array: [-120, -50] },
      Point { array: [-94, -19] },
      Point { array: [17, -4] },
      Point { array: [56, -23] },
      Point { array: [11, 43] },
      Point { array: [-14, 57] },
      Point { array: [-42, -21] },
      Point { array: [0, -95] },
      Point { array: [8, 48] },
      Point { array: [-21, -46] },
      Point { array: [16, 81] },
      Point { array: [0, 120] },
      Point { array: [26, 27] },
      Point { array: [-69, -44] },
      Point { array: [97, 42] },
    ];
    let mut rng = StepRng::new(0, 0);
    // let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
    let ret = two_opt_moves(pts, &mut rng);

    assert_eq!(ret.and_then(|val| val.validate()).err(), None);
  }

  #[test]
  fn unit_3() {
    let pts: Vec<Point<i8>> = vec![
      Point { array: [0, 0] },
      Point { array: [2, 0] },
      Point { array: [1, 0] },
      Point { array: [3, 0] },
      Point { array: [3, 1] },
      Point { array: [0, 1] },
    ];
    let mut rng = StepRng::new(0, 0);
    let ret = two_opt_moves(pts, &mut rng);

    assert_eq!(ret.and_then(|val| val.validate()).err(), None);
  }

  #[proptest]
  fn points_to_polygon(#[strategy(vec(any::<Point<i8>>(), 3..100))] mut pts: Vec<Point<i8>>) {
    let mut set = BTreeSet::new();
    pts.retain(|pt| set.insert(*pt));
    if pts.len() >= 3 && !Point::all_colinear(&pts) {
      let mut rng = StepRng::new(0, 0);
      let ret = two_opt_moves(pts, &mut rng);
      prop_assert_eq!(ret.and_then(|val| val.validate()).err(), None);
    }
  }

  #[proptest]
  fn f64_to_polygon(#[strategy(vec(any_nn(), 3..100))] mut pts: Vec<Point<NotNan<f64>>>) {
    let mut set = BTreeSet::new();
    pts.retain(|pt| set.insert(*pt));
    if pts.len() >= 3 && !Point::all_colinear(&pts) {
      let mut rng = StepRng::new(0, 0);
      let ret = two_opt_moves(pts, &mut rng);
      prop_assert_eq!(ret.and_then(|val| val.validate()).err(), None);
    }
  }

  #[proptest]
  fn fuzz_f64(#[strategy(vec(any_nn(), 0..100))] pts: Vec<Point<NotNan<f64>>>) {
    let mut rng = StepRng::new(0, 0);
    two_opt_moves(pts, &mut rng).ok();
  }

  #[proptest]
  fn fuzz_i8(#[strategy(vec(any::<Point<i8>>(), 0..100))] pts: Vec<Point<i8>>) {
    let mut rng = StepRng::new(0, 0);
    two_opt_moves(pts, &mut rng).ok();
  }

  #[proptest]
  fn linear_fuzz(#[strategy(2..10_i8)] n: i8, seed: u64) {
    let mut linear: Vec<i8> = (0..n).collect();
    let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
    linear.shuffle(&mut rng);
    let mut pts: Vec<Point<i8>> = linear.iter().map(|&n| Point::new([n, 0])).collect();
    pts.push(Point::new([0, 1]));

    let mut rng = StepRng::new(0, 0);
    // dbg!(&linear);
    let ret = two_opt_moves(pts, &mut rng);
    prop_assert_eq!(ret.and_then(|val| val.validate()).err(), None);
  }
}
