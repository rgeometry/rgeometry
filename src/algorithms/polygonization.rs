use crate::data::vertex::{DirectedEdge, Edge, Intersection, IntersectionSet, Vertex, VertexList};
use crate::data::EndPoint;
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
        if let Some(isect) = intersects(&pts, vertex_list.direct(e1), vertex_list.direct(e2)) {
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

fn intersects<T>(pts: &[Point<T, 2>], a: DirectedEdge, b: DirectedEdge) -> Option<Intersection>
where
  T: PolygonScalar + std::fmt::Debug,
{
  let e1 = LineSegmentView::from(&pts[a.src]..&pts[a.dst]);
  let e2 = LineSegmentView::from(&pts[b.src]..&pts[b.dst]);
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
  // dbg!(isect);
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
      DirectedEdge {
        src: elt_edges.0.src,
        dst: elt_edges.1.dst,
      },
      DirectedEdge {
        src: elt,
        dst: edge.dst,
      },
      DirectedEdge {
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
      DirectedEdge {
        src: da.src,
        dst: db.src,
      },
      DirectedEdge {
        src: da.dst,
        dst: db.dst,
      },
    ];
  }
  // dbg!(&removed_edges, &inserted_edges);
  for &edge in inserted_edges.iter() {
    for e1 in vertex_list.unordered_directed_edges() {
      if e1 != edge {
        if let Some(isect) = intersects(&pts, e1, edge) {
          set.push(isect)
        }
      }
    }
  }
}
