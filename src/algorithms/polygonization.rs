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
  let mut vertex_list = VertexList::new(&pts);
  let mut isects = IntersectionSet::new(pts.len());
  // dbg!(edges.sorted_edges());
  for e1 in vertex_list.edges() {
    for e2 in vertex_list.edges() {
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

type Vertex = usize;
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge(Vertex, Vertex);

impl Edge {
  fn new(a: Vertex, b: Vertex) -> Edge {
    Edge(std::cmp::min(a, b), std::cmp::max(a, b))
  }
}

impl From<IsectEdge> for Edge {
  fn from(e: IsectEdge) -> Edge {
    Edge::new(e.vertex0, e.vertex1)
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct DirectedEdge {
  src: Vertex,
  dst: Vertex,
}

impl PartialOrd for DirectedEdge {
  fn partial_cmp(&self, other: &DirectedEdge) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}
impl Ord for DirectedEdge {
  fn cmp(&self, other: &DirectedEdge) -> std::cmp::Ordering {
    Edge::from(*self).cmp(&Edge::from(*other))
  }
}

impl From<DirectedEdge> for Edge {
  fn from(directed: DirectedEdge) -> Edge {
    Edge::new(directed.src, directed.dst)
  }
}

#[derive(Copy, Clone, Debug)]
struct Intersection(Edge, Edge);

impl Intersection {
  fn new(a: Edge, b: Edge) -> Intersection {
    Intersection(std::cmp::min(a, b), std::cmp::max(a, b))
  }
}

impl From<Isect> for Intersection {
  fn from(isect: Isect) -> Intersection {
    Intersection::new(isect.edge0.into(), isect.edge1.into())
  }
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
  // dbg!(vertex_list.sorted_edges());
  // dbg!(isect);
  let Intersection(a, b) = isect;
  let da = vertex_list.direct(a);
  let db = vertex_list.direct(b);

  let removed_edges;
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
    removed_edges = vec![elt_edges.0, elt_edges.1, edge];
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
    // Case 5
    // eprintln!("Uncross");
    vertex_list.uncross(da, db);
    removed_edges = vec![da, db];
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
  for &edge in removed_edges.iter() {
    set.remove_all(edge.into())
  }
  for &edge in inserted_edges.iter() {
    for e1 in vertex_list.edges() {
      if e1 != edge {
        if let Some(isect) = intersects(&pts, e1, edge) {
          set.push(isect)
        }
      }
    }
  }
}

struct IntersectionSet {
  vertices: usize,
  // This may grow to N^2 at the most (where N = number of vertices).
  intersections: SparseVec<Isect>,
  by_edge: Vec<Option<SparseIndex>>,
}

impl IntersectionSet {
  fn new(vertices: usize) -> IntersectionSet {
    let mut by_edge = Vec::new();
    by_edge.resize(vertices * (vertices - 1), None);
    IntersectionSet {
      vertices,
      intersections: SparseVec::new(),
      by_edge,
    }
  }

  // O(1)
  fn push(&mut self, isect: Intersection) {
    let idx = self.intersections.push(Isect::new(isect));

    for &edge in &[isect.0, isect.1] {
      if let Some(loc) = self[edge] {
        assert_ne!(loc, idx);
        self[loc][edge].prev = Some(idx);
        self[idx][edge].next = Some(loc);
      }
      self[edge] = Some(idx);
    }
  }

  fn remove(&mut self, idx: SparseIndex) {
    let isect = self.intersections.remove(idx);

    for &edge in &[isect.edge0, isect.edge1] {
      match edge.prev {
        Some(prev) => self[prev][edge].next = edge.next,
        None => self[edge] = edge.next,
      }
      if let Some(next) = edge.next {
        self[next][edge].prev = edge.prev;
      }
    }
  }

  fn remove_all(&mut self, edge: Edge) {
    while let Some(idx) = self[edge] {
      self.remove(idx)
    }
  }

  // O(n) where N is the number of vertices.
  fn random<R>(&mut self, rng: &mut R) -> Option<Intersection>
  where
    R: Rng + ?Sized,
  {
    let idx = self.intersections.random(rng)?;
    let isect = self[idx];
    Some(Intersection::new(
      Edge::new(isect.edge0.vertex0, isect.edge0.vertex1),
      Edge::new(isect.edge1.vertex0, isect.edge1.vertex1),
    ))
  }

  fn to_vec(&self) -> Vec<Intersection> {
    self
      .intersections
      .to_vec()
      .into_iter()
      .map(|isect| isect.into())
      .collect()
  }
}

impl IndexMut<SparseIndex> for IntersectionSet {
  fn index_mut(&mut self, index: SparseIndex) -> &mut Isect {
    self.intersections.index_mut(index)
  }
}

impl Index<SparseIndex> for IntersectionSet {
  type Output = Isect;
  fn index(&self, index: SparseIndex) -> &Isect {
    self.intersections.index(index)
  }
}

impl IndexMut<IsectEdge> for IntersectionSet {
  fn index_mut(&mut self, edge: IsectEdge) -> &mut Option<SparseIndex> {
    let index = edge.vertex1 + self.vertices * edge.vertex0;
    if index >= self.by_edge.len() {
      self.by_edge.resize(index + 1, None);
    }
    self.by_edge.index_mut(index)
  }
}

impl Index<IsectEdge> for IntersectionSet {
  type Output = Option<SparseIndex>;
  fn index(&self, index: IsectEdge) -> &Option<SparseIndex> {
    self
      .by_edge
      .index(index.vertex0 + self.vertices * index.vertex1)
  }
}

impl IndexMut<Edge> for IntersectionSet {
  fn index_mut(&mut self, index: Edge) -> &mut Option<SparseIndex> {
    self.by_edge.index_mut(index.1 + self.vertices * index.0)
  }
}

impl Index<Edge> for IntersectionSet {
  type Output = Option<SparseIndex>;
  fn index(&self, index: Edge) -> &Option<SparseIndex> {
    self.by_edge.index(index.1 + self.vertices * index.0)
  }
}

impl PartialEq<Edge> for IsectEdge {
  fn eq(&self, other: &Edge) -> bool {
    self.vertex0 == other.0 && self.vertex1 == other.1
  }
}

impl PartialEq for IsectEdge {
  fn eq(&self, other: &IsectEdge) -> bool {
    self.vertex0 == other.vertex0 && self.vertex1 == other.vertex1
  }
}

// Invariants:
//   Edge0 < Edge1
//   Edge.0 < Edge.1
#[derive(Clone, Copy, Debug, Default)]
struct Isect {
  edge0: IsectEdge,
  edge1: IsectEdge,
}

impl Isect {
  fn new(intersection: Intersection) -> Isect {
    Isect {
      edge0: IsectEdge::new(intersection.0),
      edge1: IsectEdge::new(intersection.1),
    }
  }
}

impl IndexMut<IsectEdge> for Isect {
  fn index_mut(&mut self, index: IsectEdge) -> &mut IsectEdge {
    if self.edge0 == index {
      &mut self.edge0
    } else {
      assert_eq!(self.edge1, index);
      &mut self.edge1
    }
  }
}

impl Index<IsectEdge> for Isect {
  type Output = IsectEdge;
  fn index(&self, index: IsectEdge) -> &IsectEdge {
    if self.edge0 == index {
      &self.edge0
    } else {
      assert_eq!(self.edge1, index);
      &self.edge1
    }
  }
}

impl IndexMut<Edge> for Isect {
  fn index_mut(&mut self, index: Edge) -> &mut IsectEdge {
    if self.edge0 == index {
      &mut self.edge0
    } else {
      assert_eq!(self.edge1, index);
      &mut self.edge1
    }
  }
}

impl Index<Edge> for Isect {
  type Output = IsectEdge;
  fn index(&self, index: Edge) -> &IsectEdge {
    if self.edge0 == index {
      &self.edge0
    } else {
      assert_eq!(self.edge1, index);
      &self.edge1
    }
  }
}

#[derive(Clone, Copy, Debug, Default)]
struct IsectEdge {
  vertex0: Vertex,
  vertex1: Vertex,
  next: Option<SparseIndex>,
  prev: Option<SparseIndex>,
}

impl IsectEdge {
  // FIXME: Sort vertex0 and vertex1
  fn new(edge: Edge) -> IsectEdge {
    IsectEdge {
      vertex0: edge.0,
      vertex1: edge.1,
      next: None,
      prev: None,
    }
  }
}

#[derive(Clone, Debug)]
struct VertexList {
  links: Vec<Link>,
}

#[derive(Copy, Clone, Debug)]
struct Link {
  prev: Vertex,
  next: Vertex,
}

impl VertexList {
  fn new<T>(vertices: &[Point<T, 2>]) -> VertexList {
    let size = vertices.len();

    let mut ret = VertexList {
      links: vec![Link { prev: 0, next: 0 }; size],
    };
    for i in 0..size {
      ret.connect(i, (i + 1) % size);
    }
    ret
  }

  fn edges(&self) -> impl Iterator<Item = DirectedEdge> + '_ {
    self
      .links
      .iter()
      .enumerate()
      .map(|(nth, link)| DirectedEdge {
        src: nth,
        dst: link.next,
      })
  }

  fn parallel<T: PolygonScalar>(
    &self,
    pts: &[Point<T, 2>],
    a: DirectedEdge,
    b: DirectedEdge,
  ) -> bool {
    Orientation::is_colinear(&pts[a.src], &pts[a.dst], &pts[a.src])
      && Orientation::is_colinear(&pts[a.src], &pts[a.dst], &pts[b.dst])
  }

  fn linear_extremes<T>(&self, pts: &[Point<T, 2>], edge: DirectedEdge) -> (Vertex, Vertex)
  where
    T: Clone + NumOps + Ord,
  {
    let mut min = edge.src;
    let mut max = edge.dst;
    let is_linear = |idx: Vertex| {
      Orientation::is_colinear(
        &pts[self.links[idx].prev],
        &pts[idx],
        &pts[self.links[idx].next],
      )
    };
    while is_linear(min) {
      min = self.links[min].prev;
      if min == edge.dst {
        panic!("All points are linear.")
      }
    }
    while is_linear(max) {
      max = self.links[max].next;
      if max == edge.src {
        panic!("All points are linear.")
      }
    }
    (min, max)
  }

  fn range(&self, mut min: Vertex, max: Vertex) -> impl Iterator<Item = DirectedEdge> + '_ {
    std::iter::from_fn(move || {
      if min != max {
        let edge = DirectedEdge {
          src: min,
          dst: self.links[min].next,
        };
        min = self.links[min].next;
        Some(edge)
      } else {
        None
      }
    })
  }

  // fn get(&mut self, idx: usize) -> &mut Link {
  //   &mut self.links[idx]
  // }

  fn connect(&mut self, a_idx: usize, b_idx: usize) {
    self.links[a_idx].next = b_idx;
    self.links[b_idx].prev = a_idx;
  }

  fn connect_many<const N: usize>(&mut self, indices: [usize; N]) {
    for i in 0..N - 1 {
      self.connect(indices[i], indices[i + 1])
    }
  }

  fn direct(&self, edge: Edge) -> DirectedEdge {
    if self.links[edge.0].next == edge.1 {
      DirectedEdge {
        src: edge.0,
        dst: edge.1,
      }
    } else {
      assert_eq!(self.links[edge.1].next, edge.0);
      assert_eq!(self.links[edge.0].prev, edge.1);
      DirectedEdge {
        src: edge.1,
        dst: edge.0,
      }
    }
  }

  fn vertex_edges(&self, pt: Vertex) -> (DirectedEdge, DirectedEdge) {
    let prev_idx = self.links[pt].prev;
    let next_idx = self.links[pt].next;
    (
      DirectedEdge {
        src: prev_idx,
        dst: pt,
      },
      DirectedEdge {
        src: pt,
        dst: next_idx,
      },
    )
  }

  // Turn:
  //
  // prev -> pt -> next
  // e1 ---------> e2
  //
  // Into:
  //
  // prev -------> next
  // e1 ---> pt -> e2
  //
  //
  fn hoist(&mut self, pt: Vertex, edge: DirectedEdge) {
    let prev_idx = self.links[pt].prev;
    let next_idx = self.links[pt].next;
    self.connect(prev_idx, next_idx);
    self.connect_many([edge.src, pt, edge.dst]);
    self.validate();
  }

  // a   b'
  //   X
  // b   a'
  //         a    b'
  //         V    ^
  // next <- b    a' <- prev
  fn uncross(&mut self, a: DirectedEdge, b: DirectedEdge) {
    // reverse links between b and a'
    assert_ne!(a, b);
    let mut at = b.src;
    while at != a.src {
      let prev = self.links[at].prev;
      let next = self.links[at].next;
      self.links[at].prev = next;
      self.links[at].next = prev;
      at = prev;
    }
    self.connect(a.src, b.src);
    self.connect(a.dst, b.dst);
    self.validate();
  }

  fn sort_vertices<T>(self, pts: &mut Vec<T>) {
    let mut at = 0;
    let mut nth = 0;
    while self.links[at].next != 0 {
      pts.swap(nth, at);
      at = self.links[at].next;
      nth += 1;
    }
  }

  fn validate(&self) {
    for (nth, link) in self.links.iter().enumerate() {
      debug_assert_eq!(self.links[link.next].prev, nth);
      debug_assert_eq!(self.links[link.prev].next, nth);
    }
  }

  fn sorted_edges(&self) -> Vec<Vertex> {
    let mut out = Vec::new();
    let mut at = 0;
    loop {
      out.push(at);
      at = self.links[at].next;
      if at == 0 {
        break;
      }
    }
    out
  }
}
