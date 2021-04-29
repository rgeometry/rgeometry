use rgeometry::point::*;
use rgeometry::*;

fn main() {
  dbg!(turn(&Point([0, 0]), &Point([1, 1]), &Point([2, 2])));
  dbg!(turn(&Point([0, 0]), &Point([0, 1]), &Point([2, 2])));
  dbg!(turn(&Point([0, 0]), &Point([0, 1]), &Point([-2, 2])));
  dbg!(turn(&Point([0, 0]), &Point([0, 0]), &Point([0, 0])));
  dbg!(turn(
    &Point([0.1, 0.1]),
    &Point([0.3, 0.3]),
    &Point([0.9, 0.9])
  ));
  dbg!(raw_arr_turn(&[0.1, 0.1], &[0.3, 0.3], &[0.9, 0.9]));
}
