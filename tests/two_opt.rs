mod two_opt {
  use rgeometry::algorithms::polygonization::*;
  use rgeometry::data::*;
  use rgeometry::*;

  use rand::SeedableRng;

  #[test]
  fn clockwise() -> Result<(), Error> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    let pts = vec![Point::new([0, 0]), Point::new([0, 1]), Point::new([1, 0])];
    two_opt_moves(pts, &mut rng)?;
    Ok(())
  }

  #[test]
  fn clockwise_poly() -> Result<(), Error> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    let pts = vec![Point::new([0, 0]), Point::new([0, 1]), Point::new([1, 0])];
    two_opt_moves_poly(pts, &mut rng)?;
    Ok(())
  }

  #[test]
  fn linear() -> Result<(), Error> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    let pts = vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([2, 0]),
      Point::new([3, 0]),
      Point::new([4, 0]),
    ];
    assert!(two_opt_moves(pts, &mut rng).is_err());
    Ok(())
  }

  #[test]
  fn linear_poly() -> Result<(), Error> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    let pts = vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([2, 0]),
      Point::new([3, 0]),
      Point::new([4, 0]),
    ];
    assert!(two_opt_moves_poly(pts, &mut rng).is_err());
    Ok(())
  }

  #[test]
  fn near_linear() -> Result<(), Error> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    let pts = vec![
      Point::new([0, 0]),
      Point::new([4, 0]),
      Point::new([3, 0]),
      Point::new([1, 0]),
      Point::new([2, 0]),
      Point::new([5, 1]),
    ];
    two_opt_moves(pts, &mut rng)?;
    Ok(())
  }

  #[test]
  fn near_linear_poly() -> Result<(), Error> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    let pts = vec![
      Point::new([0, 0]),
      Point::new([4, 0]),
      Point::new([3, 0]),
      Point::new([1, 0]),
      Point::new([2, 0]),
      Point::new([5, 1]),
    ];
    two_opt_moves_poly(pts, &mut rng)?;
    Ok(())
  }

  #[test]
  fn basic() -> Result<(), Error> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    let pts = vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([1, 1]),
      Point::new([0, 1]),
      Point::new([2, 3]),
    ];
    two_opt_moves(pts, &mut rng)?;
    Ok(())
  }

  #[test]
  fn basic_poly() -> Result<(), Error> {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    let pts = vec![
      Point::new([0, 0]),
      Point::new([1, 0]),
      Point::new([1, 1]),
      Point::new([0, 1]),
      Point::new([2, 3]),
    ];
    two_opt_moves_poly(pts, &mut rng)?;
    Ok(())
  }
}
