// https://www.personal.psu.edu/cxc11/AERSP560/DELAUNEY/13_Two_algorithms_Delauney.pdf
use crate::Error;
use crate::{data::*, Orientation, PolygonScalar};
use std::convert::TryFrom;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct TriIdx(pub usize);
impl std::fmt::Debug for TriIdx {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "t{}", self.0)
  }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct VertIdx(pub usize);
impl std::fmt::Debug for VertIdx {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "v{}", self.0)
  }
}

impl VertIdx {
  pub fn is_super(&self) -> bool {
    self.0 < 3
  }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct SubIdx(pub usize);
impl std::fmt::Debug for SubIdx {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "s{}", self.0)
  }
}

impl SubIdx {
  pub fn ccw(self) -> Self {
    Self((self.0 + 1) % 3)
  }
  pub fn cw(self) -> Self {
    Self((self.0 + 2) % 3)
  }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Edge {
  tri: TriIdx,
  sub: SubIdx,
}
impl Edge {
  fn new(tri: TriIdx, sub: SubIdx) -> Self {
    Self { tri, sub }
  }
}

#[derive(Debug)]
pub struct Cut {
  /// starting vertex of the cut
  pub from: VertIdx,
  /// finishing vertex of the cut
  pub to: VertIdx,
  /// list of edges which will be removed from the cut
  pub cuts: Vec<(VertIdx, VertIdx)>,
  /// list of triangles affacted by the cut
  pub cut_triangles: Vec<TriIdx>,
  pub contour_cw: Vec<Edge>,
  pub contour_ccw: Vec<Edge>,
}

/// A location of a point, in `TriangularNetwork`
#[derive(Debug, PartialEq, Eq)]
pub enum TriangularNetworkLocation {
  /// The point is inside the triangle
  InTriangle(TriIdx),
  /// The points is at the vertex of the triangle
  OnVertex(TriIdx, SubIdx),
  /// The points lies on an edge of the triangle
  OnEdge(Edge),
  /// The point lies outside the triangle
  Outside(Edge),
}

#[derive(Debug)]
struct CutEdge {
  inner: Edge,
  outer: Option<Edge>,
  vert: VertIdx,
}

#[derive(Debug)]
enum CutIter {
  FromVertex(TriIdx, SubIdx),
  CoLinear {
    tri: TriIdx,
    src: SubIdx,
    dst: SubIdx,
  },
  ToEdge(Edge),
}

#[derive(Debug, Clone)]
pub struct TriangularNetwork<T> {
  pub vertices: Vec<Point<T>>,
  pub triangles: Vec<Triangle>,
}

type Result<T> = std::result::Result<T, Error>;

impl<T: PolygonScalar> TriangularNetwork<T> {
  /// Create new triangluar network.
  pub fn new(p0: Point<T>, p1: Point<T>, p2: Point<T>) -> Result<Self> {
    let (p1, p2) = match Point::orient_along_direction(&p0, Direction::Through(&p1), &p2) {
      Orientation::CounterClockWise => (p1, p2),
      Orientation::CoLinear => return Err(Error::CoLinearViolation),
      _ => (p2, p1),
    };

    Ok(Self {
      vertices: vec![p0, p1, p2],
      triangles: vec![Triangle {
        vertices: [VertIdx(0), VertIdx(1), VertIdx(2)],
        neighbors: [None, None, None],
      }],
    })
  }

  pub fn tri(&self, idx: TriIdx) -> &Triangle {
    &self.triangles[idx.0]
  }

  fn tri_mut(&mut self, idx: TriIdx) -> &mut Triangle {
    &mut self.triangles[idx.0]
  }

  fn add_tri(&mut self) -> TriIdx {
    let v = VertIdx(0);
    let idx = self.triangles.len();
    self.triangles.push(Triangle {
      vertices: [v, v, v],
      neighbors: [None, None, None],
    });
    TriIdx(idx)
  }

  pub fn find_vert(&self, p: &Point<T>) -> Option<VertIdx> {
    self.vertices.iter().position(|v| v == p).map(VertIdx)
  }

  pub fn vert(&self, idx: VertIdx) -> &Point<T> {
    &self.vertices[idx.0]
  }

  fn add_vert(&mut self, p: Point<T>) -> VertIdx {
    let idx = self.vertices.len();
    self.vertices.push(p);
    VertIdx(idx)
  }

  pub fn tri_vert(&self, tri_idx: TriIdx, idx: SubIdx) -> &Point<T> {
    self.vert(self.tri(tri_idx).vertices[idx.0])
  }

  pub fn edge_duel(&self, edge: &Edge) -> Result<Option<Edge>> {
    let t = self.tri(edge.tri);
    let idx_neighbor = match t.neighbor(edge.sub) {
      Some(idx) => idx,
      None => return Ok(None),
    };
    let t_neighbor = self.tri(idx_neighbor);
    let sub_neighbor = match t_neighbor.neighbor_idx(edge.tri) {
      Some(idx) => idx,
      None => {
        return Err(Error::InvariantViolation);
      }
    };
    Ok(Some(Edge::new(idx_neighbor, sub_neighbor)))
  }

  pub fn edge_to(&self, edge: &Edge) -> VertIdx {
    let t = self.tri(edge.tri);
    t.vert(edge.sub)
  }

  pub fn edge_from(&self, edge: &Edge) -> VertIdx {
    let t = self.tri(edge.tri);
    t.vert(edge.sub.cw())
  }

  fn find_vert_dest(&self, v_from: VertIdx, v_to: VertIdx) -> Result<Option<CutIter>> {
    use Orientation::*;

    let p_from = self.vert(v_from);
    let l = self.locate_recursive(p_from)?;

    let (tri0, sub0) = match l {
      TriangularNetworkLocation::OnVertex(tri, sub) => (tri, sub),
      _ => return Ok(None),
    };

    let p_end = self.vert(v_to);
    let mut candidates = Vec::new();

    // TODO: to iterator
    // ccw triangles
    let mut curtri = tri0;
    let mut cursub = sub0;
    loop {
      let t = self.tri(curtri);
      assert_eq!(t.vert(cursub), v_from);
      if t.vert(cursub.cw()) == v_to {
        return Ok(None);
      }
      if candidates.contains(&(curtri, cursub)) {
        break;
      }
      candidates.push((curtri, cursub));

      match self.edge_duel(&Edge::new(curtri, cursub))? {
        Some(Edge { tri, sub }) => {
          curtri = tri;
          cursub = sub.cw();
        }
        None => break,
      }
    }

    // TODO: to iterator
    // cw triangles
    let mut curtri = tri0;
    let mut cursub = sub0;
    loop {
      let t = self.tri(curtri);
      assert_eq!(t.vert(cursub), v_from);
      if t.vert(cursub.ccw()) == v_to {
        return Ok(None);
      }
      if candidates.contains(&(curtri, cursub)) {
        break;
      }
      candidates.push((curtri, cursub));

      match self.edge_duel(&Edge::new(curtri, cursub.ccw()))? {
        Some(Edge { tri, sub }) => {
          if tri == tri0 {
            break;
          }
          curtri = tri;
          cursub = sub;
        }
        None => break,
      }
    }

    for (t_idx, idx) in candidates {
      let p0 = self.tri_vert(t_idx, idx);
      let p1 = self.tri_vert(t_idx, idx.ccw());
      let p2 = self.tri_vert(t_idx, idx.cw());

      let d0 = Point::orient_along_direction(p0, Direction::Through(p1), p_end);
      let d1 = Point::orient_along_direction(p1, Direction::Through(p2), p_end);
      let d2 = Point::orient_along_direction(p2, Direction::Through(p0), p_end);

      let res = match (d0, d1, d2) {
        (CounterClockWise, ClockWise, CounterClockWise) => CutIter::FromVertex(t_idx, idx),
        (CoLinear, ClockWise, CounterClockWise) => CutIter::CoLinear {
          tri: t_idx,
          src: idx,
          dst: idx.ccw(),
        },
        (CounterClockWise, ClockWise, CoLinear) => CutIter::CoLinear {
          tri: t_idx,
          src: idx,
          dst: idx.cw(),
        },
        _ => continue,
      };
      return Ok(Some(res));
    }
    Ok(None)
  }

  pub fn cut(&self, v_from: VertIdx, v_to: VertIdx) -> Result<Cut> {
    use Orientation::*;

    let mut cuts = vec![];
    let mut cut_triangles = vec![];
    let mut contour_ccw = vec![];
    let mut contour_cw = vec![];

    let p_start = self.vert(v_from);
    let p_end = self.vert(v_to);

    let mut cur = self.find_vert_dest(v_from, v_to)?;
    while let Some(iter) = cur.take() {
      match iter {
        // ray from vertex idx
        CutIter::FromVertex(t_idx, idx) => {
          cut_triangles.push(t_idx);

          let t = self.tri(t_idx);

          let v0 = t.vert(idx);
          assert!(v0 != v_to);

          contour_ccw.push(Edge::new(t_idx, idx));
          contour_cw.push(Edge::new(t_idx, idx.ccw()));
          cuts.push((t.vert(idx.cw()), t.vert(idx.ccw())));

          let next = self
            .edge_duel(&Edge::new(t_idx, idx.cw()))?
            .ok_or(Error::InvariantViolation)?;
          cur = Some(CutIter::ToEdge(next));
        }

        CutIter::ToEdge(Edge {
          tri: t_idx,
          sub: idx,
        }) => {
          cut_triangles.push(t_idx);

          let t = self.tri(t_idx);

          let p0 = self.tri_vert(t_idx, idx);
          let p1 = self.tri_vert(t_idx, idx.ccw());
          let p2 = self.tri_vert(t_idx, idx.cw());

          // should not colinear
          let d0 = Point::orient_along_direction(p_start, Direction::Through(p_end), p0);
          let d1 = Point::orient_along_direction(p_start, Direction::Through(p_end), p1);
          // should not colineaer
          let d2 = Point::orient_along_direction(p_start, Direction::Through(p_end), p2);

          let idx_n = if d1 == CoLinear {
            contour_ccw.push(Edge::new(t_idx, idx.cw()));
            contour_cw.push(Edge::new(t_idx, idx.ccw()));

            cur = self.find_vert_dest(t.vert(idx.ccw()), v_to)?;
            continue;
          } else if d1.reverse() == d0 {
            contour_ccw.push(Edge::new(t_idx, idx.cw()));
            idx.ccw()
          } else if d1.reverse() == d2 {
            contour_cw.push(Edge::new(t_idx, idx.ccw()));
            idx.cw()
          } else {
            return Err(Error::InvariantViolation);
          };

          cuts.push((t.vert(idx_n), t.vert(idx_n.cw())));

          let next = self
            .edge_duel(&Edge::new(t_idx, idx.cw()))?
            .ok_or(Error::InvariantViolation)?;
          cur = Some(CutIter::ToEdge(next));
        }

        CutIter::CoLinear { tri, src, dst } => {
          let (edge_cw, edge_ccw) = if src.ccw() == dst {
            let edge_cw = Edge::new(tri, dst);
            let edge_ccw = self.edge_duel(&edge_cw)?.ok_or(Error::InvariantViolation)?;
            (edge_cw, edge_ccw)
          } else {
            let edge_ccw = Edge::new(tri, src);
            let edge_cw = self
              .edge_duel(&edge_ccw)?
              .ok_or(Error::InvariantViolation)?;
            (edge_cw, edge_ccw)
          };
          contour_cw.push(edge_cw);
          contour_ccw.push(edge_ccw);

          cur = self.find_vert_dest(self.tri(tri).vert(dst), v_to)?;
        }
      }
    }

    Ok(Cut {
      from: v_from,
      to: v_to,
      cut_triangles,
      cuts,
      contour_cw,
      contour_ccw,
    })
  }

  fn cut_apply_subdivide(
    &mut self,
    idx_p: Option<Edge>,
    slice: &[CutEdge],
    indices: &[TriIdx],
    indices_remain: &mut Vec<TriIdx>,
    dirty: &mut Vec<Edge>,
    out: &mut Vec<(VertIdx, VertIdx)>,
  ) -> Result<Option<TriIdx>> {
    if slice.len() <= 1 {
      return Err(Error::InvariantViolation);
    }

    if slice.len() == 2 {
      if let CutEdge {
        inner: _inner,
        outer: Some(outer),
        vert: _,
      } = slice[1]
      {
        if indices.contains(&outer.tri) {
          dirty.push(idx_p.ok_or(Error::InvariantViolation)?);
          return Ok(None);
        } else {
          *self.tri_mut(outer.tri).neighbor_mut(outer.sub) = idx_p.map(|e| e.tri);
          return Ok(Some(outer.tri));
        }
      } else {
        return Err(Error::InvariantViolation);
      }
    }

    let last = slice.len() - 1;

    let v_start = slice[0].vert;
    let v_end = slice[last].vert;

    let p_start = self.vert(v_start);
    let p_end = self.vert(v_end);

    let mut i_mid = 1;
    for i in 2..last {
      let cur = self.vert(slice[i_mid].vert);
      let next = self.vert(slice[i].vert);

      if T::inside_circle(p_start, cur, p_end, next) {
        i_mid = i;
      }
    }
    let v_mid = slice[i_mid].vert;

    let idx_self = indices_remain.pop().ok_or(Error::InvariantViolation)?;

    let idx_t0 = self.cut_apply_subdivide(
      Some(Edge::new(idx_self, SubIdx(1))),
      &slice[..i_mid + 1],
      indices,
      indices_remain,
      dirty,
      out,
    )?;
    let idx_t1 = self.cut_apply_subdivide(
      Some(Edge::new(idx_self, SubIdx(2))),
      &slice[i_mid..],
      indices,
      indices_remain,
      dirty,
      out,
    )?;

    *self.tri_mut(idx_self) = Triangle {
      vertices: [v_start, v_mid, v_end],
      neighbors: [idx_p.map(|e| e.tri), idx_t0, idx_t1],
    };

    out.push((v_start, v_mid));
    out.push((v_mid, v_end));

    Ok(Some(idx_self))
  }

  fn cut_apply_prepare(&mut self, res: &Cut) -> Result<Vec<CutEdge>> {
    let first = res.contour_cw[0];
    let mut verts = vec![CutEdge {
      inner: first,
      outer: None,
      vert: self.tri(first.tri).vert(first.sub.cw()),
    }];

    for edge in res.contour_cw.iter() {
      verts.push(CutEdge {
        inner: *edge,
        outer: self.edge_duel(edge)?,
        vert: self.tri(edge.tri).vert(edge.sub),
      });
    }
    for edge in res.contour_ccw.iter().rev() {
      verts.push(CutEdge {
        inner: *edge,
        outer: self.edge_duel(edge)?,
        vert: self.tri(edge.tri).vert(edge.sub),
      });
    }
    Ok(verts)
  }

  /// # Safety
  /// If there's no need for intermediate results, use `constrain_edge` instead. The function is
  /// safe only when
  ///  - `Cut` returned from `TriangularNetwork::cut` is applied without any changes, and
  ///  - `TriangularNetwork` should not be modified before applying `Cut`.
  pub unsafe fn cut_apply(&mut self, res: &Cut) -> Result<Vec<(VertIdx, VertIdx)>> {
    self.cut_apply_inner(res)
  }

  fn cut_apply_inner(&mut self, res: &Cut) -> Result<Vec<(VertIdx, VertIdx)>> {
    if res.cut_triangles.is_empty() {
      return Ok(vec![]);
    }

    let mut indices = res.cut_triangles.clone();
    let mut dirty = Vec::new();

    let mut out = Vec::new();
    let verts = self.cut_apply_prepare(res)?;
    let mut slice = verts.as_slice();

    let p_start = self.vert(res.from).clone();
    let p_end = self.vert(res.to).clone();

    let mut out_triangles = Vec::new();
    while slice.len() > 1 {
      let split = slice.iter().skip(1).position(|e| {
        Point::orient_along_direction(&p_start, Direction::Through(&p_end), self.vert(e.vert))
          == Orientation::CoLinear
      });
      let i = match split {
        Some(i) => i + 1,
        None => slice.len() - 1,
      };
      let idx = self
        .cut_apply_subdivide(
          None,
          &slice[..i + 1],
          &res.cut_triangles,
          &mut indices,
          &mut dirty,
          &mut out,
        )?
        .ok_or(Error::InvariantViolation)?;
      out_triangles.push(idx);
      slice = &slice[i..];
    }
    assert!(out_triangles.len() % 2 == 0);

    for i in 0..(out_triangles.len() / 2) {
      let t_ccw = out_triangles[i];
      let t_cw = out_triangles[out_triangles.len() - 1 - i];

      *self.tri_mut(t_ccw).neighbor_mut(SubIdx(0)) = Some(t_cw);
      *self.tri_mut(t_cw).neighbor_mut(SubIdx(0)) = Some(t_ccw);
    }

    for i in 0..dirty.len() {
      for j in (i + 1)..dirty.len() {
        let e0 = &dirty[i];
        let e1 = &dirty[j];

        if self.edge_from(e0) == self.edge_to(e1) && self.edge_to(e0) == self.edge_from(e1) {
          *self.tri_mut(e0.tri).neighbor_mut(e0.sub) = Some(e1.tri);
          *self.tri_mut(e1.tri).neighbor_mut(e1.sub) = Some(e0.tri);
        }
      }
    }

    assert!(indices.is_empty());
    self.check_invariant("post-cut_resolve")?;

    Ok(out)
  }

  /// Constraint an edge between two vertices
  /// [reference]: https://people.eecs.berkeley.edu/~jrs/papers/inccdtj.pdf
  pub fn constrain_edge(&mut self, v0: VertIdx, v1: VertIdx) -> Result<()> {
    let cut = self.cut(v0, v1)?;
    self.cut_apply_inner(&cut)?;
    Ok(())
  }

  #[allow(unused)]
  #[cfg(not(debug_assertions))]
  fn check_invariant_tri_opt(&self, _idx: Option<TriIdx>, msg: &str) -> Result<()> {
    Ok(())
  }

  #[cfg(debug_assertions)]
  fn check_invariant_tri_opt(&self, idx: Option<TriIdx>, msg: &str) -> Result<()> {
    if let Some(idx) = idx {
      self.check_invariant_tri(idx, msg)?;
    }
    Ok(())
  }

  #[allow(unused)]
  #[cfg(not(debug_assertions))]
  fn check_invariant_tri(&self, _idx: TriIdx, _msg: &str) -> Result<()> {
    Ok(())
  }

  #[cfg(debug_assertions)]
  fn check_invariant_tri(&self, idx: TriIdx, _msg: &str) -> Result<()> {
    let t = self.tri(idx);
    for i in 0..3 {
      let i = SubIdx(i);

      if let Some(idx_neighbor) = t.neighbor(i) {
        let n = self.tri(idx_neighbor);
        let violated = if let Some(j) = n.neighbor_idx(idx) {
          t.vert(i) != n.vert(j.cw()) || t.vert(i.cw()) != n.vert(j)
        } else {
          true
        };

        if violated {
          return Err(Error::InvariantViolation);
        }
      }

      let e = Edge::new(idx, i);
      if let Some(d) = self.edge_duel(&e)? {
        if Some(e) != self.edge_duel(&d)? {
          return Err(Error::InvariantViolation);
        }
      }
    }
    Ok(())
  }

  #[allow(unused)]
  #[cfg(not(debug_assertions))]
  fn check_invariant(&self, msg: &str) -> Result<()> {
    Ok(())
  }

  #[cfg(debug_assertions)]
  fn check_invariant(&self, msg: &str) -> Result<()> {
    for idx in 0..self.triangles.len() {
      self.check_invariant_tri(TriIdx(idx), msg)?;
    }
    Ok(())
  }

  fn maybe_swap(&mut self, idx0: TriIdx) -> Result<bool> {
    use Orientation::*;

    let t0_t1_idx = SubIdx(2);
    let idx1 = match self.tri(idx0).neighbor(t0_t1_idx) {
      Some(idx) => idx,
      None => return Ok(false),
    };

    let t0 = self.tri(idx0);
    let t1 = self.tri(idx1);

    let t1_t0_idx = match t1.neighbor_idx(idx0) {
      Some(idx) => idx,
      None => {
        return Err(Error::InvariantViolation);
      }
    };

    let v0 = t0.vert(t0_t1_idx);
    let v1 = t0.vert(t0_t1_idx.ccw());
    let v2 = t0.vert(t0_t1_idx.cw());
    let v3 = t1.vert(t1_t0_idx.ccw());

    let p0 = self.vert(v0);
    let p1 = self.vert(v1);
    let p2 = self.vert(v2);
    let p3 = self.vert(v3);

    // check if four points are convex in order
    let d0 = Point::orient_along_direction(p0, Direction::Through(p2), p1);
    let d1 = Point::orient_along_direction(p0, Direction::Through(p2), p3);
    let d2 = Point::orient_along_direction(p1, Direction::Through(p3), p0);
    let d3 = Point::orient_along_direction(p1, Direction::Through(p3), p2);

    if d0 == CoLinear || d1 == CoLinear || d2 == CoLinear || d3 == CoLinear {
      return Ok(false);
    }

    if d0 == d1 || d2 == d3 {
      return Ok(false);
    }

    let should_swap = if v0.is_super() || v2.is_super() {
      true
    } else if v1.is_super() || v2.is_super() {
      false
    } else {
      T::inside_circle(&p0.array, &p1.array, &p2.array, &p3.array)
    };

    if !should_swap {
      return Ok(false);
    }

    // swap
    let n0 = t1.neighbor(t1_t0_idx.cw());
    let n1 = t0.neighbor(t0_t1_idx.ccw());
    let n2 = t0.neighbor(t0_t1_idx.cw());
    let n3 = t1.neighbor(t1_t0_idx.ccw());

    *self.tri_mut(idx0) = Triangle {
      vertices: [v1, v2, v3],
      neighbors: [Some(idx1), n2, n3],
    };

    *self.tri_mut(idx1) = Triangle {
      vertices: [v1, v3, v0],
      neighbors: [n1, Some(idx0), n0],
    };

    // n0, n2 stays same, n1, n3 changes neighbor
    if let Some(idx) = n1 {
      self.tri_mut(idx).update_neighbor(idx0, idx1)?;
    }
    if let Some(idx) = n3 {
      self.tri_mut(idx).update_neighbor(idx1, idx0)?;
    }

    self.check_invariant_tri(idx0, "pre-swap idx0")?;
    self.check_invariant_tri(idx1, "pre-swap idx1")?;

    self.check_invariant_tri_opt(n0, "pre-swap n0")?;
    self.check_invariant_tri_opt(n1, "pre-swap n1")?;
    self.check_invariant_tri_opt(n2, "pre-swap n2")?;
    self.check_invariant_tri_opt(n3, "pre-swap n3")?;

    self.maybe_swap(idx0)?;
    self.maybe_swap(idx1)?;

    self.check_invariant("post-swap")?;

    Ok(true)
  }

  /// Add a new point to the network. Returns existing `VertIdx` of the point is already in the
  /// network.
  pub fn insert(&mut self, p: &Point<T>) -> Result<VertIdx> {
    use TriangularNetworkLocation::*;

    let res = match self.locate_recursive(p)? {
      InTriangle(idx_t) => {
        let idx_v = self.add_vert(p.clone());
        let t = self.tri(idx_t).clone();

        let idx_t0 = idx_t;
        let idx_t1 = self.add_tri();
        let idx_t2 = self.add_tri();

        let [v0, v1, v2] = t.vertices;
        let [n0, n1, n2] = t.neighbors;

        *self.tri_mut(idx_t0) = Triangle {
          vertices: [idx_v, v0, v1],
          neighbors: [Some(idx_t1), Some(idx_t2), n1],
        };
        *self.tri_mut(idx_t1) = Triangle {
          vertices: [idx_v, v1, v2],
          neighbors: [Some(idx_t2), Some(idx_t0), n2],
        };
        *self.tri_mut(idx_t2) = Triangle {
          vertices: [idx_v, v2, v0],
          neighbors: [Some(idx_t0), Some(idx_t1), n0],
        };

        if let Some(idx_neighbor) = n2 {
          self.tri_mut(idx_neighbor).update_neighbor(idx_t0, idx_t1)?;
        }
        if let Some(idx_neighbor) = n0 {
          self.tri_mut(idx_neighbor).update_neighbor(idx_t0, idx_t2)?;
        }

        self.check_invariant_tri(idx_t0, "InTriangle(t0)")?;
        self.check_invariant_tri(idx_t1, "InTriangle(t1)")?;
        self.check_invariant_tri(idx_t2, "InTriangle(t2)")?;

        self.maybe_swap(idx_t0)?;
        self.maybe_swap(idx_t1)?;
        self.maybe_swap(idx_t2)?;

        self.check_invariant("post-InTriangle")?;

        idx_v
      }
      OnVertex(tri, sub) => self.tri(tri).vert(sub),

      OnEdge(Edge {
        tri: idx_t,
        sub: idx_neighbor,
      }) => {
        let idx_t0 = idx_t;
        let t0 = self.tri(idx_t0).clone();

        let idx_t1 = t0.neighbor(idx_neighbor);

        let idx_t2 = self.add_tri();
        let idx_t3 = if idx_t1.is_some() {
          Some(self.add_tri())
        } else {
          None
        };

        let v0 = t0.vert(idx_neighbor);
        let v1 = t0.vert(idx_neighbor.ccw());
        let v2 = t0.vert(idx_neighbor.cw());
        let idx_v = self.add_vert(p.clone());

        //       v2(v0)
        //     t3     t2
        // v1     idx_v    v1
        //     t1     t0
        //       v0(v2)

        *self.tri_mut(idx_t0) = Triangle {
          vertices: [idx_v, v0, v1],
          neighbors: [Some(idx_t2), idx_t1, t0.neighbor(idx_neighbor.ccw())],
        };

        let n = t0.neighbor(idx_neighbor.cw());
        *self.tri_mut(idx_t2) = Triangle {
          vertices: [idx_v, v1, v2],
          neighbors: [idx_t3, Some(idx_t0), n],
        };
        if let Some(n) = n {
          self.tri_mut(n).update_neighbor(idx_t0, idx_t2)?;
        }

        if let Some(idx_t1) = idx_t1 {
          let idx_t3 = idx_t3.unwrap();

          let t1 = self.tri(idx_t1).clone();
          let idx_neighbor = t1.neighbor_idx(idx_t).ok_or(Error::InvariantViolation)?;

          let v0 = t1.vert(idx_neighbor);
          let v1 = t1.vert(idx_neighbor.ccw());
          let v2 = t1.vert(idx_neighbor.cw());

          *self.tri_mut(idx_t1) = Triangle {
            vertices: [idx_v, v1, v2],
            neighbors: [Some(idx_t0), Some(idx_t3), t1.neighbor(idx_neighbor.cw())],
          };

          let n = t1.neighbor(idx_neighbor.ccw());
          *self.tri_mut(idx_t3) = Triangle {
            vertices: [idx_v, v0, v1],
            neighbors: [Some(idx_t1), Some(idx_t2), n],
          };
          if let Some(n) = n {
            self.tri_mut(n).update_neighbor(idx_t1, idx_t3)?;
          }
        }

        self.check_invariant_tri(idx_t0, "Colinear(t0)")?;
        if let Some(idx_t1) = idx_t1 {
          self.check_invariant_tri(idx_t1, "Colinear(t1)")?;
        }
        self.check_invariant_tri(idx_t2, "Colinear(t2)")?;
        if let Some(idx_t3) = idx_t3 {
          self.check_invariant_tri(idx_t3, "Colinear(t3)")?;
        }

        self.maybe_swap(idx_t0)?;
        self.maybe_swap(idx_t2)?;
        if let Some(idx) = idx_t1 {
          self.maybe_swap(idx)?;
        }
        if let Some(idx) = idx_t3 {
          self.maybe_swap(idx)?;
        }

        self.check_invariant("post-Colinear")?;

        idx_v
      }

      Outside(_e) => {
        return Err(Error::InvariantViolation);
      }
    };
    Ok(res)
  }

  pub fn locate(&self, start: TriIdx, p: &Point<T>) -> TriangularNetworkLocation {
    use Orientation::*;
    use TriangularNetworkLocation::*;

    for i in 0..3 {
      if p == self.tri_vert(start, SubIdx(i)) {
        return OnVertex(start, SubIdx(i));
      }
    }

    let p0 = self.tri_vert(start, SubIdx(0));
    let p1 = self.tri_vert(start, SubIdx(1));
    let p2 = self.tri_vert(start, SubIdx(2));

    // 0, self <- ccw | cw -> 1
    let d0 = Point::orient_along_direction(p0, Direction::Through(p1), p);
    // 1, self <- ccw | cw -> 2
    let d1 = Point::orient_along_direction(p1, Direction::Through(p2), p);
    // 2, self <- ccw | cw -> 0
    let d2 = Point::orient_along_direction(p2, Direction::Through(p0), p);

    match (d0, d1, d2) {
      (CounterClockWise, CounterClockWise, CounterClockWise) => InTriangle(start),
      // handle cw case first
      (ClockWise, _, _) => Outside(Edge::new(start, SubIdx(1))),
      (_, ClockWise, _) => Outside(Edge::new(start, SubIdx(2))),
      (_, _, ClockWise) => Outside(Edge::new(start, SubIdx(0))),
      // TODO: (Colinear, Colinear, _) -> exact point?
      (CoLinear, CounterClockWise, CounterClockWise) => OnEdge(Edge::new(start, SubIdx(1))),
      (CounterClockWise, CoLinear, CounterClockWise) => OnEdge(Edge::new(start, SubIdx(2))),
      (CounterClockWise, CounterClockWise, CoLinear) => OnEdge(Edge::new(start, SubIdx(0))),
      _ => unreachable!(),
    }
  }

  pub fn locate_recursive(&self, p: &Point<T>) -> Result<TriangularNetworkLocation> {
    let mut start = TriIdx(0);

    loop {
      use TriangularNetworkLocation::*;

      start = match self.locate(start, p) {
        Outside(e) => {
          match self.tri(e.tri).neighbor(e.sub) {
            Some(idx) => idx,
            None => {
              // given point is outside of the network
              return Err(Error::InvariantViolation);
            }
          }
        }
        e => return Ok(e),
      };
    }
  }
}

impl<T: PolygonScalar> TryFrom<&Polygon<T>> for TriangularNetwork<T> {
  type Error = crate::Error;
  fn try_from(polygon: &Polygon<T>) -> std::result::Result<Self, Error> {
    let mut net = TriangularNetwork::new(
      polygon.cursor(polygon.boundary_slice()[0]).point().clone(),
      polygon.cursor(polygon.boundary_slice()[1]).point().clone(),
      polygon.cursor(polygon.boundary_slice()[2]).point().clone(),
    )?;
    for &pt in &polygon.boundary_slice()[3..] {
      net.insert(polygon.cursor(pt).point())?;
    }
    for edge in polygon.iter_boundary_edges() {
      let v0 = net.find_vert(edge.src).unwrap();
      let v1 = net.find_vert(edge.dst).unwrap();
      net.constrain_edge(v0, v1)?;
    }
    Ok(net)
  }
}

/// Triangle representation
#[derive(Clone)]
pub struct Triangle {
  /// list of vertex indices, in counterclockwise order
  pub vertices: [VertIdx; 3],
  /// list of neighbor triangle indices, in counterclockwise order
  pub neighbors: [Option<TriIdx>; 3],
}

impl std::fmt::Debug for Triangle {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(fmt, "Tri{{v=(")?;
    for (idx, v) in self.vertices.iter().enumerate() {
      let prefix = if idx == 0 { "" } else { ", " };
      write!(fmt, "{}{}", prefix, v.0)?;
    }
    write!(fmt, "), n=(")?;
    for (idx, n) in self.neighbors.iter().enumerate() {
      let prefix = if idx == 0 { "" } else { ", " };
      match n {
        Some(idx) => write!(fmt, "{}{}", prefix, idx.0)?,
        None => write!(fmt, "{}_", prefix)?,
      }
    }
    write!(fmt, ")}}")
  }
}

impl Triangle {
  /// Returns true if a triangle is part of supertriangle.
  pub fn is_super(&self) -> bool {
    let [v0, v1, v2] = self.vertices;
    v0.is_super() || v1.is_super() || v2.is_super()
  }

  pub fn vert(&self, idx: SubIdx) -> VertIdx {
    self.vertices[idx.0]
  }

  fn neighbor(&self, idx: SubIdx) -> Option<TriIdx> {
    self.neighbors[idx.0]
  }

  fn neighbor_mut(&mut self, idx: SubIdx) -> &mut Option<TriIdx> {
    &mut self.neighbors[idx.0]
  }

  fn update_neighbor(&mut self, idx_from: TriIdx, idx_to: TriIdx) -> Result<()> {
    for i in 0..3 {
      if self.neighbors[i] == Some(idx_from) {
        self.neighbors[i] = Some(idx_to);
        return Ok(());
      }
    }
    Err(Error::InvariantViolation)
  }

  #[allow(unused)]
  fn vertex_idx(&self, v_idx: VertIdx) -> Option<SubIdx> {
    self.vertices.iter().position(|p| *p == v_idx).map(SubIdx)
  }

  fn neighbor_idx(&self, idx: TriIdx) -> Option<SubIdx> {
    self
      .neighbors
      .iter()
      .position(|p| *p == Some(idx))
      .map(SubIdx)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use num_bigint::BigInt;

  #[test]
  fn locate() {
    use TriangularNetworkLocation::*;
    let p0 = Point::new([0.0, 0.0]);
    let p1 = Point::new([1.0, 0.0]);
    let p2 = Point::new([1.0, 1.0]);

    let net = TriangularNetwork::new(p0, p1, p2).unwrap();

    let cases = vec![
      (0.5, 0.0, OnEdge(Edge::new(TriIdx(0), SubIdx(1)))),
      (0.5, -0.1, Outside(Edge::new(TriIdx(0), SubIdx(1)))),
      (1.0, 0.5, OnEdge(Edge::new(TriIdx(0), SubIdx(2)))),
      (1.1, 0.5, Outside(Edge::new(TriIdx(0), SubIdx(2)))),
      (0.5, 0.5, OnEdge(Edge::new(TriIdx(0), SubIdx(0)))),
      (0.4, 0.6, Outside(Edge::new(TriIdx(0), SubIdx(0)))),
      (0.5, 0.1, InTriangle(TriIdx(0))),
    ];

    for (x, y, expected) in cases {
      assert_eq!(net.locate(TriIdx(0), &Point::new([x, y])), expected);
    }
  }

  fn trig_area_2x<F: PolygonScalar + Into<BigInt>>(p: &Polygon<F>) -> BigInt {
    let mut trig_area_2x = BigInt::from(0);
    let net = TriangularNetwork::try_from(p).unwrap();
    for trig in &net.triangles {
      let trig = TriangleView::new_unchecked([
        net.vert(trig.vertices[0]),
        net.vert(trig.vertices[1]),
        net.vert(trig.vertices[2]),
      ]);
      trig_area_2x += trig.signed_area_2x::<BigInt>();
    }
    trig_area_2x
  }

  #[test]
  fn basic_1() {
    let p = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([1, 1]),
    ])
    .unwrap();

    assert_eq!(p.signed_area_2x::<BigInt>(), trig_area_2x(&p));
  }

  #[test]
  fn basic_2() {
    let p = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([2, 0]),
      Point::new([3, 0]),
      Point::new([4, 0]),
      Point::new([1, 1]),
    ])
    .unwrap();

    assert_eq!(
      TriangularNetwork::try_from(&p).err(),
      Some(Error::CoLinearViolation)
    );
  }

  #[test]
  fn basic_3() {
    let p = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([1, 1]),
      Point::new([0, 1]),
    ])
    .unwrap();

    assert_eq!(p.signed_area_2x::<BigInt>(), trig_area_2x(&p));
  }

  use proptest::prelude::*;
  use test_strategy::proptest;

  #[proptest]
  fn equal_area_prop(poly: Polygon<i64>) {
    prop_assert_eq!(poly.signed_area_2x::<BigInt>(), trig_area_2x(&poly));
  }
}
