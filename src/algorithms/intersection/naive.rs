use crate::data::LineSegmentView;
use crate::Intersects;

/// $O(n^2)$ Finds all line segment intersections.
pub fn segment_intersections<'a, Edge, T: 'a>(
  edges: &'a [Edge],
) -> impl Iterator<Item = (&Edge, &Edge)>
where
  &'a Edge: Into<LineSegmentView<'a, T, 2>>,
  T: Clone + num_traits::NumOps<T, T> + Ord + std::fmt::Debug,
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
  (0..n)
    .map(move |a| (0..a).map(move |b| (&slice[a], &slice[b])))
    .flatten()
}
