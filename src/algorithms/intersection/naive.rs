use crate::data::LineSegmentView;
use crate::{Intersects, PolygonScalar};

/// Find all line segment intersections.
///
/// # Time complexity
/// $O(n^2)$
pub fn segment_intersections<'a, Edge, T: PolygonScalar + 'a>(
  edges: &'a [Edge],
) -> impl Iterator<Item = (&'a Edge, &'a Edge)>
where
  &'a Edge: Into<LineSegmentView<'a, T, 2>>,
{
  pairs(edges).filter_map(|(a, b)| {
    let a_edge: LineSegmentView<'a, T, 2> = a.into();
    let b_edge: LineSegmentView<'a, T, 2> = b.into();
    let _isect = Intersects::intersect(a_edge, b_edge)?;
    Some((a, b))
  })
}

fn pairs<E>(slice: &[E]) -> impl Iterator<Item = (&E, &E)> {
  let n = slice.len();
  (0..n).flat_map(move |a| (0..a).map(move |b| (&slice[a], &slice[b])))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::{EndPoint, LineSegment, Point};
  use num::BigRational;
  use proptest::prelude::*;
  use std::collections::BTreeSet;

  fn scalar(n: i64) -> BigRational {
    BigRational::from_integer(n.into())
  }

  fn point(coords: (i64, i64)) -> Point<BigRational, 2> {
    let (x, y) = coords;
    Point::new([scalar(x), scalar(y)])
  }

  fn segment(a: (i64, i64), b: (i64, i64)) -> LineSegment<BigRational> {
    LineSegment::new(EndPoint::Inclusive(point(a)), EndPoint::Inclusive(point(b)))
  }

  type Scalar = BigRational;
  type PairKey = (usize, usize);

  fn pair_key(a: usize, b: usize) -> PairKey {
    if a < b {
      (a, b)
    } else {
      (b, a)
    }
  }

  fn collect_pairs<'a>(
    edges: &'a [LineSegment<Scalar>],
    iter: impl Iterator<Item = (&'a LineSegment<Scalar>, &'a LineSegment<Scalar>)>,
  ) -> BTreeSet<PairKey> {
    use std::collections::HashMap;
    let index_map: HashMap<*const LineSegment<Scalar>, usize> = edges
      .iter()
      .enumerate()
      .map(|(idx, seg)| (seg as *const _, idx))
      .collect();
    iter
      .map(|(a, b)| {
        let a_idx = index_map
          .get(&(a as *const LineSegment<Scalar>))
          .expect("segment not found");
        let b_idx = index_map
          .get(&(b as *const LineSegment<Scalar>))
          .expect("segment not found");
        pair_key(*a_idx, *b_idx)
      })
      .collect()
  }

  #[test]
  fn detects_single_crossing() {
    let segments = vec![
      segment((0, 0), (2, 2)),
      segment((0, 2), (2, 0)),
      segment((0, 3), (3, 3)),
    ];
    let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));
    let expected = [(0, 1)].into_iter().collect::<BTreeSet<_>>();
    assert_eq!(pairs, expected);
  }

  #[test]
  fn finds_multiple_crossings() {
    let segments = vec![
      segment((0, 0), (3, 3)),
      segment((0, 3), (3, 0)),
      segment((1, 3), (2, 0)),
      segment((1, 0), (2, 3)),
    ];
    let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));
    let expected = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]
      .into_iter()
      .collect::<BTreeSet<_>>();
    assert_eq!(pairs, expected);
  }

  #[test]
  fn no_false_positives() {
    let segments = vec![
      segment((0, 0), (1, 0)),
      segment((2, 0), (3, 0)),
      segment((0, 1), (0, 2)),
    ];
    let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));
    assert!(pairs.is_empty());
  }

  #[test]
  fn shared_endpoint_detected() {
    let segments = vec![
      segment((0, 0), (2, 0)),
      segment((2, 0), (2, 2)),
      segment((0, 2), (2, 0)),
    ];
    let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));
    let expected = [(0, 1), (0, 2), (1, 2)]
      .into_iter()
      .collect::<BTreeSet<_>>();
    assert_eq!(pairs, expected);
  }

  #[test]
  fn overlapping_segments_detected() {
    let segments = vec![
      segment((0, 0), (3, 0)),
      segment((1, 0), (4, 0)),
      segment((0, 1), (3, 1)),
    ];
    let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));
    let expected = [(0, 1)].into_iter().collect::<BTreeSet<_>>();
    assert_eq!(pairs, expected);
  }

  #[test]
  fn vertical_segments_crossing() {
    let segments = vec![
      segment((1, -1), (1, 2)),
      segment((0, 0), (3, 0)),
      segment((2, -1), (2, 2)),
    ];
    let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));
    let expected = [(0, 1), (1, 2)].into_iter().collect::<BTreeSet<_>>();
    assert_eq!(pairs, expected);
  }

  #[test]
  fn empty_segments() {
    let segments: Vec<LineSegment<Scalar>> = vec![];
    let pairs: BTreeSet<_> =
      collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));
    assert!(pairs.is_empty());
  }

  #[test]
  fn single_segment() {
    let segments = vec![segment((0, 0), (1, 1))];
    let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));
    assert!(pairs.is_empty());
  }

  #[test]
  fn collinear_segments() {
    let segments = vec![
      segment((0, 0), (2, 0)),
      segment((1, 0), (3, 0)),
      segment((2, 0), (4, 0)),
    ];
    let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));
    let expected = [(0, 1), (0, 2), (1, 2)].into_iter().collect::<BTreeSet<_>>();
    assert_eq!(pairs, expected);
  }

  #[test]
  fn parallel_segments() {
    let segments = vec![
      segment((0, 0), (2, 0)),
      segment((0, 1), (2, 1)),
      segment((0, 2), (2, 2)),
    ];
    let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));
    assert!(pairs.is_empty());
  }

  fn arb_segment() -> impl Strategy<Value = LineSegment<Scalar>> {
    let coord = -5..=5i8;
    (coord.clone(), coord.clone(), coord.clone(), coord)
      .prop_map(|(x1, y1, x2, y2)| {
        segment(
          (i64::from(x1), i64::from(y1)),
          (i64::from(x2), i64::from(y2)),
        )
      })
      .prop_filter("non-degenerate segment", |seg| {
        let min = seg.min.inner();
        let max = seg.max.inner();
        min != max
      })
  }

  proptest! {
    #[test]
    fn naive_no_self_intersections(segments in prop::collection::vec(arb_segment(), 0..10)) {
      let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));

      for &(a, b) in &pairs {
        prop_assert_ne!(a, b, "segment should not intersect with itself");
      }
    }

    #[test]
    fn naive_intersections_are_symmetric(segments in prop::collection::vec(arb_segment(), 0..6)) {
      let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));

      for &(a, b) in &pairs {
        prop_assert!(a < b, "pairs should be ordered (a < b)");
      }
    }

    #[test]
    fn naive_comprehensive_property(segments in prop::collection::vec(arb_segment(), 0..6)) {
      let pairs = collect_pairs(&segments, segment_intersections::<_, Scalar>(&segments));

      for &(a, b) in &pairs {
        let seg_a: LineSegmentView<_, 2> = (&segments[a]).into();
        let seg_b: LineSegmentView<_, 2> = (&segments[b]).into();
        let _isect = Intersects::intersect(seg_a, seg_b)
          .expect("intersecting segments should have intersection");
      }
    }
  }
}
