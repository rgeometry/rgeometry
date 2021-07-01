use criterion::{criterion_group, criterion_main, Criterion};
use rgeometry::data::PolygonConvex;

pub fn criterion_benchmark(c: &mut Criterion) {
  let mut rng = rand::thread_rng();
  c.bench_function("PolygonConvex::random(20)", |b| {
    b.iter(|| PolygonConvex::<i64>::random(20, &mut rng))
  });
  c.bench_function("PolygonConvex::random(1000)", |b| {
    b.iter(|| PolygonConvex::<i64>::random(1000, &mut rng))
  });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
