use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use rgeometry::algorithms::convex_hull;
use rgeometry::data::*;

pub fn gen_arr<R, const N: usize>(rng: &mut R) -> [Point<i32, 2>; N]
where
  R: Rng + ?Sized,
{
  let mut arr = [Point::new([0, 0]); N];
  for pt in arr.iter_mut() {
    *pt = rng.gen();
  }
  arr
}

pub fn criterion_benchmark(c: &mut Criterion) {
  let mut rng = rand::thread_rng();
  let p1: [Point<i32, 2>; 10] = gen_arr(&mut rng);
  let p2: [Point<i32, 2>; 100] = gen_arr(&mut rng);
  let p3: [Point<i32, 2>; 1000] = gen_arr(&mut rng);
  let p4: [Point<i32, 2>; 10000] = gen_arr(&mut rng);
  let p5: [Point<i32, 2>; 100000] = gen_arr(&mut rng);
  c.bench_function("convex_hull(1e1)", |b| {
    b.iter(|| convex_hull(Vec::from(p1)))
  });
  c.bench_function("convex_hull(1e2)", |b| {
    b.iter(|| convex_hull(Vec::from(p2)))
  });
  c.bench_function("convex_hull(1e3)", |b| {
    b.iter(|| convex_hull(Vec::from(p3)))
  });
  c.bench_function("convex_hull(1e4)", |b| {
    b.iter(|| convex_hull(Vec::from(p4)))
  });
  c.bench_function("convex_hull(1e5)", |b| {
    b.iter(|| convex_hull(Vec::from(p5)))
  });
}

// 33      => 5
// 664     => 3.9
// 9965    => 3.813
// 132877  => 5.411
// 1660964 => 5.237

// 10*ln 10 * x = 0.165us
// 100*ln 100 * x = 2.6us
// 1000*ln 1000 * x = 38us
// 10000*ln 10000 * x = 719us
// 100000*ln 100000 * x = 8.7ms

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
