use rgeometry::algorithms::polygonization::*;
use rgeometry::data::*;
use rgeometry::*;

use rand::SeedableRng;

fn main() -> Result<(), Error> {
  let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
  // let pts = vec![
  //   Point::new([0, 0]),
  //   Point::new([1, 0]),
  //   Point::new([1, 1]),
  //   Point::new([0, 1]),
  //   Point::new([2, 3]),
  // ];
  // two_opt_moves(pts, &mut rng)?;

  // let pts = vec![
  //   Point::new([0, 0]),
  //   Point::new([1, 0]),
  //   Point::new([2, 0]),
  //   Point::new([3, 0]),
  //   Point::new([4, 0]),
  // ];
  // assert!(two_opt_moves(pts, &mut rng).is_err());

  let pts = vec![
    Point::new([0, 0]),
    Point::new([4, 0]),
    Point::new([3, 0]),
    Point::new([1, 0]),
    Point::new([2, 0]),
    Point::new([5, 1]),
  ];
  two_opt_moves(pts, &mut rng)?;
  Ok(())
}
