#![allow(unused_imports)]
#![allow(unused_must_use)]
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::cast::FromPrimitive;
// use num_rational::BigRational;
// use num_traits::Num;
// use num_traits::Zero;
// use rand::distributions::Standard;
// use rand::Rng;
// use rgeometry::transformation::*;
use rgeometry::algorithms::convex_hull::graham_scan::convex_hull;
use rgeometry::data::*;
// use std::ops::Mul;
// use std::ops::Neg;
// use std::ops::Sub;

// pub fn test_sub<T>(p: &Point<T, 2>, q: &Point<T, 2>) -> Vector<T, 2>
// where
//   T: Num + Clone + PartialOrd,
//   for<'a> &'a T: Sub<Output = T> + Mul<Output = T> + Neg<Output = T>,
// {
//   p - q
// }

fn main() {
  // let pt = Point::new([1, 2]);
  // dbg!(pt.y_coord());
  // let t1 = Transform::translate(Vector([BigInt::from(1), BigInt::from(2), BigInt::from(3)]));
  // let t2 = Transform::scale(Vector([BigInt::from(-1), BigInt::from(1), BigInt::from(2)]));
  // let v = Vector([BigInt::from(4), BigInt::from(5), BigInt::from(6)]);
  // dbg!(&v);
  // dbg!(&t1 * &v);
  // dbg!(&t2 * &v);
  // dbg!((&t2 * &t1) * &v);
  let p: Vec<Point<i32, 2>> = vec![
    Point::new([0, 0]),
    Point::new([1, 0]),
    Point::new([2, 0]),
    Point::new([3, 3]),
    Point::new([0, 1]),
  ];
  // dbg!(p[0].turn(&p[1], &p[2]));
  let hull = convex_hull(p);
  dbg!(hull);
  // let p2 = p.cast(|v| BigRational::from_i32(v).unwrap());
  // dbg!(p2.centroid());
  // let a: Array2<BigRational> = unimplemented!();
  // let b = array![7, 8, 9];
  // // let c: Array2<i32> = arr1(&b);
  // dbg!(a.dot(&b));
  // let m = vec![vec![1, 2, 3], vec![4, 5, 6]];
  // let v = vec![7, 8, 9];
  // dbg!(mult_mv::<i32>(m, &v));
  // let p = Point([1, 2]);
  // // println!("p: {:?}", &p);
  // let t1 = Transform::translation(Vector([1, 1]));
  // let t2 = Transform::scaling(Vector([2, -1]));
  // // println!("t: {:?}", &t);
  // let tp = (t2 * t1) * p;
  // println!("tp: {:?}", &tp);
  // let a = Point([1, 2]);
  // let b = Vector([3, 4]);
  // let c = &a + &b;
  // println!("Output: {:?}", c);
  // let p: Point<f32, 3> = Point3::new(1., 2., 3.0);
  // let x: SVector<f32, 3> = Vector3::new(1.0, 2.0, 3.0);
  // let t1: Transform<f32, TAffine, 3> = Transform::identity();
  // let t2: Transform<f32, TAffine, 3> = t1 * Translation3::new(1.0, 2., 3.);
  // let t3: Transform<f32, TAffine, 3> = Transform::identity();
  // let t4: &OMatrix<f32, Const<4>, Const<4>> = t3.matrix();
  // let t5: Transform<f32, TAffine, 3> = Transform::from(t4);
  // let t6: SVector<i32, 3> = Vector3::new(1, 2, 3);
  // let t7: SVector<BigInt, 3> = Vector3::new(BigInt::from(2), BigInt::from(3), BigInt::from(4));
  // let t8 = Matrix3::from_diagonal_element(1);
  // let t9: Transform<f32, 3> = translation(x);
  // let t10 = scaling(x);
  // println!("N: {:?}", &t9);
  // println!("Output: {:?}", t9 * p);
}
