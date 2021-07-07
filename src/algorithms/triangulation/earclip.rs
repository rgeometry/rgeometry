use std::borrow::BorrowMut;

use crate::algorithms::zhash::{ZHashBox, ZHashable};
use crate::data::{Point, PointId, PointLocation, Polygon, TriangleView};
use crate::Orientation;
use crate::PolygonScalar;

use num_traits::Zero;
use rand::rngs::mock::StepRng;
use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

/// $O(n^2)$ Polygon triangulation. Ears are selected in a pseudo-random manner.
pub fn earclip<T>(poly: &Polygon<T>) -> impl Iterator<Item = (PointId, PointId, PointId)> + '_
where
  T: PolygonScalar,
{
  // FIXME: Support holes.
  assert!(poly.rings.len() == 1);
  // let rng = StepRng::new(0, 0);
  let rng = SmallRng::seed_from_u64(0xDEADBEEF);
  triangulate_list(&poly.points, &poly.rings[0], rng)
}

/// $O(n)$ Polygon triangulation. Ears are selected in a pseudo-random manner.
///
/// $O(n^2)$ worst case complexity. Expected time is linear.
pub fn earclip_hashed<T>(
  poly: &Polygon<T>,
) -> impl Iterator<Item = (PointId, PointId, PointId)> + '_
where
  T: PolygonScalar + ZHashable,
{
  // FIXME: Support holes.
  assert!(poly.rings.len() == 1);
  // let rng = StepRng::new(0, 0);
  let rng = SmallRng::seed_from_u64(0xDEADBEEF);
  triangulate_list_hashed(&poly.points, &poly.rings[0], rng)
}

// triangulate: Vec<Point<T,2>> + Vec<PointId> -> Vec<(PointId,PointId,PointId)>
// triangulate: Polygon -> Vec<Polygon>
// triangulate: Polygon -> Vec<(PointId, PointId, PointId)>

// Create a linked list of points. O(n)
// Clone it as a list of possible ears. O(n)
// For each vertex in the list of possible ears:
//   Check if the vertex is an ear. O(n)
//   Delete it from the list of possible-ears.
//   If it is:
//     Delete it from the vertex list.
//     Insert vertex->prev and vertex->next into possible-ears if they aren't there already.
//     Emit an edge: (vertex.prev, ear, vertex.next)
pub fn triangulate_list<'a, T, R>(
  points: &'a [Point<T, 2>],
  order: &'a [PointId],
  mut rng: R,
) -> impl Iterator<Item = (PointId, PointId, PointId)> + 'a
where
  T: PolygonScalar,
  R: Rng + 'static,
{
  let mut len = order.len();
  let mut vertices = List::new(points, order);
  let mut possible_ears = EarStore::new(order.len());
  std::iter::from_fn(move || match len {
    0..=2 => None,
    _ => loop {
      let focus = vertices.cursor(possible_ears.pop(&mut rng).unwrap());
      let prev = focus.prev();
      let next = focus.next();
      if is_ear(prev, focus, next) {
        possible_ears.new_possible_ear(prev.position);
        possible_ears.new_possible_ear(next.position);
        let out = (prev.point_id(), focus.point_id(), next.point_id());
        let focus = focus.position;
        vertices.delete(focus);
        len -= 1;
        return Some(out);
      }
    },
  })
}

fn is_ear<T>(a: Cursor<'_, T>, b: Cursor<'_, T>, c: Cursor<'_, T>) -> bool
where
  T: PolygonScalar,
{
  let trig = TriangleView::new_unchecked([a.point(), b.point(), c.point()]);
  if trig.orientation() == Orientation::CounterClockWise {
    let mut focus = c.next();
    while focus != a {
      if trig.locate(focus.point()) != PointLocation::Outside {
        return false;
      }
      focus = focus.next();
    }
    true
  } else {
    false
  }
}

///////////////////////////////////////////////////////////////////////////////
// Z-order hash ear clipping

pub fn triangulate_list_hashed<'a, T, R>(
  points: &'a [Point<T, 2>],
  order: &'a [PointId],
  mut rng: R,
) -> impl Iterator<Item = (PointId, PointId, PointId)> + 'a
where
  T: PolygonScalar + ZHashable,
  R: Rng + 'static,
{
  let mut len = order.len();
  let mut vertices = List::new(points, order);
  let mut possible_ears = EarStore::new(order.len());

  let zbox = zbox_slice(points, order);
  let key = ZHashable::zhash_key(zbox);
  let zhashes: Vec<u64> = order
    .iter()
    .map(|&pid| ZHashable::zhash_fn(key, &points[pid.usize()]))
    .collect();
  let mut zorder = List::new_sorted(points, order, zhashes);

  std::iter::from_fn(move || match len {
    0..=2 => None,
    _ => loop {
      let focus = possible_ears.pop(&mut rng).unwrap();
      let prev = vertices.prev(focus);
      let next = vertices.next(focus);
      if is_ear_hashed(
        key,
        zorder.cursor(prev),
        zorder.cursor(focus),
        zorder.cursor(next),
      ) {
        possible_ears.new_possible_ear(prev);
        possible_ears.new_possible_ear(next);
        vertices.delete(focus);
        zorder.delete(focus);
        len -= 1;
        let out = (order[prev], order[focus], order[next]);
        return Some(out);
      }
    },
  })
}

fn is_ear_hashed<T: ZHashable>(
  key: <T as ZHashable>::ZHashKey,
  a: Cursor<'_, T>,
  b: Cursor<'_, T>,
  c: Cursor<'_, T>,
) -> bool
where
  T: PolygonScalar,
{
  let trig = TriangleView::new_unchecked([a.point(), b.point(), c.point()]);
  if trig.orientation() == Orientation::CounterClockWise {
    // Points inside the triangle are guaranteed to have a zhash
    // between the hashes of the bounding box.
    let (min, max) = trig.bounding_box();
    let min_hash = ZHashable::zhash_fn(key, &min);
    let max_hash = ZHashable::zhash_fn(key, &max);

    // Points inside the triangle are likely to be nearby so start
    // by searching in both directions.
    let mut up_focus = b.next();
    let mut down_focus = b.prev();

    let cond = |cursor: Cursor<'_, T>| cursor.valid() && cursor.between(min_hash, max_hash);
    let check = |cursor: Cursor<'_, T>| {
      cursor != a
        && cursor != b
        && cursor != c
        && trig.locate(cursor.point()) != PointLocation::Outside
    };

    while cond(up_focus) && cond(down_focus) {
      if check(up_focus) || check(down_focus) {
        return false;
      }
      up_focus = up_focus.next();
      down_focus = down_focus.prev();
    }

    // Look upwards
    while cond(up_focus) {
      if check(up_focus) {
        return false;
      }
      up_focus = up_focus.next();
    }

    // Look downwards
    while cond(down_focus) {
      if check(down_focus) {
        return false;
      }
      down_focus = down_focus.prev();
    }

    true
  } else {
    false
  }
}

fn zbox_slice<'a, T>(slice: &'a [Point<T, 2>], order: &'a [PointId]) -> ZHashBox<'a, T>
where
  T: PolygonScalar + ZHashable,
{
  if slice.is_empty() {
    panic!("Cannot zhash an empty slice");
    // return ZHashBox {
    //   min_x: &T::zero(),
    //   max_x: &T::zero(),
    //   min_y: &T::zero(),
    //   max_y: &T::zero(),
    // };
  }
  let mut zbox = ZHashBox {
    min_x: slice[0].x_coord(),
    max_x: slice[0].x_coord(),
    min_y: slice[0].y_coord(),
    max_y: slice[0].y_coord(),
  };
  for &pid in order {
    let pt = &slice[pid.usize()];
    if zbox.min_x > pt.x_coord() {
      zbox.min_x = pt.x_coord();
    }
    if zbox.max_x < pt.x_coord() {
      zbox.max_x = pt.x_coord();
    }
    if zbox.min_y > pt.y_coord() {
      zbox.min_y = pt.y_coord();
    }
    if zbox.max_y < pt.y_coord() {
      zbox.max_y = pt.y_coord();
    }
  }
  zbox
}

///////////////////////////////////////////////////////////////////////////////
// Tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::*;
  use num_bigint::BigInt;
  use num_traits::Zero;
  use rand::rngs::SmallRng;
  use rand::SeedableRng;

  fn trig_area_2x<F: PolygonScalar + Into<BigInt>>(p: &Polygon<F>) -> BigInt {
    let mut trig_area_2x = BigInt::zero();
    // let mut rng = StepRng::new(0,0);
    let rng = SmallRng::seed_from_u64(0);
    for (a, b, c) in triangulate_list(&p.points, &p.rings[0], rng) {
      let trig = TriangleView::new_unchecked([p.point(a), p.point(b), p.point(c)]);
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

    assert_eq!(p.signed_area_2x::<BigInt>(), trig_area_2x(&p));
  }

  #[test]
  fn basic_3() {
    let rng = SmallRng::seed_from_u64(0);
    let p: Polygon<i8> = Polygon::new(vec![
      Point { array: [-44, -11] },
      Point { array: [-43, 23] },
      Point { array: [-64, 44] },
      Point { array: [-52, 114] },
      Point { array: [-82, 69] },
    ])
    .unwrap();

    triangulate_list_hashed(&p.points, &p.rings[0], rng).count();
  }

  #[test]
  fn basic_4() {
    let rng = SmallRng::seed_from_u64(0);
    let p: Polygon<i8> = Polygon::new(vec![
      Point { array: [12, 5] },   // 0
      Point { array: [0, 8] },    // 1
      Point { array: [-10, -6] }, // 2 cut
      Point { array: [-3, 3] },   // 3
      Point { array: [-2, 4] },   // 4
    ])
    .unwrap();

    triangulate_list_hashed(&p.points, &p.rings[0], rng).count();
  }

  use proptest::prelude::*;
  use test_strategy::proptest;

  #[proptest]
  fn equal_area_prop(poly: Polygon<i64>) {
    prop_assert_eq!(poly.signed_area_2x::<BigInt>(), trig_area_2x(&poly));
  }

  #[proptest]
  fn hashed_identity_prop(poly: Polygon<i8>) {
    let rng = SmallRng::seed_from_u64(0);
    let not_hashed: Vec<(PointId, PointId, PointId)> =
      triangulate_list(&poly.points, &poly.rings[0], rng.clone()).collect();
    let hashed: Vec<(PointId, PointId, PointId)> =
      triangulate_list_hashed(&poly.points, &poly.rings[0], rng).collect();
    prop_assert_eq!(not_hashed, hashed);
  }
}

///////////////////////////////////////////////////////////////////////////////
// Linked List that supports deletions and re-insertions (of deleted items)

struct Cursor<'a, T> {
  list: &'a List<'a, T>,
  position: usize,
}

impl<'a, T> Eq for Cursor<'a, T> {}

impl<'a, T> PartialEq for Cursor<'a, T> {
  fn eq(&self, other: &Cursor<'a, T>) -> bool {
    self.position == other.position
  }
}

impl<'a, T> Copy for Cursor<'a, T> {}

impl<'a, T> Clone for Cursor<'a, T> {
  fn clone(&self) -> Cursor<'a, T> {
    Cursor {
      list: self.list,
      position: self.position,
    }
  }
}

impl<'a, T> Cursor<'a, T> {
  fn valid(&self) -> bool {
    self.position != usize::MAX
  }

  fn next(mut self) -> Cursor<'a, T> {
    self.position = self.list.next(self.position);
    self
  }

  fn prev(mut self) -> Cursor<'a, T> {
    self.position = self.list.prev(self.position);
    self
  }

  fn point(&self) -> &Point<T, 2> {
    self.list.point(self.position)
  }

  fn point_id(&self) -> PointId {
    self.list.point_id(self.position)
  }

  fn hash(&self) -> u64 {
    self.list.hash(self.position)
  }

  fn between(&self, min: u64, max: u64) -> bool {
    self.hash() >= min && self.hash() <= max
  }
}

struct List<'a, T> {
  points: &'a [Point<T, 2>],
  order: &'a [PointId],
  hashes: Vec<u64>,
  prev: Vec<usize>,
  next: Vec<usize>,
}

impl<'a, T> List<'a, T> {
  fn new(points: &'a [Point<T, 2>], order: &'a [PointId]) -> List<'a, T> {
    let size = order.len();
    let mut prev = Vec::with_capacity(size);
    let mut next = Vec::with_capacity(size);
    prev.resize(size, 0);
    next.resize(size, 0);
    for i in 0..size {
      prev[(i + 1) % size] = i;
      next[i] = (i + 1) % size;
    }
    List {
      points,
      order,
      hashes: vec![],
      prev,
      next,
    }
  }

  fn prev(&self, vertex: usize) -> usize {
    self.prev[vertex]
  }

  fn next(&self, vertex: usize) -> usize {
    self.next[vertex]
  }

  fn delete(&mut self, vertex: usize) {
    let prev = self.prev[vertex];
    let next = self.next[vertex];
    if prev != usize::MAX {
      self.next[prev] = next;
    }
    if next != usize::MAX {
      self.prev[next] = prev;
    }
  }

  fn cursor(&self, vertex: usize) -> Cursor<'_, T> {
    Cursor {
      list: self,
      position: vertex,
    }
  }

  fn point(&self, vertex: usize) -> &Point<T, 2> {
    &self.points[self.point_id(vertex).usize()]
  }

  fn point_id(&self, vertex: usize) -> PointId {
    self.order[vertex]
  }

  fn hash(&self, vertex: usize) -> u64 {
    self.hashes[vertex]
  }
}

impl<'a, T> List<'a, T> {
  fn new_sorted(points: &'a [Point<T, 2>], order: &'a [PointId], keys: Vec<u64>) -> List<'a, T> {
    let size = keys.len();
    let mut v: Vec<usize> = (0..size).collect();
    v.sort_unstable_by_key(|&idx| keys[idx]);
    let mut prev = Vec::with_capacity(size);
    let mut next = Vec::with_capacity(size);
    prev.resize(size, 0);
    next.resize(size, 0);
    for i in 0..size {
      prev[v[(i + 1) % size]] = v[i];
      next[v[i]] = v[(i + 1) % size];
    }
    next[v[size - 1]] = usize::MAX;
    prev[v[0]] = usize::MAX;
    List {
      points,
      order,
      hashes: keys,
      prev,
      next,
    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Collection of possible ears.

struct EarStore {
  possible_ears_vec: Vec<usize>,
  possible_ears_set: IntSet,
}

impl EarStore {
  fn new(size: usize) -> EarStore {
    EarStore {
      possible_ears_vec: (0..size).collect(),
      possible_ears_set: IntSet::with_capacity(size),
    }
  }

  fn new_possible_ear(&mut self, possible_ear: usize) {
    if !self.possible_ears_set.contains(possible_ear) {
      self.possible_ears_set.insert(possible_ear);
      self.possible_ears_vec.push(possible_ear);
    }
  }

  fn pop<R>(&mut self, rng: &mut R) -> Option<usize>
  where
    R: Rng + ?Sized,
  {
    let n = rng.gen_range(0..self.possible_ears_vec.len());
    let next = self.possible_ears_vec.swap_remove(n);
    self.possible_ears_set.delete(next);
    Some(next)
  }
}

///////////////////////////////////////////////////////////////////////////////
// IntSet

// FIXME: Use a bitset?
struct IntSet {
  set: Vec<bool>,
}

impl IntSet {
  fn with_capacity(capacity: usize) -> IntSet {
    let mut set = Vec::with_capacity(capacity);
    set.resize(capacity, true);
    IntSet { set }
  }

  fn contains(&self, value: usize) -> bool {
    self.set[value]
  }

  fn insert(&mut self, value: usize) {
    self.set[value] = true
  }

  fn delete(&mut self, value: usize) {
    self.set[value] = false
  }
}
