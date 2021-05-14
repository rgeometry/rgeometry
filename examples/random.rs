// use rand::Rng;
use num_bigint::BigInt;
use num_rational::BigRational;
use rand::distributions::Standard;
use rand::Rng;
use rgeometry::*;

fn main() {
  let mut rng = rand::thread_rng();
  dbg!(random_between(10, 100, &mut rng));
  dbg!(random_between(10, 100, &mut rng)
    .iter()
    .fold(0, |a, b| a + b));
  dbg!(random_between(10, 100, &mut rng).iter().sum::<BigInt>());
  dbg!(random_between_zero(10, 100, &mut rng));
  dbg!(random_between_zero(10, 100, &mut rng)
    .iter()
    .sum::<BigInt>());

  // dbg!(random(10, 1000, &mut rng));
  let val: ConvexPolygon<BigRational> = rng.sample(Standard);
  dbg!(val);
}
