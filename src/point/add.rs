// use std::mem::MaybeUninit;
use std::ops::Add;
// use std::ops::Index;

use crate::array::raw_arr_zipwith;
use crate::point::Point;
use crate::vector::Vector;

// &point + &vector = vector
impl<'a, 'b, T, const N: usize> Add<&'a Vector<T, N>> for &'b Point<T, N>
where
  T: Add<T, Output = T> + Clone,
{
  type Output = Point<T, N>;

  fn add(self: &'b Point<T, N>, other: &'a Vector<T, N>) -> Self::Output {
    let arr: [T; N] = raw_arr_zipwith(&self.0, &other.0, |a, b| a.clone() + b.clone());
    Point(arr)
  }
}

// point + vector = point
// impl<'a, 'b, T, const N: usize> Add<Vector<&'a T, N>> for &'b Point<T, N>
// where
//   T: Add<T, Output = T> + Clone,
// {
//   type Output = Point<T, N>;

//   fn add(self: &'b Point<T, N>, other: Vector<&'a T, N>) -> Self::Output {
//     let arr: [T; N] = unsafe {
//       let mut arr = MaybeUninit::uninit();
//       for i in 0..N {
//         (arr.as_mut_ptr() as *mut T)
//           .add(i)
//           .write(self.0.index(i).clone() + (*other.0.index(i)).clone());
//       }
//       arr.assume_init()
//     };
//     Point(arr)
//   }
// }
