use crate::data::Point;
use crate::data::Polygon;
use crate::{Error, PolygonScalar};

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
  pts.sort_unstable();
  pts.dedup();
  if pts.len() < 3 {
    return Err(Error::InsufficientVertices);
  }
  dbg!(&pts);
  let mut edges = EdgePolygon::new(rng, pts.len());
  let mut isects = IntersectionSet::new(pts.len());
  dbg!(edges.edges().collect::<Vec<Edge>>());
  for e1 in edges.edges() {
    for e2 in edges.edges() {
      if e1 < e2 {
        if let Some(isect) = intersects(&pts, &edges, e1, e2) {
          dbg!(isect);
          isects.push(isect)
        }
      }
    }
  }
  while let Some(isect) = isects.random(rng) {
    dbg!("untangle", isect);
    untangle(&pts, &mut edges, &mut isects, isect)
  }

  let mut out = vec![];
  let mut at = 0;
  while edges.links[at].next != 0 {
    out.push(pts[at].clone());
    at = edges.links[at].next;
  }
  Polygon::new(out)
}

fn intersects<T>(pts: &[Point<T, 2>], ep: &EdgePolygon, a: Edge, b: Edge) -> Option<Intersection>
where
  T: PolygonScalar,
{
  let (a0, a1) = ep.order(a);
  let (b0, b1) = ep.order(b);
  let o1 = Orientation::new(&pts[a0], &pts[a1], &pts[b0]);
  let o2 = Orientation::new(&pts[a1], &pts[a0], &pts[b1]);
  let o3 = Orientation::new(&pts[b0], &pts[b1], &pts[a0]);
  let o4 = Orientation::new(&pts[b1], &pts[b0], &pts[a1]);

  // dbg!(a, b, o1, o2, o3, o4);
  if o1 == Orientation::CoLinear && o2 == Orientation::CoLinear {
    if b.0 <= a.1 {
      Some(Intersection::new(a, b))
    } else {
      None
    }
  } else if o1 == o2 && o3 == o4 {
    Some(Intersection::new(a, b))
  } else if (o1 == Orientation::CoLinear || o3 == Orientation::CoLinear) && (a1 != b0 && b1 != a0) {
    Some(Intersection::new(a, b))
  } else {
    None
  }
}

type Vertex = usize;
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Edge(Vertex, Vertex);

impl Edge {
  fn new(a: Vertex, b: Vertex) -> Edge {
    Edge(std::cmp::min(a, b), std::cmp::max(a, b))
  }
}

#[derive(Copy, Clone, Debug)]
struct Intersection(Edge, Edge);

impl Intersection {
  fn new(a: Edge, b: Edge) -> Intersection {
    Intersection(std::cmp::min(a, b), std::cmp::max(a, b))
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
fn untangle<T: PolygonScalar>(
  pts: &[Point<T, 2>],
  edges: &mut EdgePolygon,
  set: &mut IntersectionSet,
  isect: Intersection,
) {
  let Intersection(a, b) = isect;
  let (a1, a2) = edges.order(a); // a1.next = a2
  let (b1, b2) = edges.order(b); // b1.next = b2

  if Orientation::is_colinear(&pts[a1], &pts[b1], &pts[b2]) && b.1 < a.1 {
    // b1 and b2 are on the line between a1 and a2.
    if b1 == a2 {
      dbg!("case1");
      // Case 1
      edges.swap_with_next(a2);
      set.remove_all(a);
      set.remove_all(b);
    } else if a1 == b2 {
      dbg!("case2");
      // Case 2
      edges.swap_with_next(b1);
      set.remove_all(a);
      set.remove_all(b);
    } else {
      if (a1 < a2) == (b1 < b2) {
        dbg!("case3");
        // Case 3
        edges.cut(b1);
        edges.connect_many([a1, b1, b2, a2]);
      } else {
        dbg!("case4");
        // Case 4
        edges.cut(b1);
        edges.connect_many([a1, b2, b1, a2]);
      }
      set.remove_all(a);
      set.remove_all(b);
    }
  } else {
    dbg!("case5");
    // Case 5
    edges.uncross(a1, b1);
    dbg!("remove_all a");
    set.remove_all(a);
    dbg!("remove_all b");
    set.remove_all(b);
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
      vertices: vertices,
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
    // self.remove_all(isect.edge0);
    // self.remove_all(isect.edge1);
    Some(Intersection::new(
      Edge::new(isect.edge0.vertex0, isect.edge0.vertex1),
      Edge::new(isect.edge1.vertex0, isect.edge1.vertex1),
    ))
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
    self.dense_rev.resize(elt + 1, usize::MAX);
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
    if self.dense.len() == 0 {
      return None;
    }
    let idx = rng.gen_range(0..self.dense.len());
    Some(self.dense[idx])
  }
}

struct EdgePolygon {
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
  type Item = Edge;
  fn next(&mut self) -> Option<Edge> {
    if self.nth < self.links.len() {
      let this = self.nth;
      self.nth += 1;
      Some(Edge::new(this, self.links[this].next))
    } else {
      None
    }
  }
}

impl EdgePolygon {
  fn new<R>(rng: &mut R, size: usize) -> EdgePolygon
  where
    R: Rng + ?Sized,
  {
    let mut locs: Vec<usize> = Vec::from_iter(0..size);
    locs.shuffle(rng);

    let mut vec = Vec::with_capacity(size);
    vec.resize(size, Link { prev: 0, next: 0 });

    let mut ret = EdgePolygon { links: vec };
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

  fn order(&self, edge: Edge) -> (Vertex, Vertex) {
    if self.links[edge.0].next == edge.1 {
      (edge.0, edge.1)
    } else {
      assert_eq!(self.links[edge.1].next, edge.0);
      (edge.1, edge.0)
    }
  }

  // prev <-> a <-> b <-> next =>
  // prev <-> b <-> a <-> next
  fn swap_with_next(&mut self, a_idx: usize) {
    let b_idx = self.links[a_idx].next;
    let prev_idx = self.links[a_idx].prev;
    let next_idx = self.links[b_idx].next;
    self.connect_many([prev_idx, b_idx, a_idx, next_idx]);
  }

  // prev -> a -> b -> next
  // prev -> next
  fn cut(&mut self, a_idx: usize) {
    let prev_idx = self.links[a_idx].prev;
    let b_idx = self.links[a_idx].next;
    let next_idx = self.links[b_idx].next;
    self.connect(prev_idx, next_idx);
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
    while at != a_next {
      let prev = self.links[at].prev;
      let next = self.links[at].next;
      self.links[at].prev = next;
      self.links[at].next = prev;
      at = prev;
    }
    self.connect(a_idx, b_idx);
    self.connect(a_next, b_next);
  }
}

// impl Index for EdgePolygon {
//   type Output = Link
// }
