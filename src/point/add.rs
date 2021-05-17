use crate::point::Point;
use crate::vector::Vector;
use array_init::array_init;
use std::ops::Add;
use std::ops::AddAssign;
use std::ops::Index;

// &point + &vector = point
impl<'a, 'b, T, const N: usize> Add<&'a Vector<T, N>> for &'b Point<T, N>
where
  for<'c> &'c T: Add<&'c T, Output = T>,
{
  type Output = Point<T, N>;

  fn add(self: &'b Point<T, N>, other: &'a Vector<T, N>) -> Self::Output {
    Point {
      array: array_init(|i| self.array.index(i) + other.0.index(i)),
    }
  }
}

// point + vector = point
impl<T, const N: usize> Add<Vector<T, N>> for Point<T, N>
where
  for<'a> &'a T: Add<&'a T, Output = T>,
{
  type Output = Point<T, N>;

  fn add(self: Point<T, N>, other: Vector<T, N>) -> Self::Output {
    Point {
      array: array_init(|i| self.array.index(i) + other.0.index(i)),
    }
  }
}

// point + &vector = point
impl<T, const N: usize> Add<&Vector<T, N>> for Point<T, N>
where
  for<'a> &'a T: Add<&'a T, Output = T>,
{
  type Output = Point<T, N>;

  fn add(self: Point<T, N>, other: &Vector<T, N>) -> Self::Output {
    Point {
      array: array_init(|i| self.array.index(i) + other.0.index(i)),
    }
  }
}

// point += &vector
impl<T, const N: usize> AddAssign<&Vector<T, N>> for Point<T, N>
where
  for<'a> T: AddAssign<&'a T>,
{
  fn add_assign(&mut self, other: &Vector<T, N>) {
    for i in 0..N {
      self.array[i] += other.0.index(i)
    }
  }
}

// point += vector
impl<T, const N: usize> AddAssign<Vector<T, N>> for Point<T, N>
where
  for<'a> T: AddAssign<&'a T>,
{
  fn add_assign(&mut self, other: Vector<T, N>) {
    for i in 0..N {
      self.array[i] += other.0.index(i)
    }
  }
}

// // point + point = point
// impl<T, const N: usize> Add<Point<T, N>> for Point<T, N>
// where
//   T: Add<T, Output = T> + Clone,
// {
//   type Output = Point<T, N>;

//   fn add(self: Point<T, N>, other: Point<T, N>) -> Self::Output {
//     let arr: [T; N] = raw_arr_zipwith(&self.0, &other.0, |a, b| a.clone() + b.clone());
//     Point(arr)
//   }
// }

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
