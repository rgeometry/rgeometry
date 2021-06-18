pub trait Intersects<T = Self> {
  type Result;
  fn intersect(self, other: T) -> Option<Self::Result>;
}

// // impl<T, U> Intersects<U> for T
// // where
// //   U: Intersects<T>,
// // {
// //   type Result = U::Result;
// //   fn intersect(&self, other: &U) -> Self::Result {
// //     other.intersect(self)
// //   }
// // }

// struct Circle();

// type Result<A, B> = <B as Intersects<A>>::Result;

// pub enum CircleCircle {
//   Touching,
//   Overlapping,
//   Identical,
// }

// impl Intersects<Circle> for Circle {
//   type Result = CircleCircle;
//   fn intersect(&self, _other: &Circle) -> Option<Self::Result> {
//     todo!()
//   }
// }

// fn test() {
//   let x: Result<Circle, Circle> = CircleCircle::Touching;
// }
