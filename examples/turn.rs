use ordered_float::NotNan;
use rgeometry::data::*;

fn main() {
  let n01 = NotNan::new(0.1).unwrap();
  let n03 = NotNan::new(0.3).unwrap();
  let n09 = NotNan::new(0.9).unwrap();
  dbg!(Point::new([0, 0]).orientation(&Point::new([1, 1]), &Point::new([2, 2])));
  dbg!(Point::new([0, 0]).orientation(&Point::new([0, 1]), &Point::new([2, 2])));
  dbg!(Point::new([0, 0]).orientation(&Point::new([0, 1]), &Point::new([-2, 2])));
  dbg!(Point::new([0, 0]).orientation(&Point::new([0, 0]), &Point::new([0, 0])));
  dbg!(Point::new([n01, n01]).orientation(&Point::new([n03, n03]), &Point::new([n09, n09])));
  // dbg!(raw_arr_turn(&[0.1, 0.1], &[0.3, 0.3], &[0.9, 0.9]));
}
