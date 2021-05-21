#[cfg(test)]
mod tests {
  use crate::Orientation::*;
  use crate::*;

  #[test]
  fn test_turns() {
    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([1, 1]), &Point::new([2, 2])),
      CoLinear
    );
    assert_eq!(
      Point::new([0.0, 0.0]).orientation(&Point::new([1.0, 1.0]), &Point::new([2.0, 2.0])),
      CoLinear
    );

    assert_eq!(
      Point::new([0, 0]).orientation(&Point::new([0, 1]), &Point::new([2, 2])),
      ClockWise
    );
    assert_eq!(
      Point::new([0.0, 0.0]).orientation(&Point::new([0.0, 1.0]), &Point::new([2.0, 2.0])),
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
