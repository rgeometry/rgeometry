use rgeometry::*;

fn main() {
  dbg!(Point([0, 0]).turn(&Point([1, 1]), &Point([2, 2])));
  dbg!(Point([0, 0]).turn(&Point([0, 1]), &Point([2, 2])));
  dbg!(Point([0, 0]).turn(&Point([0, 1]), &Point([-2, 2])));
  dbg!(Point([0, 0]).turn(&Point([0, 0]), &Point([0, 0])));
  dbg!(Point([0.1, 0.1]).turn(&Point([0.3, 0.3]), &Point([0.9, 0.9])));
  // dbg!(raw_arr_turn(&[0.1, 0.1], &[0.3, 0.3], &[0.9, 0.9]));
}
