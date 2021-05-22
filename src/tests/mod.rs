#[cfg(test)]
mod tests {
  use crate::Orientation::*;
  use crate::*;

  use proptest::prelude::*;
  use proptest::strategy::*;
  use proptest::test_runner::*;

  use ordered_float::NotNan;

  impl Arbitrary for ConvexPolygon<BigRational> {
    type Strategy = Just<ConvexPolygon<BigRational>>;
    type Parameters = ();
    fn arbitrary_with(_params: ()) -> Self::Strategy {
      Self::arbitrary()
    }
    fn arbitrary() -> Self::Strategy {
      let mut rng = rand::thread_rng();
      let n = rng.gen_range(3..=100);
      let max = rng.gen_range(n..=1_000_000_000);
      let p = ConvexPolygon::random(n, max, &mut rng);
      Just(p)
    }
  }

  proptest! {
    #[test]
    fn all_random_convex_polygons_are_valid(poly: ConvexPolygon<BigRational>) {
      prop_assert_eq!(poly.validate(), Ok(()))
    }

    #[test]
    fn sum_to_max(n in 1..1000, max in 0..1_000_000) {
      let mut rng = rand::thread_rng();
      let max = std::cmp::max(max, n);
      let vecs = random_between(n as usize, max as usize, &mut rng);
      prop_assert_eq!(vecs.iter().sum::<usize>(), max as usize)
    }

    #[test]
    fn random_between_zero_properties(n in 2..1000, max in 0..1_000_000) {
      let mut rng = rand::thread_rng();
      let max = std::cmp::max(max, n);
      let vecs = random_between_zero(n as usize, max as usize, &mut rng);
      prop_assert_eq!(vecs.iter().sum::<BigInt>(), BigInt::from(0));
      prop_assert!(vecs.iter().all(|v| !v.is_zero()));
      prop_assert_eq!(vecs.len(), n as usize);
    }

    #[test]
    fn sum_to_zero_vector(n in 2..1000, max in 0..1_000_000) {
      let mut rng = rand::thread_rng();
      let max = std::cmp::max(max, n);
      let vecs = random_vectors(n as usize, max as usize, &mut rng);
      prop_assert_eq!(vecs.into_iter().sum::<Vector<BigRational,2>>(), Vector::zero())
    }
  }

  fn n(f: f64) -> NotNan<f64> {
    NotNan::new(f).unwrap()
  }

  #[test]
  fn test_turns() {
    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([1, 1]), &Point::new([2, 2])),
      CoLinear
    );
    assert_eq!(
      Point::new([n(0.0), n(0.0)])
        .orientation(&Point::new([n(1.0), n(1.0)]), &Point::new([n(2.0), n(2.0)])),
      CoLinear
    );

    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([0, 1]), &Point::new([2, 2])),
      ClockWise
    );
    assert_eq!(
      Point::new([n(0.0), n(0.0)])
        .orientation(&Point::new([n(0.0), n(1.0)]), &Point::new([n(2.0), n(2.0)])),
      ClockWise
    );

    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([0, 1]), &Point::new([-2, 2])),
      CounterClockWise
    );
    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([0, 0]), &Point::new([0, 0])),
      CoLinear
    );
  }
}
