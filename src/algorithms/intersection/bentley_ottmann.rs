//! Bentley–Ottmann sweep-line intersection detection.
//!
//! This module implements the classic Bentley–Ottmann algorithm for finding all
//! pairwise intersections among a set of line segments. The sweep line advances
//! from left to right across the event queue, which is populated by the segment
//! endpoints and dynamically enriched with discovered intersection points. A
//! balanced search structure maintains the vertical ordering of the active
//! segments, making it possible to report every intersecting pair while
//! performing only logarithmic work per structural update.
//!
//! # High-level workflow
//! 1. **Event queue** – A priority queue ordered lexicographically on the
//!    `num::Rational` x/y coordinates, seeded with every segment endpoint.
//! 2. **Status structure** – An order-statistics tree storing the segments that
//!    currently intersect the sweep line. The ordering uses the current sweep
//!    position to evaluate each segment at the event's x-coordinate.
//! 3. **Processing** – For each event, active segments entering or leaving the
//!    sweep line are inserted or removed, neighbouring segments are checked for
//!    intersections, and any newly discovered intersection events are injected
//!    back into the queue.
//! 4. **Reporting** – Every time two segments are found to intersect, the pair
//!    is yielded to the caller. Degenerate overlaps are deduplicated so that
//!    each unordered pair is emitted at most once.
//!
//! The algorithm runs in `O((n + k) log n)` time where `n` is the number of
//! segments and `k` is the number of intersections that exist. Memory usage is
//! `O(n)` for the queue and status structure. The implementation is specialised
//! to `num::Rational` coordinates in order to guarantee exact arithmetic and to
//! avoid robustness issues when computing intersection points.
//!
//! # Testing guidance
//! - **Common cases** – Axes-aligned and random segments where crossings are
//!   easy to count by hand or compare against the `naive::segment_intersections`
//!   baseline. Include scenarios with multiple crossings sharing a common
//!   vertex.
//! - **Special cases** – Overlapping colinear segments, shared endpoints,
//!   vertical segments, and events where more than two segments meet at the
//!   same coordinate. These cases stress the tie-breaking in the event queue
//!   and the deduplication logic.
//! - **Tricky cases** – Intersections that occur to the left of their triggering
//!   event (caused by neighbouring swaps), segments that barely touch (e.g.
//!   one endpoint sitting precisely on another segment), and configurations that
//!   cause cascades of intersection events. These help validate that the sweep
//!   status stays consistent after local reordering.
//! - **Property checks** – Fuzz or proptest-style generators that compare the
//!   sweep output against the quadratic baseline for small input sizes, and
//!   invariants such as “all reported pairs are unique”, “the iterator never
//!   reports non-intersecting segments”, and “every true intersection eventually
//!   appears”. Running both algorithms on randomly generated segment sets with
//!   Rational coordinates is particularly effective because deviations can be
//!   spotted exactly.
#![allow(deprecated)]

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};

use crate::data::{EndPoint, LineSegmentView, Point};
use crate::Intersects;

use num::traits::Zero;
#[allow(deprecated)]
use num::Rational;

type PairKey = (usize, usize);

/// Find all line segment intersections using a Bentley–Ottmann sweep.
///
/// The input segments are required to use exact `num::Rational` coordinates.
pub fn segment_intersections<'a, Edge>(
  edges: &'a [Edge],
) -> impl Iterator<Item = (&'a Edge, &'a Edge)>
where
  &'a Edge: Into<LineSegmentView<'a, Rational, 2>>,
{
  let segments: Vec<Segment<'a, Edge>> = edges
    .iter()
    .enumerate()
    .map(|(idx, edge)| Segment::new(idx, edge))
    .collect();

  let mut queue = EventQueue::default();
  for segment in &segments {
    queue.add_upper(*segment.left.point, segment.index);
    queue.add_lower(*segment.right.point, segment.index);
  }

  let mut status = Status::new(&segments);
  let mut reported = HashSet::<PairKey>::new();
  let mut scheduled = HashMap::<PairKey, Point<Rational>>::new();
  let mut results = Vec::<PairKey>::new();

  while let Some((point, mut event)) = queue.pop() {
    event.dedup();

    for_each_pair(&event.crossing, |pair| {
      if let Some(scheduled_point) = scheduled.get(&pair) {
        if scheduled_point == &point {
          // This event is now live; drop the pending marker.
          scheduled.remove(&pair);
        }
      }
    });

    let participants = event.all_segments();
    report_intersections(
      &participants,
      &segments,
      &mut reported,
      &mut results,
      &mut scheduled,
    );

    status.set_sweep_point(point);
    status.remove_all(&event.lowers);
    status.remove_all(&event.crossing);
    status.insert_all(&event.uppers);
    status.insert_all(&event.crossing);
    status.resort();

    let adjacent_pairs = status.adjacent_pairs();
    let mut processor = PairProcessor {
      segments: &segments,
      scheduled: &mut scheduled,
      reported: &mut reported,
      results: &mut results,
    };
    for (a, b) in adjacent_pairs {
      processor.process(pair_key(a, b), status.current_point(), &mut queue);
    }
  }

  let edge_lookup: Vec<&'a Edge> = segments.iter().map(|segment| segment.edge).collect();
  results
    .into_iter()
    .map(move |(a, b)| (edge_lookup[a], edge_lookup[b]))
}

fn report_intersections<'a, Edge>(
  participants: &[usize],
  segments: &[Segment<'a, Edge>],
  reported: &mut HashSet<PairKey>,
  results: &mut Vec<PairKey>,
  scheduled: &mut HashMap<PairKey, Point<Rational>>,
) where
  &'a Edge: Into<LineSegmentView<'a, Rational, 2>>,
{
  if participants.len() < 2 {
    return;
  }
  for_each_pair(participants, |pair| {
    if reported.contains(&pair) {
      return;
    }
    let (a, b) = pair;
    if segments[a].view.intersect(segments[b].view).is_some() {
      reported.insert(pair);
      scheduled.remove(&pair);
      results.push(pair);
    }
  });
}

fn for_each_pair<F>(items: &[usize], mut f: F)
where
  F: FnMut(PairKey),
{
  for i in 0..items.len() {
    for j in (i + 1)..items.len() {
      f(pair_key(items[i], items[j]));
    }
  }
}

fn pair_key(a: usize, b: usize) -> PairKey {
  if a < b {
    (a, b)
  } else {
    (b, a)
  }
}

struct PairProcessor<'segments, 'state, 'edge, Edge> {
  segments: &'segments [Segment<'edge, Edge>],
  scheduled: &'state mut HashMap<PairKey, Point<Rational>>,
  reported: &'state mut HashSet<PairKey>,
  results: &'state mut Vec<PairKey>,
}

impl<'edge, Edge> PairProcessor<'_, '_, 'edge, Edge>
where
  &'edge Edge: Into<LineSegmentView<'edge, Rational, 2>>,
{
  fn process(&mut self, pair: PairKey, current_point: &Point<Rational>, queue: &mut EventQueue) {
    if self.reported.contains(&pair) || self.scheduled.contains_key(&pair) {
      return;
    }
    let (a, b) = pair;
    let intersection = self.segments[a].view.intersect(self.segments[b].view);
    let Some(intersection) = intersection else {
      return;
    };
    match intersection {
      crate::data::ILineSegment::Overlap(_) => {
        if self.reported.insert(pair) {
          self.results.push(pair);
        }
        self.scheduled.remove(&pair);
      }
      crate::data::ILineSegment::Crossing => {
        if let Some(point) = self.segments[a].intersection_point(&self.segments[b]) {
          match point.cmp(current_point) {
            Ordering::Less | Ordering::Equal => {
              if self.reported.insert(pair) {
                self.results.push(pair);
              }
              self.scheduled.remove(&pair);
            }
            Ordering::Greater => {
              self.scheduled.insert(pair, point);
              queue.add_crossing(point, a, b);
            }
          }
        }
      }
    }
  }
}

#[derive(Default)]
struct EventQueue {
  map: BTreeMap<Point<Rational>, EventData>,
}

impl EventQueue {
  fn add_upper(&mut self, point: Point<Rational>, segment: usize) {
    self.map.entry(point).or_default().uppers.push(segment);
  }

  fn add_lower(&mut self, point: Point<Rational>, segment: usize) {
    self.map.entry(point).or_default().lowers.push(segment);
  }

  fn add_crossing(&mut self, point: Point<Rational>, a: usize, b: usize) {
    let entry = self.map.entry(point).or_default();
    entry.crossing.push(a);
    entry.crossing.push(b);
  }

  fn pop(&mut self) -> Option<(Point<Rational>, EventData)> {
    let first_key = self.map.keys().next().copied()?;
    let data = self.map.remove(&first_key)?;
    Some((first_key, data))
  }
}

#[derive(Default)]
struct EventData {
  uppers: Vec<usize>,
  lowers: Vec<usize>,
  crossing: Vec<usize>,
}

impl EventData {
  fn dedup(&mut self) {
    Self::dedup_vec(&mut self.uppers);
    Self::dedup_vec(&mut self.lowers);
    Self::dedup_vec(&mut self.crossing);
  }

  fn all_segments(&self) -> Vec<usize> {
    let mut all = Vec::with_capacity(self.uppers.len() + self.lowers.len() + self.crossing.len());
    all.extend(&self.uppers);
    all.extend(&self.lowers);
    all.extend(&self.crossing);
    Self::dedup_vec(&mut all);
    all
  }

  fn dedup_vec(vec: &mut Vec<usize>) {
    vec.sort_unstable();
    vec.dedup();
  }
}

struct Status<'s, 'a, Edge> {
  active: Vec<usize>,
  segments: &'s [Segment<'a, Edge>],
  current: Point<Rational>,
}

impl<'s, 'a, Edge> Status<'s, 'a, Edge>
where
  &'a Edge: Into<LineSegmentView<'a, Rational, 2>>,
{
  fn new(segments: &'s [Segment<'a, Edge>]) -> Self {
    Status {
      active: Vec::new(),
      segments,
      current: Point::new([Rational::from_integer(0), Rational::from_integer(0)]),
    }
  }

  fn set_sweep_point(&mut self, point: Point<Rational>) {
    self.current = point;
  }

  fn current_point(&self) -> &Point<Rational> {
    &self.current
  }

  fn remove_all(&mut self, segments: &[usize]) {
    for id in segments {
      if let Some(idx) = self.active.iter().position(|cand| cand == id) {
        self.active.remove(idx);
      }
    }
  }

  fn insert_all(&mut self, segments: &[usize]) {
    for id in segments {
      if !self.active.contains(id) {
        self.active.push(*id);
      }
    }
  }

  fn resort(&mut self) {
    self
      .active
      .sort_by(|a, b| compare_segments(&self.segments[*a], &self.segments[*b], &self.current));
  }

  fn adjacent_pairs(&self) -> Vec<PairKey> {
    let mut pairs = Vec::new();
    for window in self.active.windows(2) {
      if let [a, b] = window {
        pairs.push(pair_key(*a, *b));
      }
    }
    pairs
  }
}

fn compare_segments<'a, Edge>(
  a: &Segment<'a, Edge>,
  b: &Segment<'a, Edge>,
  sweep_point: &Point<Rational>,
) -> Ordering
where
  &'a Edge: Into<LineSegmentView<'a, Rational, 2>>,
{
  if a.index == b.index {
    return Ordering::Equal;
  }
  let x = &sweep_point.array[0];
  let ay = a.value_at(x);
  let by = b.value_at(x);
  match ay.cmp(&by) {
    Ordering::Equal => {
      let a_slope = a.slope();
      let b_slope = b.slope();
      match (a_slope, b_slope) {
        (Some(a_slope), Some(b_slope)) => {
          let order = a_slope.cmp(&b_slope);
          if order == Ordering::Equal {
            a.index.cmp(&b.index)
          } else {
            order
          }
        }
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (None, None) => a.index.cmp(&b.index),
      }
    }
    ordering => ordering,
  }
}

struct Segment<'a, Edge> {
  index: usize,
  edge: &'a Edge,
  view: LineSegmentView<'a, Rational, 2>,
  left: EndpointRef<'a>,
  right: EndpointRef<'a>,
}

impl<'a, Edge> Segment<'a, Edge>
where
  &'a Edge: Into<LineSegmentView<'a, Rational, 2>>,
{
  fn new(index: usize, edge: &'a Edge) -> Self {
    let view: LineSegmentView<'a, Rational, 2> = edge.into();
    let left = EndpointRef::from(view.min);
    let right = EndpointRef::from(view.max);
    Segment {
      index,
      edge,
      view,
      left,
      right,
    }
  }

  fn value_at(&self, x: &Rational) -> Rational {
    let x1 = *self.left.x();
    let y1 = *self.left.y();
    let x2 = *self.right.x();
    let y2 = *self.right.y();

    if x1 == x2 {
      return if y1 <= y2 { y1 } else { y2 };
    }

    let dx = x2 - x1;
    if dx.is_zero() {
      return y1;
    }
    let t = (*x - x1) / dx;
    let dy = y2 - y1;
    y1 + dy * t
  }

  fn slope(&self) -> Option<Rational> {
    let dx = *self.right.x() - *self.left.x();
    if dx.is_zero() {
      return None;
    }
    Some((*self.right.y() - *self.left.y()) / dx)
  }

  fn intersection_point(&self, other: &Self) -> Option<Point<Rational>> {
    let p1x = *self.left.x();
    let p1y = *self.left.y();
    let p2x = *self.right.x();
    let p2y = *self.right.y();
    let q1x = *other.left.x();
    let q1y = *other.left.y();
    let q2x = *other.right.x();
    let q2y = *other.right.y();

    let denom = (p1x - p2x) * (q1y - q2y) - (p1y - p2y) * (q1x - q2x);
    if denom.is_zero() {
      return None;
    }
    let part_a = p1x * p2y - p1y * p2x;
    let part_b = q1x * q2y - q1y * q2x;
    let x_num = part_a * (q1x - q2x) - (p1x - p2x) * part_b;
    let y_num = part_a * (q1y - q2y) - (p1y - p2y) * part_b;
    Some(Point::new([x_num / denom, y_num / denom]))
  }
}

struct EndpointRef<'a> {
  point: &'a Point<Rational, 2>,
  inclusive: bool,
}

impl EndpointRef<'_> {
  fn x(&self) -> &Rational {
    &self.point.array[0]
  }

  fn y(&self) -> &Rational {
    &self.point.array[1]
  }

  #[allow(dead_code)]
  fn inclusive(&self) -> bool {
    self.inclusive
  }
}

impl<'a> From<EndPoint<&'a Point<Rational, 2>>> for EndpointRef<'a> {
  fn from(endpoint: EndPoint<&'a Point<Rational, 2>>) -> Self {
    match endpoint {
      EndPoint::Inclusive(point) => EndpointRef {
        point,
        inclusive: true,
      },
      EndPoint::Exclusive(point) => EndpointRef {
        point,
        inclusive: false,
      },
    }
  }
}
