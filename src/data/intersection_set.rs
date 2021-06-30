use crate::data::IndexEdge;
use crate::data::PointId;
use crate::utils::SparseIndex;
use crate::utils::SparseVec;

use rand::Rng;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub struct IndexIntersection {
  pub min: IndexEdge,
  pub max: IndexEdge,
}

impl std::fmt::Debug for IndexIntersection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    f.debug_tuple("IndexIntersection")
      .field(&self.min)
      .field(&self.max)
      .finish()
  }
}

impl IndexIntersection {
  pub fn new(a: IndexEdge, b: IndexEdge) -> IndexIntersection {
    IndexIntersection {
      min: std::cmp::min(a, b),
      max: std::cmp::max(a, b),
    }
  }
}

impl From<Isect> for IndexIntersection {
  fn from(isect: Isect) -> IndexIntersection {
    IndexIntersection::new(isect.edge0.into(), isect.edge1.into())
  }
}

pub struct IndexIntersectionSet {
  vertices: usize,
  // This may grow to N^2 at the most (where N = number of vertices).
  by_idx: SparseVec<Isect>,
  by_edge: Vec<Option<SparseIndex>>,
}

impl IndexIntersectionSet {
  /// O(n^2)
  pub fn new(vertices: usize) -> IndexIntersectionSet {
    let size = vertices * (vertices - 1);
    let by_edge = vec![None; size];
    IndexIntersectionSet {
      vertices,
      by_idx: SparseVec::new(),
      by_edge,
    }
  }

  // O(1)
  pub fn push(&mut self, isect: IndexIntersection) {
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

    // eprintln!("Removing intersection: {:?}", isect);

    for &edge in &[isect.edge0, isect.edge1] {
      match edge.prev {
        Some(prev) => self.by_idx[prev][IndexEdge::from(edge)].next = edge.next,
        None => self[IndexEdge::from(edge)] = edge.next,
      }
      if let Some(next) = edge.next {
        self.by_idx[next][IndexEdge::from(edge)].prev = edge.prev;
      }
    }
  }

  // O(n) where N is the number of removed intersections.
  pub fn remove_all(&mut self, edge: impl Into<IndexEdge>) {
    let key = edge.into();
    while let Some(idx) = self[key] {
      self.remove(idx)
    }
  }

  // O(1)
  pub fn random<R>(&mut self, rng: &mut R) -> Option<IndexIntersection>
  where
    R: Rng + ?Sized,
  {
    let idx = self.by_idx.random(rng)?;
    let isect = self.by_idx[idx];
    Some(IndexIntersection::new(
      IndexEdge::new(isect.edge0.vertex0, isect.edge0.vertex1),
      IndexEdge::new(isect.edge1.vertex0, isect.edge1.vertex1),
    ))
  }

  pub fn iter(&self) -> impl Iterator<Item = IndexIntersection> + '_ {
    self.by_idx.iter().map(|&isect| isect.into())
  }
}

impl IndexMut<IndexEdge> for IndexIntersectionSet {
  fn index_mut(&mut self, index: IndexEdge) -> &mut Option<SparseIndex> {
    self
      .by_edge
      .index_mut(index.max.usize() + self.vertices * index.min.usize())
  }
}

impl Index<IndexEdge> for IndexIntersectionSet {
  type Output = Option<SparseIndex>;
  fn index(&self, index: IndexEdge) -> &Option<SparseIndex> {
    self
      .by_edge
      .index(index.max.usize() + self.vertices * index.min.usize())
  }
}

impl PartialEq<IndexEdge> for IsectEdge {
  fn eq(&self, other: &IndexEdge) -> bool {
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
#[derive(Clone, Copy, Debug)]
// FIXME: Should not be pub.
struct Isect {
  edge0: IsectEdge,
  edge1: IsectEdge,
}

impl Default for Isect {
  fn default() -> Isect {
    Isect {
      edge0: IsectEdge {
        vertex0: PointId::INVALID,
        vertex1: PointId::INVALID,
        next: None,
        prev: None,
      },
      edge1: IsectEdge {
        vertex0: PointId::INVALID,
        vertex1: PointId::INVALID,
        next: None,
        prev: None,
      },
    }
  }
}

impl Isect {
  fn new(intersection: IndexIntersection) -> Isect {
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

impl IndexMut<IndexEdge> for Isect {
  fn index_mut(&mut self, index: IndexEdge) -> &mut IsectEdge {
    if self.edge0 == index {
      &mut self.edge0
    } else {
      assert_eq!(self.edge1, index);
      &mut self.edge1
    }
  }
}

impl Index<IndexEdge> for Isect {
  type Output = IsectEdge;
  fn index(&self, index: IndexEdge) -> &IsectEdge {
    if self.edge0 == index {
      &self.edge0
    } else {
      assert_eq!(self.edge1, index);
      &self.edge1
    }
  }
}

#[derive(Clone, Copy, Debug)]
// FIXME: Should not be pub
struct IsectEdge {
  vertex0: PointId,
  vertex1: PointId,
  next: Option<SparseIndex>,
  prev: Option<SparseIndex>,
}

impl IsectEdge {
  // FIXME: Sort vertex0 and vertex1
  fn new(edge: IndexEdge) -> IsectEdge {
    IsectEdge {
      vertex0: edge.min,
      vertex1: edge.max,
      next: None,
      prev: None,
    }
  }
}

impl From<IsectEdge> for IndexEdge {
  fn from(e: IsectEdge) -> IndexEdge {
    IndexEdge::new(e.vertex0, e.vertex1)
  }
}
