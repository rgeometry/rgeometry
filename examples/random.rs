use rand::Rng;
use rgeometry::*;

fn main() {
  let mut rng = rand::thread_rng();
  dbg!(random_between(10, 100, &mut rng));
  dbg!(random_between(10, 100, &mut rng)
    .iter()
    .fold(0, |a, b| a + b));
  dbg!(random_between(10, 100, &mut rng).iter().sum::<usize>());
  dbg!(random_between_zero(10, 100, &mut rng));
  dbg!(random_between_zero(10, 100, &mut rng).iter().sum::<isize>());
}
