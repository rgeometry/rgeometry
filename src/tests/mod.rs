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
      let p = ConvexPolygon::random(10, 1000, &mut rng);
      Just(p)
    }
  }

  proptest! {
    #[test]
    fn prop1(poly: ConvexPolygon<BigRational>) {
      prop_assert!(poly.validate().is_ok())
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
