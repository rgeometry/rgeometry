use crate::data::{Point, PointId, PointLocation, TriangleView};
use crate::Orientation;
use crate::PolygonScalar;

use num_traits::Zero;
use rand::distributions::{Distribution, Standard};
use rand::rngs::mock::StepRng;
use rand::seq::SliceRandom;
use rand::Rng;

// FIXME: Rename to ear-clipping.

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
  rng: &'a mut R,
) -> impl Iterator<Item = (PointId, PointId, PointId)> + 'a
where
  T: PolygonScalar,
  R: Rng + ?Sized,
{
  let mut len = order.len();
  let mut vertices = List::new(order.len());
  let mut possible_ears = EarStore::new(order.len());
  std::iter::from_fn(move || match len {
    0..=2 => None,
    _ => loop {
      let focus = possible_ears.pop(rng).unwrap();
      let prev = vertices.prev(focus);
      let next = vertices.next(focus);
      if is_ear(points, order, &vertices, prev, focus, next) {
        possible_ears.new_possible_ear(prev);
        possible_ears.new_possible_ear(next);
        vertices.delete(focus);
        len -= 1;
        let out = (order[prev], order[focus], order[next]);
        return Some(out);
      }
    },
  })
}

fn is_ear<T>(
  points: &[Point<T, 2>],
  order: &[PointId],
  vertices: &List,
  a: usize,
  b: usize,
  c: usize,
) -> bool
where
  T: PolygonScalar,
{
  let get_point = |key: usize| &points[order[key].usize()];
  let trig = TriangleView::new_unchecked([get_point(a), get_point(b), get_point(c)]);
  if trig.orientation() == Orientation::CounterClockWise {
    let mut focus = vertices.next(c);
    while focus != a {
      if trig.locate(&points[focus]) != PointLocation::Outside {
        return false;
      }
      focus = vertices.next(focus);
    }
    true
  } else {
    false
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::*;
  use num_bigint::BigInt;
  use rand::SeedableRng;

  fn trig_area_2x<F: PolygonScalar + Into<BigInt>>(p: Polygon<F>) -> BigInt {
    let mut trig_area_2x = BigInt::zero();
    // let mut rng = StepRng::new(0,0);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
    for (a, b, c) in triangulate_list(&p.points, &p.rings[0], &mut rng) {
      let trig = TriangleView::new([p.point(a), p.point(b), p.point(c)]);
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

    assert_eq!(p.signed_area_2x::<BigInt>(), trig_area_2x(p));
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

    assert_eq!(p.signed_area_2x::<BigInt>(), trig_area_2x(p));
  }

  use proptest::prelude::*;
  use proptest::strategy::*;
  use proptest::test_runner::*;

  proptest! {
    // #[test]
    // fn all_random_convex_polygons_are_valid(poly: PolygonConvex<BigRational>) {
    //   prop_assert_eq!(poly.validate().err(), None)
    // }

    #[test]
    fn hash_unhash(a in any::<u32>(), b in any::<u32>()) {
      prop_assert_eq!(zunhash_pair(zhash_pair(a,b)), (a,b))

    }
  }
}

///////////////////////////////////////////////////////////////////////////////
// Z-Order Hash

// A        = aaaa =  a a a a
// B        = bbbb = b b b b
// zhash_pair(a,b) = babababa

pub struct ZHashBox<'a, T> {
  min_x: &'a T,
  max_x: &'a T,
  min_y: &'a T,
  max_y: &'a T,
}

pub trait ZHashable: Sized {
  type ZHashKey;
  fn zhash_key(zbox: ZHashBox<'_, Self>) -> Self::ZHashKey;
  fn zhash_fn(key: Self::ZHashKey, point: &Point<Self, 2>) -> u64;
}

impl ZHashable for f64 {
  type ZHashKey = (f64, f64, f64, f64);
  fn zhash_key(zbox: ZHashBox<'_, f64>) -> Self::ZHashKey {
    let width = zbox.max_x - zbox.min_x;
    let height = zbox.max_y - zbox.min_y;
    (*zbox.min_x, *zbox.min_y, width, height)
  }
  fn zhash_fn(key: Self::ZHashKey, point: &Point<Self, 2>) -> u64 {
    let (min_x, min_y, width, height) = key;
    let z_hash_max = u32::MAX as f64;
    let x = ((point.x_coord() - min_x) / width * z_hash_max) as u32;
    let y = ((point.y_coord() - min_y) / height * z_hash_max) as u32;
    zhash_pair(x, y)
  }
}

impl ZHashable for u64 {
  type ZHashKey = (u64, u64, u32, u32);
  fn zhash_key(zbox: ZHashBox<'_, u64>) -> Self::ZHashKey {
    let width = zbox.max_x - zbox.min_x;
    let height = zbox.max_y - zbox.min_y;
    let x_r_shift = 32u32.saturating_sub(width.leading_zeros());
    let y_r_shift = 32u32.saturating_sub(height.leading_zeros());
    (*zbox.min_x, *zbox.min_y, x_r_shift, y_r_shift)
  }
  fn zhash_fn(key: Self::ZHashKey, point: &Point<Self, 2>) -> u64 {
    let (min_x, min_y, x_r_shift, y_r_shift) = key;
    let x = ((*point.x_coord() - min_x) >> x_r_shift) as u32;
    let y = ((*point.y_coord() - min_y) >> y_r_shift) as u32;
    zhash_pair(x, y)
  }
}

impl ZHashable for u32 {
  type ZHashKey = ();
  fn zhash_key(_zbox: ZHashBox<'_, u32>) -> Self::ZHashKey {}
  fn zhash_fn(_key: Self::ZHashKey, point: &Point<Self, 2>) -> u64 {
    zhash_pair(*point.x_coord(), *point.y_coord())
  }
}

pub fn zunhash_pair(w: u64) -> (u32, u32) {
  (zunhash_u32(w), zunhash_u32(w >> 1))
}

fn zunhash_u32(w: u64) -> u32 {
  let w = w & 0x5555555555555555;
  let w = (w | w >> 1) & 0x3333333333333333;
  let w = (w | w >> 2) & 0x0F0F0F0F0F0F0F0F;
  let w = (w | w >> 4) & 0x00FF00FF00FF00FF;
  let w = (w | w >> 8) & 0x0000FFFF0000FFFF;
  let w = (w | w >> 16) & 0x00000000FFFFFFFF;
  w as u32
}

pub fn zhash_pair(a: u32, b: u32) -> u64 {
  zhash_u32(a) | zhash_u32(b) << 1
}

fn zhash_u32(w: u32) -> u64 {
  let w = w as u64; // & 0x00000000FFFFFFFF;
  let w = (w | w << 16) & 0x0000FFFF0000FFFF;
  let w = (w | w << 8) & 0x00FF00FF00FF00FF;
  let w = (w | w << 4) & 0x0F0F0F0F0F0F0F0F;
  let w = (w | w << 2) & 0x3333333333333333;
  (w | w << 1) & 0x5555555555555555
}

///////////////////////////////////////////////////////////////////////////////
// Linked List that supports deletions and re-insertions (of deleted items)

struct List {
  prev: Vec<usize>,
  next: Vec<usize>,
}

impl List {
  fn new(size: usize) -> List {
    let mut prev = Vec::with_capacity(size);
    let mut next = Vec::with_capacity(size);
    prev.resize(size, 0);
    next.resize(size, 0);
    for i in 0..size {
      prev[(i + 1) % size] = i;
      next[i] = (i + 1) % size;
    }
    List { prev, next }
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
    self.next[prev] = next;
    self.prev[next] = prev;
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
