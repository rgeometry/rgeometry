use crate::data::vertex::{Intersection, IntersectionSet, Vertex, VertexList};
use crate::data::Cursor;
use crate::data::DirectedIndexEdge;
use crate::data::EndPoint;
use crate::data::IndexEdge;
use crate::data::LineSegment;
use crate::data::LineSegmentView;
use crate::data::Point;
use crate::data::Polygon;
use crate::utils::*;
use crate::Intersects;
use crate::{Error, PolygonScalar};

use num_traits::NumOps;
use rand::seq::SliceRandom;
use rand::Rng;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use crate::Orientation;

// Create list of edges
// Find all intersections
/// O(n^4)
pub fn two_opt_moves<T, R>(mut pts: Vec<Point<T, 2>>, rng: &mut R) -> Result<Polygon<T>, Error>
where
  T: PolygonScalar + std::fmt::Debug,
  R: Rng + ?Sized,
{
  // pts.sort_unstable();
  // pts.dedup();
  if pts.len() < 3 {
    return Err(Error::InsufficientVertices);
  }
  if pts
    .iter()
    .all(|pt| Orientation::is_colinear(&pts[0], &pts[1], pt))
  {
    return Err(Error::InsufficientVertices);
  }
  // if all points are colinear, return error.
  // dbg!(&pts);
  let mut vertex_list = VertexList::new(pts.len());
  let mut isects = IntersectionSet::new(pts.len());
  // dbg!(edges.sorted_edges());
  for e1 in vertex_list.unordered_edges() {
    for e2 in vertex_list.unordered_edges() {
      if e1 < e2 {
        if let Some(isect) = intersects(&pts, e1, e2) {
          isects.push(isect)
        }
      }
    }
  }
  // dbg!(isects.to_vec());
  while let Some(isect) = isects.random(rng) {
    untangle(&pts, &mut vertex_list, &mut isects, isect)
  }

  vertex_list.sort_vertices(&mut pts);
  Polygon::new(pts)
}

pub fn two_opt_moves_poly<T, R>(pts: Vec<Point<T, 2>>, rng: &mut R) -> Result<Polygon<T>, Error>
where
  T: PolygonScalar + std::fmt::Debug,
  R: Rng + ?Sized,
{
  // pts.sort_unstable();
  // pts.dedup();
  if pts.len() < 3 {
    return Err(Error::InsufficientVertices);
  }
  if pts
    .iter()
    .all(|pt| Orientation::is_colinear(&pts[0], &pts[1], pt))
  {
    return Err(Error::InsufficientVertices);
  }
  // if all points are colinear, return error.
  // dbg!(&pts);
  let mut isects = IntersectionSet::new(pts.len());
  let mut poly = Polygon::new_unchecked(pts);
  // dbg!(edges.sorted_edges());
  for e1 in edges(&poly) {
    for e2 in edges(&poly) {
      if e1 < e2 {
        if let Some(isect) = intersects(&poly.vertices, e1, e2) {
          isects.push(isect)
        }
      }
    }
  }
  // dbg!(isects.to_vec());
  while let Some(isect) = isects.random(rng) {
    untangle_poly(&mut poly, &mut isects, isect)
  }
  poly.to_ccw();
  poly.validate()?;
  Ok(poly)
}

fn intersects<T>(pts: &[Point<T, 2>], a: IndexEdge, b: IndexEdge) -> Option<Intersection>
where
  T: PolygonScalar + std::fmt::Debug,
{
  let a_min = if a.min == b.min || a.min == b.max {
    EndPoint::Exclusive(&pts[a.min])
  } else {
    EndPoint::Inclusive(&pts[a.min])
  };
  let a_max = if a.max == b.min || a.max == b.max {
    EndPoint::Exclusive(&pts[a.max])
  } else {
    EndPoint::Inclusive(&pts[a.max])
  };
  let b_min = if b.min == a.min || b.min == a.max {
    EndPoint::Exclusive(&pts[b.min])
  } else {
    EndPoint::Inclusive(&pts[b.min])
  };
  let b_max = if b.max == a.min || b.max == a.max {
    EndPoint::Exclusive(&pts[b.max])
  } else {
    EndPoint::Inclusive(&pts[b.max])
  };
  let e1 = LineSegmentView::new(a_min, a_max);
  let e2 = LineSegmentView::new(b_min, b_max);
  e1.intersect(e2)?; // Returns Some(...) if there exist a point shared by both line segments.

  // dbg!(ret, &pts[a.src], &pts[a.dst], &pts[b.src], &pts[b.dst]);
  Some(Intersection::new(a.into(), b.into()))
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
  pts: &[Point<T, 2>],
  vertex_list: &mut VertexList,
  set: &mut IntersectionSet,
  isect: Intersection,
) {
  // dbg!(vertex_list.vertices().collect::<Vec<Vertex>>());
  // eprintln!("Untangle: {:?}", isect);
  let da = vertex_list.direct(isect.min);
  let db = vertex_list.direct(isect.max);

  let inserted_edges;

  if vertex_list.parallel(&pts, da, db) {
    let (a_min, a_max) = vertex_list.linear_extremes(pts, da);
    let (b_min, b_max) = vertex_list.linear_extremes(pts, db);
    let mut mergable = None;
    'outer: for edge in vertex_list
      .range(a_min, a_max)
      .chain(vertex_list.range(b_min, b_max))
    {
      for &elt in [a_min, a_max, b_min, b_max].iter() {
        let segment = LineSegmentView::new(
          EndPoint::Exclusive(&pts[edge.src]),
          EndPoint::Exclusive(&pts[edge.dst]),
        );
        if segment.contains(&pts[elt]) {
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
    let elt_edges = vertex_list.vertex_edges(elt);
    vertex_list.hoist(elt, edge);
    set.remove_all(elt_edges.0);
    set.remove_all(elt_edges.1);
    set.remove_all(edge);
    inserted_edges = vec![
      DirectedIndexEdge {
        src: elt_edges.0.src,
        dst: elt_edges.1.dst,
      },
      DirectedIndexEdge {
        src: elt,
        dst: edge.dst,
      },
      DirectedIndexEdge {
        src: edge.src,
        dst: elt,
      },
    ];
  } else {
    // eprintln!("Uncross");
    vertex_list.uncross(da, db);
    set.remove_all(da);
    set.remove_all(db);
    inserted_edges = vec![
      DirectedIndexEdge {
        src: da.src,
        dst: db.src,
      },
      DirectedIndexEdge {
        src: da.dst,
        dst: db.dst,
      },
    ];
  }
  // dbg!(&removed_edges, &inserted_edges);
  for &edge in inserted_edges.iter() {
    for e1 in vertex_list.unordered_directed_edges() {
      if e1 != edge {
        if let Some(isect) = intersects(&pts, e1.into(), edge.into()) {
          set.push(isect)
        }
      }
    }
  }
}

fn untangle_poly<T: PolygonScalar + std::fmt::Debug>(
  poly: &mut Polygon<T>,
  set: &mut IntersectionSet,
  isect: Intersection,
) {
  // dbg!(vertex_list.vertices().collect::<Vec<Vertex>>());
  // dbg!(isect);
  eprintln!("Poly order: {:?}", poly.order);
  eprintln!("Poly pos:   {:?}", poly.positions);
  eprintln!("Untangle: {:?}", isect);
  // let da = poly.direct(isect.min);
  // let db = poly.direct(isect.max);
  let da = poly.vertex(poly.direct(isect.min).src);
  let db = poly.vertex(poly.direct(isect.max).src);

  let inserted_edges;

  if parallel_edges(da, db) {
    let (a_min, a_max) = linear_extremes(da);
    let (b_min, b_max) = linear_extremes(db);

    let mut mergable = None;
    'outer: for edge in a_min.to(a_max).chain(b_min.to(b_max)) {
      for &elt in [a_min, a_max, b_min, b_max].iter() {
        let segment = LineSegmentView::new(
          EndPoint::Exclusive(edge.index()),
          EndPoint::Exclusive(edge.index_next()),
        );
        if segment.contains(elt.index()) {
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
    let del_edge_1 = IndexEdge::new(elt.prev().vertex_id(), elt.vertex_id());
    let del_edge_2 = IndexEdge::new(elt.vertex_id(), elt.next().vertex_id());
    let del_edge_3 = IndexEdge::new(edge.vertex_id(), edge.next().vertex_id());
    set.remove_all(del_edge_1);
    set.remove_all(del_edge_2);
    set.remove_all(del_edge_3);

    inserted_edges = vec![
      IndexEdge::new(elt.prev().vertex_id(), elt.next().vertex_id()),
      IndexEdge::new(edge.vertex_id(), elt.vertex_id()),
      IndexEdge::new(elt.vertex_id(), edge.vertex_id()),
    ];

    let p1 = edge.position;
    let p2 = elt.position;
    eprintln!("Hoist: {:?} {:?}", p1, p2);
    poly.vertices_join(p1, p2);
  } else {
    // vertex_list.uncross(da, db);
    set.remove_all(dbg!(IndexEdge::new(da.vertex_id(), da.next().vertex_id())));
    set.remove_all(dbg!(IndexEdge::new(db.vertex_id(), db.next().vertex_id())));

    inserted_edges = vec![
      IndexEdge::new(da.vertex_id(), db.vertex_id()),
      IndexEdge::new(da.next().vertex_id(), db.next().vertex_id()),
    ];

    let p1 = da.next().position;
    let p2 = db.position;
    eprintln!("Uncross: {:?} {:?}", p1, p2);
    poly.vertices_reverse(p1, p2);
  }
  // dbg!(&removed_edges, &inserted_edges);
  for &edge in inserted_edges.iter() {
    for e1 in edges(&poly) {
      if e1 != edge {
        if let Some(isect) = intersects(&poly.vertices, e1, edge) {
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
  let a1 = a.index();
  let a2 = a.index_next();
  let b1 = b.index();
  let b2 = b.index_next();
  Orientation::is_colinear(a1, a2, b1) && Orientation::is_colinear(a1, a2, b2)
}

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
    .map(|cursor| IndexEdge::new(cursor.vertex_id(), cursor.next().vertex_id()))
}
