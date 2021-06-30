use crate::data::{Point, PointId, PointLocation, TriangleView};
use crate::Orientation;
use crate::PolygonScalar;

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
  use num_traits::Zero;
  use rand::SeedableRng;

  fn trig_area_2x<F: PolygonScalar + Into<BigInt>>(p: &Polygon<F>) -> BigInt {
    let mut trig_area_2x = BigInt::zero();
    // let mut rng = StepRng::new(0,0);
    let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
    for (a, b, c) in triangulate_list(&p.points, &p.rings[0], &mut rng) {
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

  use proptest::prelude::*;

  proptest! {
    #[test]
    fn equal_area_prop(poly: Polygon<i64>) {
      prop_assert_eq!(poly.signed_area_2x::<BigInt>(), trig_area_2x(&poly));
    }
  }
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
