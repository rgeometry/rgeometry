use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};

use num::BigRational;

use crate::Orientation;
use crate::algorithms::triangulation::earclip;
use crate::data::{IndexEdge, Point, PointId, Polygon};

/// Computes a constrained Delaunay triangulation for a simple polygon.
///
/// The implementation first ear-clips the polygon to obtain an initial
/// triangulation and then applies Lawson edge flips to restore the constrained
/// Delaunay condition while keeping boundary edges fixed. The worst-case time
/// complexity is $O(n^2)$.
///
/// # Panics
///
/// Panics if the polygon contains holes. Support for polygons with holes is
/// not yet implemented.
pub fn constrained_delaunay(poly: &Polygon<BigRational>) -> Vec<(PointId, PointId, PointId)> {
  assert!(
    poly.rings.len() == 1,
    "constrained Delaunay currently only supports simple polygons without holes"
  );

  let mut triangles: Vec<[PointId; 3]> =
    earclip::earclip(poly).map(|(a, b, c)| [a, b, c]).collect();

  if triangles.is_empty() {
    return Vec::new();
  }

  let constraint_edges = collect_constraint_edges(poly);
  let mut edge_map = build_edge_map(&triangles, &constraint_edges);
  let mut queue = VecDeque::new();
  for (&edge, info) in edge_map.iter() {
    if !info.constraint && info.triangles.len() == 2 {
      queue.push_back(edge);
    }
  }

  while let Some(edge) = queue.pop_front() {
    let Some(info) = edge_map.get(&edge) else {
      continue;
    };
    if info.constraint || info.triangles.len() != 2 {
      continue;
    }
    let [t1_idx, t2_idx] = [info.triangles[0], info.triangles[1]];
    let tri1 = triangles[t1_idx];
    let tri2 = triangles[t2_idx];

    let Some(opp1) = tri1.iter().copied().find(|pid| !tri2.contains(pid)) else {
      continue;
    };
    let tri1_rot = rotate_triangle(&tri1, opp1);
    let shared0 = tri1_rot[1];
    let shared1 = tri1_rot[2];
    if !tri2.contains(&shared0) || !tri2.contains(&shared1) {
      continue;
    }
    let opp2 = tri2
      .iter()
      .copied()
      .find(|pid| *pid != shared0 && *pid != shared1)
      .unwrap();

    // Skip non-convex quadrilaterals.
    let o1 = Point::orient(
      &poly.points[opp1.usize()],
      &poly.points[opp2.usize()],
      &poly.points[shared0.usize()],
    );
    let o2 = Point::orient(
      &poly.points[opp1.usize()],
      &poly.points[opp2.usize()],
      &poly.points[shared1.usize()],
    );
    if matches!(
      (o1, o2),
      (Orientation::CounterClockWise, Orientation::CounterClockWise)
        | (Orientation::ClockWise, Orientation::ClockWise)
    ) {
      continue;
    }

    let incircle = incircle_sign(&poly.points, opp1, shared0, shared1, opp2);
    if incircle != Ordering::Greater {
      continue;
    }

    let tri1_old_edges = triangle_edges(&triangles[t1_idx]);
    let tri2_old_edges = triangle_edges(&triangles[t2_idx]);
    remove_triangle_edges(&mut edge_map, &tri1_old_edges, t1_idx);
    remove_triangle_edges(&mut edge_map, &tri2_old_edges, t2_idx);

    triangles[t1_idx] = [opp1, shared1, opp2];
    triangles[t2_idx] = [opp2, shared0, opp1];

    ensure_ccw(&mut triangles[t1_idx], &poly.points);
    ensure_ccw(&mut triangles[t2_idx], &poly.points);

    let tri1_new_edges = triangle_edges(&triangles[t1_idx]);
    let tri2_new_edges = triangle_edges(&triangles[t2_idx]);
    add_triangle_edges(&mut edge_map, &constraint_edges, &tri1_new_edges, t1_idx);
    add_triangle_edges(&mut edge_map, &constraint_edges, &tri2_new_edges, t2_idx);

    for edge in tri1_new_edges.into_iter().chain(tri2_new_edges.into_iter()) {
      if let Some(info) = edge_map.get(&edge)
        && !info.constraint
        && info.triangles.len() == 2
      {
        queue.push_back(edge);
      }
    }
  }

  triangles
    .into_iter()
    .map(|tri| (tri[0], tri[1], tri[2]))
    .collect()
}

fn collect_constraint_edges(poly: &Polygon<BigRational>) -> HashSet<IndexEdge> {
  let mut set = HashSet::new();
  for ring in &poly.rings {
    for window in ring.windows(2) {
      let edge = IndexEdge::new(window[0], window[1]);
      set.insert(edge);
    }
    if let (Some(first), Some(last)) = (ring.first(), ring.last()) {
      set.insert(IndexEdge::new(*last, *first));
    }
  }
  set
}

fn build_edge_map(
  triangles: &[[PointId; 3]],
  constraints: &HashSet<IndexEdge>,
) -> HashMap<IndexEdge, EdgeInfo> {
  let mut map = HashMap::new();

  for &edge in constraints {
    map.insert(
      edge,
      EdgeInfo {
        constraint: true,
        triangles: Vec::new(),
      },
    );
  }

  for (idx, tri) in triangles.iter().enumerate() {
    for edge in triangle_edges(tri) {
      map
        .entry(edge)
        .or_insert_with(|| EdgeInfo {
          constraint: constraints.contains(&edge),
          triangles: Vec::new(),
        })
        .add_triangle(idx);
    }
  }

  map
}

fn rotate_triangle(tri: &[PointId; 3], first: PointId) -> [PointId; 3] {
  if tri[0] == first {
    [tri[0], tri[1], tri[2]]
  } else if tri[1] == first {
    [tri[1], tri[2], tri[0]]
  } else {
    [tri[2], tri[0], tri[1]]
  }
}

#[derive(Clone, Debug)]
struct EdgeInfo {
  constraint: bool,
  triangles: Vec<usize>,
}

impl EdgeInfo {
  fn add_triangle(&mut self, idx: usize) {
    if !self.triangles.contains(&idx) {
      self.triangles.push(idx);
    }
  }

  fn remove_triangle(&mut self, idx: usize) {
    if let Some(pos) = self.triangles.iter().position(|&other| other == idx) {
      self.triangles.swap_remove(pos);
    }
  }
}

fn triangle_edges(tri: &[PointId; 3]) -> [IndexEdge; 3] {
  [
    IndexEdge::new(tri[0], tri[1]),
    IndexEdge::new(tri[1], tri[2]),
    IndexEdge::new(tri[2], tri[0]),
  ]
}

fn remove_triangle_edges(
  edge_map: &mut HashMap<IndexEdge, EdgeInfo>,
  edges: &[IndexEdge; 3],
  tri_idx: usize,
) {
  for edge in edges {
    if let Some(info) = edge_map.get_mut(edge) {
      info.remove_triangle(tri_idx);
      if info.triangles.is_empty() && !info.constraint {
        edge_map.remove(edge);
      }
    }
  }
}

fn add_triangle_edges(
  edge_map: &mut HashMap<IndexEdge, EdgeInfo>,
  constraints: &HashSet<IndexEdge>,
  edges: &[IndexEdge; 3],
  tri_idx: usize,
) {
  for edge in edges {
    edge_map
      .entry(*edge)
      .or_insert_with(|| EdgeInfo {
        constraint: constraints.contains(edge),
        triangles: Vec::new(),
      })
      .add_triangle(tri_idx);
  }
}

fn ensure_ccw(tri: &mut [PointId; 3], points: &[Point<BigRational>]) {
  if Point::orient(
    &points[tri[0].usize()],
    &points[tri[1].usize()],
    &points[tri[2].usize()],
  )
  .is_cw()
  {
    tri.swap(1, 2);
  }
}

fn incircle_sign(
  points: &[Point<BigRational>],
  a: PointId,
  b: PointId,
  c: PointId,
  d: PointId,
) -> Ordering {
  let pa = &points[a.usize()];
  let pb = &points[b.usize()];
  let pc = &points[c.usize()];
  let pd = &points[d.usize()];

  let ax = pa.x_coord().clone() - pd.x_coord().clone();
  let ay = pa.y_coord().clone() - pd.y_coord().clone();
  let bx = pb.x_coord().clone() - pd.x_coord().clone();
  let by = pb.y_coord().clone() - pd.y_coord().clone();
  let cx = pc.x_coord().clone() - pd.x_coord().clone();
  let cy = pc.y_coord().clone() - pd.y_coord().clone();

  let a_len = ax.clone() * ax.clone() + ay.clone() * ay.clone();
  let b_len = bx.clone() * bx.clone() + by.clone() * by.clone();
  let c_len = cx.clone() * cx.clone() + cy.clone() * cy.clone();

  let det = det3(&ax, &ay, &a_len, &bx, &by, &b_len, &cx, &cy, &c_len);
  det.cmp(&BigRational::from_integer(0.into()))
}

#[allow(clippy::too_many_arguments)]
fn det3(
  a1: &BigRational,
  a2: &BigRational,
  a3: &BigRational,
  b1: &BigRational,
  b2: &BigRational,
  b3: &BigRational,
  c1: &BigRational,
  c2: &BigRational,
  c3: &BigRational,
) -> BigRational {
  let m1 = b2.clone() * c3.clone() - b3.clone() * c2.clone();
  let m2 = b1.clone() * c3.clone() - b3.clone() * c1.clone();
  let m3 = b1.clone() * c2.clone() - b2.clone() * c1.clone();
  a1.clone() * m1 - a2.clone() * m2 + a3.clone() * m3
}

#[cfg(test)]
mod tests {
  use super::*;
  use num::BigInt;
  use proptest::collection::vec;
  use proptest::prelude::*;
  use std::collections::{BTreeSet, HashSet};

  fn br(val: i64) -> BigRational {
    BigRational::from_integer(BigInt::from(val))
  }

  #[test]
  fn square_prefers_longer_diagonal() {
    let poly = Polygon::new_unchecked(vec![
      Point::new([br(0), br(0)]),
      Point::new([br(1), br(0)]),
      Point::new([br(1), br(1)]),
      Point::new([br(0), br(1)]),
    ]);

    let triangles = constrained_delaunay(&poly);
    let edges: HashSet<IndexEdge> = triangles
      .iter()
      .flat_map(|tri| {
        let arr = [tri.0, tri.1, tri.2];
        triangle_edges(&arr).into_iter()
      })
      .collect();
    let diag = IndexEdge::new(poly.rings[0][1], poly.rings[0][3]);
    assert!(edges.contains(&diag));
  }

  #[test]
  fn concave_polygon_respects_constraints() {
    let poly = Polygon::new_unchecked(vec![
      Point::new([br(0), br(0)]),
      Point::new([br(4), br(0)]),
      Point::new([br(4), br(4)]),
      Point::new([br(2), br(2)]),
      Point::new([br(0), br(4)]),
    ]);
    let triangles = constrained_delaunay(&poly);
    assert_eq!(triangles.len(), 3);

    let boundary: HashSet<IndexEdge> = collect_constraint_edges(&poly);
    let triset: HashSet<IndexEdge> = triangles
      .iter()
      .flat_map(|tri| {
        let arr = [tri.0, tri.1, tri.2];
        triangle_edges(&arr).into_iter()
      })
      .collect();
    for edge in boundary {
      assert!(triset.contains(&edge), "missing boundary edge {:?}", edge);
    }
  }

  fn small_polygons() -> impl Strategy<Value = Polygon<BigRational>> {
    vec((-10..=10, -10..=10), 3..8).prop_filter_map("simple polygon", |pts| {
      let mut set = BTreeSet::new();
      let mut points = Vec::new();
      for (x, y) in pts {
        if !set.insert((x, y)) {
          return None;
        }
        points.push(Point::new([
          BigRational::from_integer(BigInt::from(x)),
          BigRational::from_integer(BigInt::from(y)),
        ]));
      }
      Polygon::new(points).ok()
    })
  }

  proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]
    #[test]
    fn constrained_triangulation_is_delaunay(
      poly in small_polygons()
    ) {
      prop_assume!(poly.rings.len() == 1);

      let triangles = constrained_delaunay(&poly);
      let n = poly.rings[0].len();
      prop_assert_eq!(triangles.len(), n.saturating_sub(2));

      for tri in &triangles {
        let orientation = Point::orient(
          &poly.points[tri.0.usize()],
          &poly.points[tri.1.usize()],
          &poly.points[tri.2.usize()]
        );
        prop_assert!(orientation.is_ccw());
      }

      let constraints = collect_constraint_edges(&poly);
      let triangle_arrays: Vec<[PointId; 3]> = triangles
        .iter()
        .map(|tri| [tri.0, tri.1, tri.2])
        .collect();
      let adjacency = build_edge_map(&triangle_arrays, &constraints);

      for (&edge, info) in adjacency.iter() {
        if constraints.contains(&edge) {
          continue;
        }
        if info.triangles.len() != 2 {
          continue;
        }
        let t1_idx = info.triangles[0];
        let t2_idx = info.triangles[1];
        let tri1 = triangle_arrays[t1_idx];
        let tri2 = triangle_arrays[t2_idx];
        let Some(opp1) = tri1
          .iter()
          .copied()
          .find(|pid| !tri2.contains(pid))
        else {
          continue;
        };
        let tri1_rot = rotate_triangle(&tri1, opp1);
        let shared0 = tri1_rot[1];
        let shared1 = tri1_rot[2];
        prop_assert!(tri2.contains(&shared0));
        prop_assert!(tri2.contains(&shared1));
        let opp2 = tri2
          .iter()
          .copied()
          .find(|pid| *pid != shared0 && *pid != shared1)
          .unwrap();

        let sign = incircle_sign(&poly.points, opp1, shared0, shared1, opp2);
        prop_assert_ne!(sign, Ordering::Greater);
      }
    }
  }
}
