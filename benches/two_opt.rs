use criterion::{criterion_group, criterion_main, Criterion};
use rgeometry::algorithms::polygonization::two_opt_moves;
use rgeometry::data::*;

use num::BigInt;
use num::BigRational;
use ordered_float::OrderedFloat;

use rand::distributions::Standard;
use rand::Rng;
use rand::SeedableRng;

const SET_SIZE: usize = 100;

pub fn criterion_benchmark(c: &mut Criterion) {
  let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
  // M1:    2.1ms, 1x
  // 3950X: 2.6ms, 1x
  c.bench_function("two_opt_moves::<isize>", |b| {
    b.iter(|| {
      let mut pts = Vec::new();
      while pts.len() < SET_SIZE {
        let pt: Point<isize> = rng.sample(Standard);
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
        let pt: Point<OrderedFloat<f64>> = rng.sample::<Point<f64, 2>, _>(Standard).map(OrderedFloat);
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
        let pt: Point<isize, 2> = rng.sample(Standard);
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
        let pt: Point<isize> = rng.sample(Standard);
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
          let pt: Point<isize, 2> = rng.sample(Standard);
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
