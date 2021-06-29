/*
use array_init::array_init;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use num_bigint::BigInt;
use ordered_float::*;
use rand::Rng;
use rgeometry::algorithms::convex_hull;
use rgeometry::data::*;
use std::convert::*;

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
pub fn gen_arr_f64<R, const N: usize>(rng: &mut R) -> [Point<OrderedFloat<f64>, 2>; N]
where
  R: Rng + ?Sized,
{
  let mut arr = [Point::new([OrderedFloat(0.), OrderedFloat(0.)]); N];
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
  let p4: [Point<OrderedFloat<f64>, 2>; 10000] = gen_arr_f64(&mut rng);
  let p5: [Point<i32, 2>; 100000] = gen_arr(&mut rng);
  // c.bench_function("convex_hull(1e1)", |b| {
  //   b.iter(|| convex_hull(Vec::from(p1)))
  // });
  // c.bench_function("convex_hull(1e2)", |b| {
  //   b.iter(|| convex_hull(Vec::from(p2)))
  // });
  // c.bench_function("convex_hull(1e3)", |b| {
  //   b.iter(|| convex_hull(Vec::from(p3)))
  // });
  c.bench_function("convex_hull(1e4)", |b| {
    b.iter(|| convex_hull(Vec::from(p4)))
  });
  c.bench_function("convex_hull(1e4)", |b| {
    b.iter(|| {
      let mut vec = Vec::from(p4);
      vec.sort_unstable();
      vec.dedup();
      vec
    })
  });
  c.bench_function("convex_hull(1e4)", |b| b.iter(|| Vec::from(p4)));
  // c.bench_function("convex_hull(1e5)", |b| {
  //   b.iter(|| convex_hull(Vec::from(p5)))
  // });
  // let p: [Point<BigInt, 2>; 10_000] = array_init(|_i| {
  //   let tmp: Point<u64, 2> = rng.gen();
  //   tmp.cast(BigInt::from)
  // });
  // for &n in &[1_000, 10_000] {
  //   c.bench_function(&format!("convex_hull({})", n), |b| {
  //     b.iter_batched(
  //       || Vec::from(&p[0..n]),
  //       |inp| convex_hull(inp),
  //       BatchSize::LargeInput,
  //     )
  //   });
  // }
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
*/
fn main() {}
