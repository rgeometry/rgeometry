#[cfg(test)]
mod tests {
  use crate::*;
  #[test]
  fn test_turns() {
    assert_eq!(
      Point([0, 0]).turn(&Point([1, 1]), &Point([2, 2])),
      Turn::CoLinear
    );
    assert_eq!(
      Point([0.0, 0.0]).turn(&Point([1.0, 1.0]), &Point([2.0, 2.0])),
      Turn::CoLinear
    );

    assert_eq!(
      Point([0, 0]).turn(&Point([0, 1]), &Point([2, 2])),
      Turn::ClockWise
    );
    assert_eq!(
      Point([0.0, 0.0]).turn(&Point([0.0, 1.0]), &Point([2.0, 2.0])),
      Turn::ClockWise
    );

    assert_eq!(
      Point([0, 0]).turn(&Point([0, 1]), &Point([-2, 2])),
      Turn::CounterClockWise
    );
    assert_eq!(
      Point([0, 0]).turn(&Point([0, 0]), &Point([0, 0])),
      Turn::CoLinear
    );
  }
}
