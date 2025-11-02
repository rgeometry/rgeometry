pub trait Intersects<T = Self> {
  type Result;
  fn intersect(self, other: T) -> Option<Self::Result>;
}
