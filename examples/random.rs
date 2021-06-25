#![allow(unused_imports)]
// use rand::Rng;
use num_bigint::BigInt;
use num_rational::BigRational;
use rand::distributions::Standard;
use rand::Rng;
use rgeometry::data::*;
use rgeometry::*;

use rand::SeedableRng;

fn main() {
  // let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
  let mut rng = rand::thread_rng();
  // dbg!(random_between(10, 100, &mut rng));
  // dbg!(random_between(10, 100, &mut rng)
  //   .iter()
  //   .fold(0, |a, b| a + b));
  // dbg!(random_between(10, 100, &mut rng).iter().sum::<BigInt>());
  // dbg!(random_between_zero(10, 100, &mut rng));
  // dbg!(random_between_zero(10, 100, &mut rng)
  //   .iter()
  //   .sum::<BigInt>());

  let p: Polygon<BigRational> = PolygonConvex::random(20, 1_000_000_000, &mut rng).into();
  dbg!(p.orientation());
  // for view in p.iter_boundary_edges() {
  //   dbg!(view);
  // }
  // dbg!(&p);
  // dbg!(p.centroid());
  // let val: ConvexPolygon<BigRational> = rng.sample(Standard);
  // dbg!(val);
}
