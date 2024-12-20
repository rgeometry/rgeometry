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
