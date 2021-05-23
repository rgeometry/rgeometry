use super::Point;

#[derive(Debug, Clone, Copy)]
pub enum EndPoint<T> {
  Open(T),
  Closed(T),
}

impl<T> EndPoint<T> {
  pub fn inner(&self) -> &T {
    match self {
      EndPoint::Open(t) => &t,
      EndPoint::Closed(t) => &t,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct LineSegment<T, P, const N: usize>(
  pub EndPoint<(Point<T, N>, P)>,
  pub EndPoint<(Point<T, N>, P)>,
);

#[derive(Debug, Clone, Copy)]
pub struct LineSegmentView<'a, T, P, const N: usize>(
  pub EndPoint<(&'a Point<T, N>, &'a P)>,
  pub EndPoint<(&'a Point<T, N>, &'a P)>,
);
