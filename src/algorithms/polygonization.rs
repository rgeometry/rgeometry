use crate::data::Cursor;
use crate::data::DirectedIndexEdge;
use crate::data::EndPoint;
use crate::data::IndexEdge;
use crate::data::LineSegment;
use crate::data::LineSegmentView;
use crate::data::Point;
use crate::data::PointId;
use crate::data::Polygon;
use crate::data::{IndexIntersection, IndexIntersectionSet};
use crate::utils::*;
use crate::Intersects;
use crate::{Error, PolygonScalar};

use num_traits::NumOps;
use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::BTreeSet;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

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
          isects.push(isect)
        }
      }
    }
  }
  // dbg!(isects.to_vec());
  while let Some(isect) = isects.random(rng) {
    untangle(poly, &mut isects, isect)
  }
  poly.ensure_ccw();
  poly.validate()?;
  Ok(())
}

// Create list of edges
// Find all intersections
/// $O(n^4)$ Generate a random, valid polygon from a set of points.
pub fn two_opt_moves<T, R>(mut pts: Vec<Point<T, 2>>, rng: &mut R) -> Result<Polygon<T>, Error>
where
  T: PolygonScalar + std::fmt::Debug,
  R: Rng + ?Sized,
{
  // pts.sort_unstable();
  // pts.dedup();
  {
    let mut seen = BTreeSet::new();
    pts.retain(|pt| seen.insert(pt.clone()));
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
  e1.intersect(e2)?; // Returns Some(...) if there exist a point shared by both line segments.
                     // dbg!(e1, e2);
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
  eprintln!("Untangle: {:?}", isect);
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
    set.remove_all(del_edge_1);
    set.remove_all(del_edge_2);
    set.remove_all(del_edge_3);

    inserted_edges = vec![
      IndexEdge::new(elt.prev().point_id(), elt.next().point_id()),
      IndexEdge::new(edge.point_id(), elt.point_id()),
      IndexEdge::new(elt.point_id(), edge.point_id()),
    ];

    let p1 = edge.position;
    let p2 = elt.position;
    eprintln!("Hoist: {:?} {:?}", p1, p2);
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
    eprintln!("Uncross: {:?} {:?}", p1, p2);
    poly.vertices_reverse(p1, p2);
    // dbg!(&poly.rings[0]);
  }
  // dbg!(&removed_edges, &inserted_edges);
  for &edge in inserted_edges.iter() {
    for e1 in edges(&poly) {
      if e1 != edge {
        if let Some(isect) = intersects(&poly, e1, edge) {
          eprintln!("Inserting new intersection: {:?} {:?}", e1, edge);
          set.push(isect)
        }
      }
    }
  }
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
