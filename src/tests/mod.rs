#[cfg(test)]
mod tests {
  use crate::*;
  #[test]
  fn test_turns() {
    assert_eq!(
      Point::new([0, 0]).turn(&Point::new([1, 1]), &Point::new([2, 2])),
      Turn::CoLinear
    );
    assert_eq!(
      Point::new([0.0, 0.0]).turn(&Point::new([1.0, 1.0]), &Point::new([2.0, 2.0])),
      Turn::CoLinear
    );

    assert_eq!(
      Point::new([0, 0]).turn(&Point::new([0, 1]), &Point::new([2, 2])),
      Turn::ClockWise
    );
    assert_eq!(
      Point::new([0.0, 0.0]).turn(&Point::new([0.0, 1.0]), &Point::new([2.0, 2.0])),
      Turn::ClockWise
    );

    assert_eq!(
      Point::new([0, 0]).turn(&Point::new([0, 1]), &Point::new([-2, 2])),
      Turn::CounterClockWise
    );
    assert_eq!(
      Point::new([0, 0]).turn(&Point::new([0, 0]), &Point::new([0, 0])),
      Turn::CoLinear
    );
  }
}
