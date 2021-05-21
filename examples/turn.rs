use rgeometry::*;

fn main() {
  dbg!(Point::new([0, 0]).orientation(&Point::new([1, 1]), &Point::new([2, 2])));
  dbg!(Point::new([0, 0]).orientation(&Point::new([0, 1]), &Point::new([2, 2])));
  dbg!(Point::new([0, 0]).orientation(&Point::new([0, 1]), &Point::new([-2, 2])));
  dbg!(Point::new([0, 0]).orientation(&Point::new([0, 0]), &Point::new([0, 0])));
  dbg!(Point::new([0.1, 0.1]).orientation(&Point::new([0.3, 0.3]), &Point::new([0.9, 0.9])));
  // dbg!(raw_arr_turn(&[0.1, 0.1], &[0.3, 0.3], &[0.9, 0.9]));
}
