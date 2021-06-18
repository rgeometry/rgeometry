use crate::data::EndPoint;
use crate::data::LineSegment;
use crate::data::LineSegmentView;
use crate::data::Point;
use crate::data::Polygon;
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
  let mut edges = EdgePolygon::new(&pts);
  let mut isects = IntersectionSet::new(pts.len());
  // dbg!(edges.sorted_edges());
  for e1 in edges.edges() {
    for e2 in edges.edges() {
      if e1 < e2 {
        if let Some(isect) = intersects(&pts, e1, e2) {
          isects.push(isect)
        }
      }
    }
  }
  // dbg!(isects.to_vec());
  while let Some(isect) = isects.random(rng) {
    untangle(&pts, &mut edges, &mut isects, isect)
  }

  edges.sort_vertices(&mut pts);
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

// untangle
//   Five distinct cases:
//       1. Edge 1 is before edge 2:
//               a1 -------> (a2/b1)
//                     b2 <- (a2/b1)
//                     b2 ------------> next
//          Solution:
//               a1 -> b2 -> (a2/b1) -> next
//       2. Edge 2 is before edge 1:
//            prev ------------> b1
//                    (a1/b2) <- b1
//                    (a1/b2) -------> a2
//          Solution:
//            prev -> (a1/b2) -> b1 -> a2
//       3. Edge 2 is inside edge 1: e1.1 -> (e2.1 -> e2.2) -> e1.2
//                                   e1.1 -> (e2.2 <- e2.1) -> e1.2
//             a1 -------------> a2
//                   b1 -> b2
//             prev-/        \-next
//          Solution:
//             a1 -> b1 -> b2 -> a2
//             prev ---------> next
//       4. Edge 2 is inside edge 1: e1.1 -> (e2.1 -> e2.2) -> e1.2
//                                   e1.1 -> (e2.2 <- e2.1) -> e1.2
//             a1 -------------> a2
//                   b2 -> b1
//             next-/        \-prev
//          Solution:
//             a1 -> b2 -> b1 -> a2
//             next <--------- prev
//       5. Edge 1 and Edge 2 crosses.
//                  b1 <- b.prev
//                  |
//             a1 --+--> a2 -> a.next
//                  |
//                  V
//                  b2
//           Solution
//               /->b1 -> b.prev
//              /
//             a1      /--a2 <- a.next
//                    /
//                   /
//                  b2 ->
//
// Swap:
//   a -> b
//   b -> a
// SetNext
// ReverseOrder
//

// untangle
//   Five distinct cases:
//       1. Edge 1 is before edge 2:
//               a1 -------> (a2/b1)
//                     b2 <- (a2/b1)
//                     b2 ------------> next
//          Solution:
//               a1 -> b2 -> (a2/b1) -> next
//       2. Edge 2 is before edge 1:
//            prev ------------> b1
//                    (a1/b2) <- b1
//                    (a1/b2) -------> a2
//          Solution:
//            prev -> (a1/b2) -> b1 -> a2
//       3. Edge 2 is inside edge 1: e1.1 -> (e2.1 -> e2.2) -> e1.2
//                                   e1.1 -> (e2.2 <- e2.1) -> e1.2
//             a1 -------------> a2
//                   b1 -> b2
//             prev-/        \-next
//          Solution:
//             a1 -> b1 -> b2 -> a2
//             prev ---------> next
//       4. Edge 2 is inside edge 1: e1.1 -> (e2.1 -> e2.2) -> e1.2
//                                   e1.1 -> (e2.2 <- e2.1) -> e1.2
//             a1 -------------> a2
//                   b2 <- b1
//             next-/        \-prev
//          Solution:
//             a1 -> b2 -> b1 -> a2
//             next <--------- prev
//       5. Edge 1 and Edge 2 crosses.
//                  b1 <- b.prev
//                  |
//             a1 --+--> a2 -> a.next
//                  |
//                  V
//                  b2
//           Solution
//               /->b1 -> b.prev
//              /
//             a1      /--a2 <- a.next
//                    /
//                   /
//                  b2 ->
fn untangle<T: PolygonScalar + std::fmt::Debug>(
  pts: &[Point<T, 2>],
  edges: &mut EdgePolygon,
  set: &mut IntersectionSet,
  isect: Intersection,
) {
  let Intersection(a, b) = isect;
  let da = edges.direct(a);
  let db = edges.direct(b);
  let DirectedEdge { src: a1, dst: a2 } = da; // a1.next = a2
  let DirectedEdge { src: b1, dst: b2 } = db; // b1.next = b2

  // assert!(&pts[a.0] <= &pts[b.0]);

  // If edges cross, uncross them.
  // If edges overlap:
  //   Find the two sets of linear points.
  //   If the sets are the same:
  //     connect prev to min, max to next.
  //   If the sets are different:
  //     connect s1.prev to min, max to s1. next.
  //     connect s2.prev to s2.next
  let are_overlapping = Orientation::is_colinear(&pts[a1], &pts[a2], &pts[b1])
    && Orientation::is_colinear(&pts[a1], &pts[a2], &pts[b2]);
  if are_overlapping {
    let (a_min, a_max) = edges.linear_extremes(pts, da);
    let (b_min, b_max) = edges.linear_extremes(pts, db);
    let mut mergable = None;
    'outer: for edge in edges.range(a_min, a_max).chain(edges.range(b_min, b_max)) {
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
    edges.hoist(elt, edge)
  } else {
    // Case 5
    edges.uncross(a1, b1);
  }
  for &edge in edges.removed_edges.iter() {
    set.remove_all(edge.into())
  }
  for &edge in edges.inserted_edges.iter() {
    for e1 in edges.edges() {
      if e1 != edge {
        if let Some(isect) = intersects(&pts, e1, edge) {
          set.push(isect)
        }
      }
    }
  }
  edges.removed_edges.clear();
  edges.inserted_edges.clear();
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

type SparseIndex = usize;
struct SparseVec<T> {
  dense: DenseCollection,
  arr: Vec<T>,
  free: Vec<SparseIndex>,
}

impl<T: Copy + Default> SparseVec<T> {
  fn new() -> SparseVec<T> {
    SparseVec {
      dense: DenseCollection::new(),
      arr: Vec::new(),
      free: Vec::new(),
    }
  }

  fn alloc(&mut self) -> SparseIndex {
    let idx = self.free.pop().unwrap_or(self.arr.len());
    self.dense.push(idx);
    idx
  }

  fn push(&mut self, elt: T) -> SparseIndex {
    let idx = self.alloc();
    self[idx] = elt;
    idx
  }

  fn remove(&mut self, idx: SparseIndex) -> T {
    self.dense.remove(idx);
    self.free.push(idx);
    let ret = self.arr[idx];
    self.arr[idx] = T::default();
    ret
  }

  fn random<R>(&self, rng: &mut R) -> Option<SparseIndex>
  where
    R: Rng + ?Sized,
  {
    self.dense.random(rng)
  }

  fn to_vec(&self) -> Vec<T> {
    let mut out = Vec::new();
    for &idx in self.dense.dense.iter() {
      out.push(self.arr[idx])
    }
    out
  }
}

impl<T: Copy + Default> Index<SparseIndex> for SparseVec<T> {
  type Output = T;
  fn index(&self, index: SparseIndex) -> &T {
    self.arr.index(index)
  }
}
impl<T: Copy + Default> IndexMut<SparseIndex> for SparseVec<T> {
  fn index_mut(&mut self, index: SparseIndex) -> &mut T {
    if index >= self.arr.len() {
      self.arr.resize(index + 1, T::default());
    }
    self.arr.index_mut(index)
  }
}

type Dense = usize;
type Sparse = usize;
struct DenseCollection {
  dense: Vec<Sparse>,
  dense_rev: Vec<Dense>,
}

impl DenseCollection {
  fn new() -> DenseCollection {
    DenseCollection {
      dense: Vec::new(),
      dense_rev: Vec::new(),
    }
  }

  fn push(&mut self, elt: Sparse) {
    let idx = self.dense.len();
    self.dense.push(elt);
    self
      .dense_rev
      .resize(std::cmp::max(self.dense_rev.len(), elt + 1), usize::MAX);
    self.dense_rev[elt] = idx;
  }

  // Swap the dense entry for 'elt' with the last entry.
  // Update reverse mapping for the swapped entry to point to the new idx.
  fn remove(&mut self, elt: Sparse) {
    let elt_dense_idx = self.dense_rev[elt];
    let last_sparse = *self.dense.last().unwrap();
    self.dense.swap_remove(elt_dense_idx);
    self.dense_rev[last_sparse] = elt_dense_idx;
  }

  fn random<R>(&self, rng: &mut R) -> Option<Sparse>
  where
    R: Rng + ?Sized,
  {
    if self.dense.is_empty() {
      return None;
    }
    let idx = rng.gen_range(0..self.dense.len());
    Some(self.dense[idx])
  }
}

struct EdgePolygon {
  inserted_edges: Vec<DirectedEdge>,
  removed_edges: Vec<DirectedEdge>,
  links: Vec<Link>,
}

#[derive(Copy, Clone, Debug)]
struct Link {
  prev: usize,
  next: usize,
}

struct EdgeIter<'a> {
  nth: usize,
  links: &'a Vec<Link>,
}

impl<'a> Iterator for EdgeIter<'a> {
  type Item = DirectedEdge;
  fn next(&mut self) -> Option<Self::Item> {
    if self.nth < self.links.len() {
      let this = self.nth;
      self.nth += 1;
      Some(DirectedEdge {
        src: this,
        dst: self.links[this].next,
      })
    } else {
      None
    }
  }
}

impl EdgePolygon {
  fn new<T>(vertices: &[Point<T, 2>]) -> EdgePolygon {
    let size = vertices.len();
    let locs: Vec<usize> = (0..size).collect();

    let mut vec = Vec::with_capacity(size);
    vec.resize(size, Link { prev: 0, next: 0 });

    let mut ret = EdgePolygon {
      inserted_edges: Vec::new(),
      removed_edges: Vec::new(),
      links: vec,
    };
    for i in 0..size - 1 {
      ret.connect(locs[i], locs[i + 1]);
    }
    ret.connect(locs[size - 1], locs[0]);
    ret
  }

  fn edges(&self) -> EdgeIter<'_> {
    EdgeIter {
      nth: 0,
      links: &self.links,
    }
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
    self.inserted_edges.push(DirectedEdge {
      src: a_idx,
      dst: b_idx,
    })
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

  // // prev <-> a1 <-> a2 <-> next =>
  // // prev <-> a2 <-> a1 <-> next
  // fn swap_edge(&mut self, edge: DirectedEdge) -> Vec<DirectedEdge> {
  //   let prev_idx = self.links[edge.src].prev;
  //   let next_idx = self.links[edge.dst].next;
  //   self.connect_many([prev_idx, edge.dst, edge.src, next_idx])
  // }

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
    self.removed_edges.push(DirectedEdge {
      src: prev_idx,
      dst: pt,
    });
    self.removed_edges.push(DirectedEdge {
      src: pt,
      dst: next_idx,
    });
    self.removed_edges.push(edge);
    self.connect_many([edge.src, pt, edge.dst]);
  }

  // a   b'
  //   X
  // b   a'
  //         a    b'
  //         V    ^
  // next <- b    a' <- prev
  fn uncross(&mut self, a_idx: usize, b_idx: usize) {
    // reverse links between b and a'
    let a_next = self.links[a_idx].next;
    let b_next = self.links[b_idx].next;
    let mut at = b_idx;
    while at != a_idx {
      let prev = self.links[at].prev;
      let next = self.links[at].next;
      self.links[at].prev = next;
      self.links[at].next = prev;
      at = prev;
    }
    self.connect(a_idx, b_idx);
    self.connect(a_next, b_next);
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

// impl Index for EdgePolygon {
//   type Output = Link
// }
