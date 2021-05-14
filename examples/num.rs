// use rand::Rng;
use num_rational::BigRational;
use num_traits::Num;
use rand::distributions::Standard;
use rand::Rng;
use rgeometry::*;
use std::ops::Mul;
use std::ops::Neg;
use std::ops::Sub;

// pub fn test_sub<T>(p: &Point<T, 2>, q: &Point<T, 2>) -> Vector<T, 2>
// where
//   T: Num + Clone + PartialOrd,
//   for<'a> &'a T: Sub<Output = T> + Mul<Output = T> + Neg<Output = T>,
// {
//   p - q
// }

fn main() {
  let a = Point([1, 2]);
  let b = Vector([3, 4]);
  let c = &a + &b;
  println!("Output: {:?}", c);
}
