use nalgebra::linalg::LU;
use nalgebra::*;
use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::ops::{Index, IndexMut, Neg};

use crate::data::Point;
use crate::Error;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HalfEdgeId(pub usize);

impl std::fmt::Debug for HalfEdgeId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    write!(f, "HalfEdgeId({})", self.0)
  }
}

impl HalfEdgeId {
  const INVALID: HalfEdgeId = HalfEdgeId(usize::MAX);

  fn twin(self) -> HalfEdgeId {
    HalfEdgeId(self.0 ^ 1)
  }

  // fn face(self, pg: &mut PlanarGraph) -> &mut FaceId {
  //   &mut self.mutate(pg).face
  // }

  // fn vertex(self, pg: &mut PlanarGraph) -> &mut VertexId {
  //   &mut self.mutate(pg).vertex
  // }

  // fn next(self, pg: &mut PlanarGraph) -> &mut HalfEdgeId {
  //   &mut self.mutate(pg).next
  // }

  // fn prev(self, pg: &mut PlanarGraph) -> &mut HalfEdgeId {
  //   &mut self.mutate(pg).prev
  // }

  fn mutate(self, pg: &mut PlanarGraph) -> &mut HalfEdgeEntry {
    &mut pg.half_edges[self.0]
  }

  fn read(self, pg: &PlanarGraph) -> &HalfEdgeEntry {
    &pg.half_edges[self.0]
  }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VertexId(pub usize);

impl std::fmt::Debug for VertexId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    write!(f, "VertexId({})", self.0)
  }
}

impl VertexId {
  const INVALID: VertexId = VertexId(usize::MAX);

  fn half_edge(self, pg: &mut PlanarGraph) -> &mut HalfEdgeId {
    &mut pg.vertex_edges[self.0]
  }
}

///////////////////////////////////////////////////////////////////////////////
// PlanarGraph

#[derive(Copy, Clone, Debug)]
struct HalfEdgeEntry {
  next: HalfEdgeId,
  prev: HalfEdgeId,
  vertex: VertexId,
  face: FaceId,
}

#[derive(Clone, Debug)]
pub struct PlanarGraph {
  half_edges: Vec<HalfEdgeEntry>,  // HalfEdge indexed
  vertex_edges: Vec<HalfEdgeId>,   // Vertex indexed
  face_edges: Vec<HalfEdgeId>,     // Face indexed
  boundary_edges: Vec<HalfEdgeId>, // Boundary faces
}

impl PlanarGraph {
  fn empty() -> PlanarGraph {
    PlanarGraph {
      half_edges: Vec::new(),
      vertex_edges: Vec::new(),
      face_edges: Vec::new(),
      boundary_edges: Vec::new(),
    }
  }

  /// ```rust
  /// # use rgeometry::data::{PlanarGraph, planar_graph::*};
  /// # use rgeometry::Error;
  /// let pg = PlanarGraph::new_from_faces(vec![vec![0, 1, 2]])?;
  /// let mut faces = pg.faces();
  ///
  /// assert_eq!(faces.next(), Some(FaceId(0)));
  /// assert_eq!(faces.next(), None);
  /// # Ok::<(), Error>(())
  /// ```
  pub fn new_from_faces(faces: Vec<Vec<usize>>) -> Result<PlanarGraph, Error> {
    fn zip_half_edges(pg: &mut PlanarGraph, face: FaceId, edges: Vec<HalfEdgeId>) {
      for n in 0..edges.len() {
        let prev = edges[n];
        let this = edges[(n + 1) % edges.len()].mutate(pg);
        let next = edges[(n + 2) % edges.len()];
        this.next = next;
        this.prev = prev;
        this.face = face;
      }
    }

    // // hopefully this gets optimized away
    // let faces: Vec<Vec<VertexId>> = faces
    //   .into_iter()
    //   .map(|vec| vec.into_iter().map(Into::into).collect())
    //   .collect();

    let max_vertex_id = faces
      .iter()
      .filter_map(|vec| vec.iter().max())
      .max()
      .ok_or(Error::InsufficientVertices)?;

    let mut pg = PlanarGraph::empty();

    pg.vertex_edges
      .resize(max_vertex_id + 1, HalfEdgeId::INVALID);

    let mut edge_map: HashMap<(VertexId, VertexId), HalfEdgeId> = HashMap::new();
    for vertices in faces {
      assert!(vertices.len() >= 3);

      let face = pg.new_face();
      let mut edges: Vec<HalfEdgeId> = Vec::with_capacity(vertices.len());
      for (&tail, &tip) in pairs(&vertices) {
        let tail = VertexId(tail);
        let tip = VertexId(tip);
        if edge_map.get(&(tail, tip)).is_some() {
          return Err(Error::SelfIntersections);
        }
        match edge_map.get(&(tip, tail)) {
          Some(twin) => edges.push(twin.twin()),
          None => {
            let he = pg.new_half_edge();
            edge_map.insert((tail, tip), he);
            *tip.half_edge(&mut pg) = he;
            he.mutate(&mut pg).vertex = tail;
            he.twin().mutate(&mut pg).vertex = tip;
            edges.push(he);
          }
        }
      }
      eprintln!("Setting face edge: {:?} {:?}", face, edges[0]);
      *face.half_edge_mut(&mut pg) = edges[0];
      zip_half_edges(&mut pg, face, edges);
    }

    for vertex in (0..pg.vertex_edges.len()).map(VertexId) {
      let he = pg.vertex(vertex).half_edge().twin();
      if !he.face().is_valid() {
        let boundary = he.construct_boundary();
        let he = he.half_edge_id();

        let face = pg.new_boundary();
        *face.half_edge_mut(&mut pg) = he;
        zip_half_edges(&mut pg, face, boundary);
      }
    }
    // for he in (0..pg.half_edges.len()).map(HalfEdgeId) {
    //   if !he.read(&pg).face.is_valid() {
    //     let face = pg.new_boundary();
    //     *face.half_edge_mut(&mut pg) = he;
    //     let boundary = pg.half_edge(he).construct_boundary();
    //     zip_half_edges(&mut pg, face, boundary);
    //   }
    // }

    Ok(pg)
  }

  pub fn faces(&self) -> impl Iterator<Item = FaceId> + '_ {
    self.face_edges.iter().enumerate().filter_map(|(nth, &he)| {
      if he == HalfEdgeId::INVALID {
        None
      } else {
        Some(FaceId::face(nth))
      }
    })
  }

  pub fn tutte_embedding(&self) -> Vec<Point<OrderedFloat<f64>, 2>> {
    fn regular_polygon(n: usize) -> impl Iterator<Item = (f64, f64)> {
      let frac = std::f64::consts::TAU / (n as f64);
      (0..n).map(move |nth| {
        let ang = nth as f64 * frac + std::f64::consts::FRAC_PI_2;
        (ang.cos(), ang.sin())
      })
    }
    let n = self.vertex_edges.len();
    let mut m = DMatrix::<f64>::zeros(n, n);
    let mut vx = DVector::<f64>::zeros(n);
    let mut vy = DVector::<f64>::zeros(n);

    let boundary: Vec<Vertex<'_>> = self.face(FaceId::boundary(0)).boundary().collect();

    // Fix the outer boundary.
    for ((x, y), vertex) in regular_polygon(boundary.len()).zip(boundary.into_iter()) {
      let vid = vertex.vertex_id().0;
      m[(vid, vid)] = 1.0;
      vx[vid] = x;
      vy[vid] = y;
    }

    // Define the inner vertices relative to its neighbours.
    for vertex_id in 0..self.vertex_edges.len() {
      let vertex = self.vertex(VertexId(vertex_id));
      if !vertex.is_boundary() {
        let mut neighbours = 0;
        for neighbour in vertex.neighbours() {
          m[(vertex_id, neighbour.vertex_id().0)] = 1.0;
          neighbours += 1;
        }
        m[(vertex_id, vertex_id)] = -(neighbours as f64);
      }
    }
    let lu = LU::new(m);
    assert!(lu.solve_mut(&mut vx));
    assert!(lu.solve_mut(&mut vy));

    vx.into_iter()
      .zip(vy.into_iter())
      .map(|(&x, &y)| Point::new([OrderedFloat(x), OrderedFloat(y)]))
      .collect()
  }

  fn vertex_edge(&self, key: VertexId) -> HalfEdgeId {
    self.vertex_edges[key.0]
  }

  fn new_face(&mut self) -> FaceId {
    let id = FaceId::face(self.face_edges.len());
    self.face_edges.push(HalfEdgeId::INVALID);
    id
  }

  fn new_boundary(&mut self) -> FaceId {
    let id = FaceId::boundary(self.boundary_edges.len());
    self.boundary_edges.push(HalfEdgeId::INVALID);
    id
  }

  fn new_half_edge(&mut self) -> HalfEdgeId {
    assert!(self.half_edges.len() % 2 == 0);
    let idx = HalfEdgeId(self.half_edges.len());

    self.half_edges.push(HalfEdgeEntry {
      next: HalfEdgeId::INVALID,
      prev: HalfEdgeId::INVALID,
      vertex: VertexId::INVALID,
      face: FaceId::INVALID,
    });
    self.half_edges.push(HalfEdgeEntry {
      next: HalfEdgeId::INVALID,
      prev: HalfEdgeId::INVALID,
      vertex: VertexId::INVALID,
      face: FaceId::INVALID,
    });

    idx
  }

  pub fn half_edge(&self, he: HalfEdgeId) -> HalfEdge<'_> {
    HalfEdge {
      pg: self,
      index: he,
    }
  }

  pub fn face(&self, face_id: FaceId) -> Face<'_> {
    Face {
      pg: self,
      index: face_id,
    }
  }

  pub fn vertex(&self, vertex_id: VertexId) -> Vertex<'_> {
    Vertex {
      pg: self,
      index: vertex_id,
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Vertex

#[derive(Clone, Copy)]
pub struct Vertex<'a> {
  pg: &'a PlanarGraph,
  index: VertexId,
}

impl<'a> PartialEq<VertexId> for Vertex<'a> {
  fn eq(&self, other: &VertexId) -> bool {
    self.index == *other
  }
}

impl std::fmt::Debug for Vertex<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    write!(f, "{:?}", self.index)
  }
}

impl<'a> Vertex<'a> {
  pub fn vertex_id(self) -> VertexId {
    self.index
  }

  pub fn half_edge(self) -> HalfEdge<'a> {
    HalfEdge {
      pg: self.pg,
      index: self.pg.vertex_edge(self.index),
    }
  }

  pub fn is_boundary(self) -> bool {
    self.half_edge().twin().face().is_boundary()
  }

  pub fn outgoing_half_edges(self) -> impl Iterator<Item = HalfEdge<'a>> {
    iter_finite(self.half_edge(), |he| he.twin().next())
  }

  pub fn incoming_half_edges(self) -> impl Iterator<Item = HalfEdge<'a>> {
    self.outgoing_half_edges().map(|he| he.twin())
  }

  pub fn neighbours(self) -> impl Iterator<Item = Vertex<'a>> {
    self.incoming_half_edges().map(|he| he.vertex())
  }
}

///////////////////////////////////////////////////////////////////////////////
// HalfEdge

#[derive(Clone, Copy)]
pub struct HalfEdge<'a> {
  pg: &'a PlanarGraph,
  index: HalfEdgeId,
}

impl<'a> PartialEq<HalfEdgeId> for HalfEdge<'a> {
  fn eq(&self, other: &HalfEdgeId) -> bool {
    self.index == *other
  }
}

impl<'a> PartialEq for HalfEdge<'a> {
  fn eq(&self, other: &HalfEdge<'a>) -> bool {
    self.index == other.index
  }
}

impl std::fmt::Debug for HalfEdge<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    write!(f, "{:?}", self.index)
  }
}

impl<'a> HalfEdge<'a> {
  pub fn half_edge_id(self) -> HalfEdgeId {
    self.index
  }

  pub fn next(self) -> HalfEdge<'a> {
    HalfEdge {
      pg: self.pg,
      index: self.read().next,
    }
  }

  pub fn prev(self) -> HalfEdge<'a> {
    HalfEdge {
      pg: self.pg,
      index: self.read().prev,
    }
  }

  pub fn next_outgoing(self) -> HalfEdge<'a> {
    self.twin().next()
  }

  pub fn next_incoming(self) -> HalfEdge<'a> {
    self.twin().prev()
  }

  pub fn vertex(self) -> Vertex<'a> {
    Vertex {
      pg: self.pg,
      index: self.read().vertex,
    }
  }

  pub fn twin(self) -> HalfEdge<'a> {
    HalfEdge {
      pg: self.pg,
      index: self.index.twin(),
    }
  }

  pub fn tail_vertex(self) -> Vertex<'a> {
    self.vertex()
  }

  pub fn tip_vertex(self) -> Vertex<'a> {
    self.twin().vertex()
  }

  pub fn face(self) -> Face<'a> {
    Face {
      pg: self.pg,
      index: self.read().face,
    }
  }

  pub fn is_interior(self) -> bool {
    self.face().is_interior()
  }

  fn read(self) -> &'a HalfEdgeEntry {
    self.index.read(self.pg)
  }

  fn construct_boundary(self) -> Vec<HalfEdgeId> {
    let mut out = Vec::new();
    let mut edge = self;
    loop {
      if edge.face().is_valid() {
        edge = edge.prev().twin();
      } else {
        out.push(edge.index);
        edge = edge.next_incoming().twin();
      }
      if edge.index == self.index {
        return out;
      }
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Face

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FaceId(pub isize);

impl std::fmt::Debug for FaceId {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    if self.0 < 0 {
      write!(f, "BoundaryId({})", (self.0 + 1).neg())
    } else {
      write!(f, "FaceId({})", self.0)
    }
  }
}

impl FaceId {
  const INVALID: FaceId = FaceId(isize::MAX);

  pub fn face(index: usize) -> FaceId {
    FaceId(index as isize)
  }

  pub fn boundary(index: usize) -> FaceId {
    FaceId((index as isize).neg() - 1)
  }

  fn half_edge_mut(self, pg: &mut PlanarGraph) -> &mut HalfEdgeId {
    if self.0 < 0 {
      &mut pg.boundary_edges[(self.0 + 1).neg() as usize]
    } else {
      &mut pg.face_edges[self.0 as usize]
    }
  }

  fn half_edge(self, pg: &PlanarGraph) -> HalfEdgeId {
    if self.0 < 0 {
      pg.boundary_edges[(self.0 + 1).neg() as usize]
    } else {
      pg.face_edges[self.0 as usize]
    }
  }

  pub fn is_interior(self) -> bool {
    !self.is_boundary()
  }

  pub fn is_boundary(self) -> bool {
    self < FaceId(0)
  }

  pub fn is_valid(self) -> bool {
    self != FaceId::INVALID
  }
}

#[derive(Copy, Clone)]
pub struct Face<'a> {
  pg: &'a PlanarGraph,
  index: FaceId,
}

impl<'a> PartialEq<FaceId> for Face<'a> {
  fn eq(&self, other: &FaceId) -> bool {
    self.index == *other
  }
}

impl<'a> PartialEq for Face<'a> {
  fn eq(&self, other: &Face<'a>) -> bool {
    self.index == other.index
  }
}

impl std::fmt::Debug for Face<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    write!(f, "{:?}", self.index)
  }
}

impl<'a> Face<'a> {
  /// ```rust
  /// # use rgeometry::data::{PlanarGraph, planar_graph::*};
  /// # use rgeometry::Error;
  /// let pg = PlanarGraph::new_from_faces(vec![vec![0, 1, 2]])?;
  /// let he = pg.face(FaceId::face(0)).half_edge();
  ///
  /// assert_eq!(he, HalfEdgeId(0));
  /// assert_eq!(he.vertex(), VertexId(0));
  /// assert_eq!(he.next(), HalfEdgeId(2));
  /// assert_eq!(he.next().vertex(), VertexId(1));
  /// assert_eq!(he.next().next(), HalfEdgeId(4));
  /// # Ok::<(), Error>(())
  /// ```
  pub fn half_edge(self) -> HalfEdge<'a> {
    HalfEdge {
      pg: self.pg,
      index: self.index.half_edge(self.pg),
    }
  }

  pub fn half_edges(self) -> impl Iterator<Item = HalfEdge<'a>> {
    if self.is_boundary() {
      // Reverse the orientation such that edges are in CCW order.
      iter_finite(self.half_edge(), |he| he.prev())
    } else {
      iter_finite(self.half_edge(), |he| he.next())
    }
  }

  /// ```rust
  /// # use rgeometry::data::{PlanarGraph, planar_graph::*};
  /// # use rgeometry::Error;
  /// let pg = PlanarGraph::new_from_faces(vec![vec![0, 1, 2]])?;
  /// let mut boundary = pg.face(FaceId::boundary(0)).boundary().map(|v| v.vertex_id());
  ///
  /// assert_eq!(boundary.next(), Some(VertexId(0)));
  /// assert_eq!(boundary.next(), Some(VertexId(1)));
  /// assert_eq!(boundary.next(), Some(VertexId(2)));
  /// assert_eq!(boundary.next(), None);
  /// # Ok::<(), Error>(())
  /// ```
  ///
  /// ```rust
  /// # use rgeometry::data::{PlanarGraph, planar_graph::*};
  /// # use rgeometry::Error;
  /// let pg = PlanarGraph::new_from_faces(vec![vec![0, 1, 2]])?;
  /// let mut boundary = pg.face(FaceId::face(0)).boundary().map(|v| v.vertex_id());
  ///
  /// assert_eq!(boundary.next(), Some(VertexId(0)));
  /// assert_eq!(boundary.next(), Some(VertexId(1)));
  /// assert_eq!(boundary.next(), Some(VertexId(2)));
  /// assert_eq!(boundary.next(), None);
  /// # Ok::<(), Error>(())
  /// ```
  pub fn boundary(self) -> impl Iterator<Item = Vertex<'a>> {
    self.half_edges().map(|he| he.vertex())
  }

  pub fn is_interior(self) -> bool {
    self.index.is_interior()
  }

  pub fn is_boundary(self) -> bool {
    self.index.is_boundary()
  }

  pub fn is_valid(self) -> bool {
    self.index.is_valid()
  }
}

///////////////////////////////////////////////////////////////////////////////
// misc

fn pairs<E>(slice: &[E]) -> impl Iterator<Item = (&E, &E)> {
  slice
    .iter()
    .zip(slice[1..].iter().chain(std::iter::once(&slice[0])))
}

fn iter_finite<T>(begin: T, next: fn(T) -> T) -> impl Iterator<Item = T>
where
  T: PartialEq + Copy,
{
  let mut done = false;
  let mut at = begin;
  std::iter::from_fn(move || {
    if done {
      return None;
    }
    let this = at;
    at = next(at);
    if at == begin {
      done = true;
    }
    Some(this)
  })
}
