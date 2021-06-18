mod intersection_set;
mod vertex_list;

pub use intersection_set::*;
pub use vertex_list::*;

pub type Vertex = usize;
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
  pub min: Vertex,
  pub max: Vertex,
  _private: (),
}

impl Edge {
  pub fn new(a: Vertex, b: Vertex) -> Edge {
    Edge {
      min: std::cmp::min(a, b),
      max: std::cmp::max(a, b),
      _private: (),
    }
  }
}

impl std::fmt::Debug for Edge {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Edge")
      .field("min", &self.min)
      .field("max", &self.max)
      .finish()
  }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DirectedEdge {
  pub src: Vertex,
  pub dst: Vertex,
}

impl From<DirectedEdge> for Edge {
  fn from(directed: DirectedEdge) -> Edge {
    Edge::new(directed.src, directed.dst)
  }
}
