use super::Vertex;
use crate::data::DirectedIndexEdge;
use crate::data::IndexEdge;
use crate::data::Point;
use crate::Orientation;
use crate::PolygonScalar;

use num_traits::*;

#[derive(Clone, Debug)]
pub struct VertexList {
  links: Vec<Link>,
}

#[derive(Copy, Clone, Debug)]
struct Link {
  prev: Vertex,
  next: Vertex,
}

impl VertexList {
  pub fn new(vertices: usize) -> VertexList {
    let mut ret = VertexList {
      links: vec![Link { prev: 0, next: 0 }; vertices],
    };
    for i in 0..vertices {
      ret.connect(i, (i + 1) % vertices);
    }
    ret
  }

  /// Iterator for unsorted edges.
  pub fn unordered_edges(&self) -> impl Iterator<Item = IndexEdge> + '_ {
    self
      .links
      .iter()
      .enumerate()
      .map(|(nth, link)| IndexEdge::new(nth, link.next))
  }

  pub fn unordered_directed_edges(&self) -> impl Iterator<Item = DirectedIndexEdge> + '_ {
    self
      .links
      .iter()
      .enumerate()
      .map(|(nth, link)| DirectedIndexEdge {
        src: nth,
        dst: link.next,
      })
  }

  pub fn vertices(&self) -> impl Iterator<Item = Vertex> + '_ {
    let mut at = 0;
    std::iter::from_fn(move || {
      let this = at;
      at = self.links[at].next;
      Some(this)
    })
    .take(self.links.len())
  }

  pub fn parallel<T: PolygonScalar>(
    &self,
    pts: &[Point<T, 2>],
    a: DirectedIndexEdge,
    b: DirectedIndexEdge,
  ) -> bool {
    Orientation::is_colinear(&pts[a.src], &pts[a.dst], &pts[b.src])
      && Orientation::is_colinear(&pts[a.src], &pts[a.dst], &pts[b.dst])
  }

  pub fn linear_extremes<T>(&self, pts: &[Point<T, 2>], edge: DirectedIndexEdge) -> (Vertex, Vertex)
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

  pub fn range(
    &self,
    mut min: Vertex,
    max: Vertex,
  ) -> impl Iterator<Item = DirectedIndexEdge> + '_ {
    std::iter::from_fn(move || {
      if min != max {
        let edge = DirectedIndexEdge {
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

  pub fn direct(&self, edge: IndexEdge) -> DirectedIndexEdge {
    if self.links[edge.min].next == edge.max {
      DirectedIndexEdge {
        src: edge.min,
        dst: edge.max,
      }
    } else {
      assert_eq!(self.links[edge.max].next, edge.min);
      assert_eq!(self.links[edge.min].prev, edge.max);
      DirectedIndexEdge {
        src: edge.max,
        dst: edge.min,
      }
    }
  }

  pub fn vertex_edges(&self, pt: Vertex) -> (DirectedIndexEdge, DirectedIndexEdge) {
    let prev_idx = self.links[pt].prev;
    let next_idx = self.links[pt].next;
    (
      DirectedIndexEdge {
        src: prev_idx,
        dst: pt,
      },
      DirectedIndexEdge {
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
  pub fn hoist(&mut self, pt: Vertex, edge: DirectedIndexEdge) {
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
  pub fn uncross(&mut self, a: DirectedIndexEdge, b: DirectedIndexEdge) {
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

  pub fn sort_vertices<T>(self, pts: &mut Vec<T>) {
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
}
