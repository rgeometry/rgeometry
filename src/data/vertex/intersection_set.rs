use super::Edge;
use super::Vertex;
use crate::utils::SparseIndex;
use crate::utils::SparseVec;

use rand::seq::SliceRandom;
use rand::Rng;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct Intersection {
  pub min: Edge,
  pub max: Edge,
}

impl Intersection {
  pub fn new(a: Edge, b: Edge) -> Intersection {
    Intersection {
      min: std::cmp::min(a, b),
      max: std::cmp::max(a, b),
    }
  }
}

impl From<Isect> for Intersection {
  fn from(isect: Isect) -> Intersection {
    Intersection::new(isect.edge0.into(), isect.edge1.into())
  }
}

pub struct IntersectionSet {
  vertices: usize,
  // This may grow to N^2 at the most (where N = number of vertices).
  by_idx: SparseVec<Isect>,
  by_edge: Vec<Option<SparseIndex>>,
}

impl IntersectionSet {
  /// O(n^2)
  pub fn new(vertices: usize) -> IntersectionSet {
    let size = vertices * (vertices - 1);
    let by_edge = vec![None; size];
    IntersectionSet {
      vertices,
      by_idx: SparseVec::new(),
      by_edge,
    }
  }

  // O(1)
  pub fn push(&mut self, isect: Intersection) {
    let idx = self.by_idx.push(Isect::new(isect));

    for &edge in &[isect.min, isect.max] {
      if let Some(loc) = self[edge] {
        assert_ne!(loc, idx);
        self.by_idx[loc][edge].prev = Some(idx);
        self.by_idx[idx][edge].next = Some(loc);
      }
      self[edge] = Some(idx);
    }
  }

  // O(1)
  fn remove(&mut self, idx: SparseIndex) {
    let isect = self.by_idx.remove(idx);

    for &edge in &[isect.edge0, isect.edge1] {
      match edge.prev {
        Some(prev) => self.by_idx[prev][Edge::from(edge)].next = edge.next,
        None => self[Edge::from(edge)] = edge.next,
      }
      if let Some(next) = edge.next {
        self.by_idx[next][Edge::from(edge)].prev = edge.prev;
      }
    }
  }

  // O(n) where N is the number of removed intersections.
  pub fn remove_all(&mut self, edge: impl Into<Edge>) {
    let key = edge.into();
    while let Some(idx) = self[key] {
      self.remove(idx)
    }
  }

  // O(1)
  pub fn random<R>(&mut self, rng: &mut R) -> Option<Intersection>
  where
    R: Rng + ?Sized,
  {
    let idx = self.by_idx.random(rng)?;
    let isect = self.by_idx[idx];
    Some(Intersection::new(
      Edge::new(isect.edge0.vertex0, isect.edge0.vertex1),
      Edge::new(isect.edge1.vertex0, isect.edge1.vertex1),
    ))
  }

  pub fn to_vec(&self) -> Vec<Intersection> {
    self
      .by_idx
      .to_vec()
      .into_iter()
      .map(|isect| isect.into())
      .collect()
  }
}

impl IndexMut<Edge> for IntersectionSet {
  fn index_mut(&mut self, index: Edge) -> &mut Option<SparseIndex> {
    self
      .by_edge
      .index_mut(index.max + self.vertices * index.min)
  }
}

impl Index<Edge> for IntersectionSet {
  type Output = Option<SparseIndex>;
  fn index(&self, index: Edge) -> &Option<SparseIndex> {
    self.by_edge.index(index.max + self.vertices * index.min)
  }
}

impl PartialEq<Edge> for IsectEdge {
  fn eq(&self, other: &Edge) -> bool {
    self.vertex0 == other.min && self.vertex1 == other.max
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
// FIXME: Should not be pub.
struct Isect {
  edge0: IsectEdge,
  edge1: IsectEdge,
}

impl Isect {
  fn new(intersection: Intersection) -> Isect {
    Isect {
      edge0: IsectEdge::new(intersection.min),
      edge1: IsectEdge::new(intersection.max),
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
// FIXME: Should not be pub
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
      vertex0: edge.min,
      vertex1: edge.max,
      next: None,
      prev: None,
    }
  }
}

impl From<IsectEdge> for Edge {
  fn from(e: IsectEdge) -> Edge {
    Edge::new(e.vertex0, e.vertex1)
  }
}
