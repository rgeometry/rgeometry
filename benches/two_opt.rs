use criterion::{Criterion, criterion_group, criterion_main};
use rgeometry::algorithms::polygonization::two_opt_moves;
use rgeometry::data::*;

use num::BigInt;
use num::BigRational;

use rand::Rng;
use rand::SeedableRng;
use rand::distr::StandardUniform;

const SET_SIZE: usize = 100;

pub fn criterion_benchmark(c: &mut Criterion) {
  let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
  // M1:    2.1ms, 1x
  // 3950X: 2.6ms, 1x
  c.bench_function("two_opt_moves::<isize>", |b| {
    b.iter(|| {
      let mut pts = Vec::new();
      while pts.len() < SET_SIZE {
        let pt: Point<i64> = rng.sample(StandardUniform);
        pts.push(pt)
      }
      two_opt_moves(pts, &mut rng)
    })
  });

  let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
  c.bench_function("two_opt_moves::<f64>", |b| {
    b.iter(|| {
      let mut pts = Vec::new();
      while pts.len() < SET_SIZE {
        let pt: Point<f64> = rng.sample::<Point<f64, 2>, _>(StandardUniform);
        pts.push(pt)
      }
      two_opt_moves(pts, &mut rng)
    })
  });

  let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
  // M1:    56ms, 26x
  // 3950X: 39ms, 15x
  c.bench_function("two_opt_moves::<BigInt>", |b| {
    b.iter(|| {
      let mut pts: Vec<Point<BigInt>> = Vec::new();
      while pts.len() < SET_SIZE {
        let pt: Point<i64, 2> = rng.sample(StandardUniform);
        pts.push(pt.cast())
      }
      two_opt_moves(pts, &mut rng)
    })
  });

  // M1:    1.66 s, 783x
  // 3950X: 1.63 s, 635x
  let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
  c.bench_function("two_opt_moves::<BigRational>", |b| {
    b.iter(|| {
      let mut pts: Vec<Point<BigRational>> = Vec::new();
      while pts.len() < SET_SIZE {
        let pt: Point<i64> = rng.sample(StandardUniform);
        let pt: Point<BigInt> = pt.cast();
        pts.push(pt.cast())
      }
      two_opt_moves(pts, &mut rng)
    })
  });

  #[cfg(feature = "rug")]
  {
    // M1:    54ms
    // 3950X: 29ms
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    c.bench_function("two_opt_moves::<rug::Integer>", |b| {
      b.iter(|| {
        let mut pts: Vec<Point<rug::Integer, 2>> = Vec::new();
        while pts.len() < SET_SIZE {
          let pt: Point<i64, 2> = rng.sample(StandardUniform);
          // let pt: Point<BigInt, 2> = pt.cast();
          pts.push(pt.cast())
        }
        two_opt_moves(pts, &mut rng)
      })
    });
  }
  // c.bench_function("two_opt_moves::<BigInt>", |b| {
  //   b.iter(|| PolygonConvex::<i64>::random(1000, &mut rng))
  // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
