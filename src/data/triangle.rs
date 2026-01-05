use super::{Point, PointLocation};
use crate::{Error, Orientation, PolygonScalar, TotalOrd};
use claims::debug_assert_ok;
use num_traits::*;
use rand::Rng;
use rand::distr::uniform::SampleUniform;

// FIXME: Support n-dimensional triangles?
#[derive(Debug, Clone)]
pub struct Triangle<T>([Point<T, 2>; 3]);

impl<T> Triangle<T>
where
  T: PolygonScalar,
{
  pub fn new(pts: [Point<T, 2>; 3]) -> Result<Triangle<T>, Error> {
    let triangle = Triangle(pts);
    triangle.validate()?;
    Ok(triangle)
  }

  pub fn new_ccw(mut pts: [Point<T, 2>; 3]) -> Triangle<T> {
    match Point::orient(&pts[0], &pts[1], &pts[2]) {
      Orientation::CounterClockWise => Triangle(pts),
      Orientation::ClockWise => {
        pts.swap(0, 2);
        Triangle(pts)
      }
      Orientation::CoLinear => panic!("Cannot orient colinear points."),
    }
  }

  pub fn validate(&self) -> Result<(), Error> {
    self.view().validate()
  }

  // O(1)
  pub fn locate(&self, pt: &Point<T, 2>) -> PointLocation {
    self.view().locate(pt)
  }

  pub fn view(&self) -> TriangleView<'_, T> {
    TriangleView([&self.0[0], &self.0[1], &self.0[2]])
  }
}

pub struct TriangleView<'a, T>([&'a Point<T, 2>; 3]);

impl<'a, T> TriangleView<'a, T>
where
  T: PolygonScalar,
{
  // O(1)
  pub fn new(pts: [&'a Point<T, 2>; 3]) -> Result<TriangleView<'a, T>, Error> {
    let triangle = TriangleView(pts);
    triangle.validate()?;
    Ok(triangle)
  }

  pub fn new_ccw(pts: [&'a Point<T, 2>; 3]) -> TriangleView<'a, T> {
    match Point::orient(pts[0], pts[1], pts[2]) {
      Orientation::CounterClockWise => TriangleView(pts),
      Orientation::ClockWise => TriangleView([pts[2], pts[1], pts[0]]),
      Orientation::CoLinear => panic!("Cannot orient colinear points."),
    }
  }

  pub fn new_unchecked(pts: [&'a Point<T, 2>; 3]) -> TriangleView<'a, T> {
    TriangleView(pts)
  }

  // O(1)
  pub fn validate(&self) -> Result<(), Error> {
    if self.orientation() != Orientation::CounterClockWise {
      Err(Error::ClockWiseViolation)
    } else {
      Ok(())
    }
  }

  pub fn orientation(&self) -> Orientation {
    let arr = &self.0;
    Point::orient(arr[0], arr[1], arr[2])
  }

  // O(1)
  pub fn locate(&self, pt: &Point<T, 2>) -> PointLocation {
    use Orientation::*;
    debug_assert_ok!(self.validate());
    let [a, b, c] = self.0;
    let ab = Point::orient(a, b, pt);
    let bc = Point::orient(b, c, pt);
    let ca = Point::orient(c, a, pt);
    if ab == ClockWise || bc == ClockWise || ca == ClockWise {
      PointLocation::Outside
    } else if ab == CoLinear || bc == CoLinear || ca == CoLinear {
      PointLocation::OnBoundary
    } else {
      PointLocation::Inside
    }
  }

  pub fn signed_area<F>(&self) -> F
  where
    T: PolygonScalar + Into<F>,
    F: NumOps<F, F> + FromPrimitive + Clone,
  {
    self.signed_area_2x::<F>() / F::from_usize(2).unwrap()
  }

  pub fn signed_area_2x<F>(&self) -> F
  where
    T: PolygonScalar + Into<F>,
    F: NumOps<F, F> + Clone,
  {
    let [a, b, c] = self.0;
    let ax = a.x_coord().clone().into();
    let ay = a.y_coord().clone().into();
    let bx = b.x_coord().clone().into();
    let by = b.y_coord().clone().into();
    let cx = c.x_coord().clone().into();
    let cy = c.y_coord().clone().into();
    ax.clone() * by.clone() - bx.clone() * ay.clone() + bx * cy.clone() - cx.clone() * by + cx * ay
      - ax * cy
    // x1*y2 - x2*y1 +
    // x2*y3 - x3*y2 +
    // x3*y1 - x1*y3
  }

  pub fn bounding_box(&self) -> (Point<T, 2>, Point<T, 2>) {
    let min_x = self.0[0]
      .x_coord()
      .total_min(self.0[1].x_coord())
      .total_min(self.0[2].x_coord())
      .clone();
    let max_x = self.0[0]
      .x_coord()
      .total_max(self.0[1].x_coord())
      .total_max(self.0[2].x_coord())
      .clone();
    let min_y = self.0[0]
      .y_coord()
      .total_min(self.0[1].y_coord())
      .total_min(self.0[2].y_coord())
      .clone();
    let max_y = self.0[0]
      .y_coord()
      .total_max(self.0[1].y_coord())
      .total_max(self.0[2].y_coord())
      .clone();
    (Point::new([min_x, min_y]), Point::new([max_x, max_y]))
  }

  /// Sample a random point inside this triangle using rejection sampling.
  ///
  /// This method generates random points within the triangle's bounding box
  /// and returns the first one that falls inside the triangle.
  ///
  /// # Panics
  ///
  /// Panics if no valid point is found after 10,000 attempts. This can happen
  /// if the triangle is degenerate (has zero or near-zero area).
  pub fn rejection_sampling<R>(&self, rng: &mut R) -> Point<T, 2>
  where
    R: Rng + ?Sized,
    T: SampleUniform,
  {
    const MAX_ITERATIONS: u32 = 10_000;
    let (min, max) = self.bounding_box();
    for _ in 0..MAX_ITERATIONS {
      let [min_x, min_y] = min.array.clone();
      let [max_x, max_y] = max.array.clone();
      let x = rng.random_range(min_x..=max_x);
      let y = rng.random_range(min_y..=max_y);
      let pt = Point::new([x, y]);
      if self.locate(&pt) != PointLocation::Outside {
        return pt;
      }
    }
    panic!(
      "rejection_sampling failed after {} iterations - triangle may be degenerate",
      MAX_ITERATIONS
    );
  }

  // sampleTriangle :: (RandomGen g, Random r, Fractional r, Ord r) => Triangle 2 p r -> Rand g (Point 2 r)
  // sampleTriangle (Triangle v1 v2 v3) = do
  //   a' <- getRandomR (0, 1)
  //   b' <- getRandomR (0, 1)
  //   let (a, b) = if a' + b' > 1 then (1 - a', 1 - b') else (a', b')
  //   return $ v1^.core .+^ a*^u .+^ b*^v
  //   where
  //     u = v2^.core .-. v1^.core
  //     v = v3^.core .-. v1^.core
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::Error;
  use proptest::prelude::*;
  use rand::SeedableRng;

  fn point<T: PolygonScalar>(x: T, y: T) -> Point<T, 2> {
    Point::new([x, y])
  }

  #[test]
  fn triangle_new_valid_ccw() {
    let pts = [point(0i32, 0), point(2, 0), point(1, 2)];
    let result = Triangle::new(pts);
    assert!(result.is_ok());
  }

  #[test]
  fn triangle_new_valid_ccw_f64() {
    let pts = [point(0.0f64, 0.0), point(2.0, 0.0), point(1.0, 2.0)];
    let result = Triangle::new(pts);
    assert!(result.is_ok());
  }

  #[test]
  fn triangle_new_clockwise_rejects() {
    let pts = [point(0i32, 0), point(1, 2), point(2, 0)];
    let result = Triangle::new(pts);
    assert!(matches!(result, Err(Error::ClockWiseViolation)));
  }

  #[test]
  fn triangle_new_colinear_rejects() {
    let pts = [point(0i32, 0), point(1, 1), point(2, 2)];
    let result = Triangle::new(pts);
    match &result {
      Ok(_) => panic!("Expected error, got Ok"),
      Err(e) => {
        assert!(matches!(e, Error::ClockWiseViolation | Error::CoLinearViolation),
          "Expected CoLinearViolation or ClockWiseViolation, got: {:?}", e);
      }
    }
  }

  #[test]
  fn triangle_new_ccw_orientation() {
    let pts = [point(0i32, 0), point(2, 0), point(1, 2)];
    let tri = Triangle::new_ccw(pts);
    assert_eq!(tri.view().orientation(), Orientation::CounterClockWise);
  }

  #[test]
  fn triangle_new_ccw_flips_clockwise() {
    let pts = [point(0i32, 0), point(1, 2), point(2, 0)];
    let tri = Triangle::new_ccw(pts);
    assert_eq!(tri.view().orientation(), Orientation::CounterClockWise);
  }

  #[test]
  #[should_panic(expected = "Cannot orient colinear points")]
  fn triangle_new_ccw_panics_on_colinear() {
    let pts = [point(0i32, 0), point(1, 1), point(2, 2)];
    let _tri = Triangle::new_ccw(pts);
  }

  #[test]
  fn triangle_view_new_valid() {
    let p0 = point(0i32, 0);
    let p1 = point(2, 0);
    let p2 = point(1, 2);
    let result = TriangleView::new([&p0, &p1, &p2]);
    assert!(result.is_ok());
  }

  #[test]
  fn triangle_view_new_clockwise_rejects() {
    let p0 = point(0i32, 0);
    let p1 = point(1, 2);
    let p2 = point(2, 0);
    let result = TriangleView::new([&p0, &p1, &p2]);
    assert!(matches!(result, Err(Error::ClockWiseViolation)));
  }

  #[test]
  fn triangle_view_new_colinear_rejects() {
    let p0 = point(0i32, 0);
    let p1 = point(1, 1);
    let p2 = point(2, 2);
    let result = TriangleView::new([&p0, &p1, &p2]);
    match &result {
      Ok(_) => panic!("Expected error, got Ok"),
      Err(e) => {
        assert!(matches!(e, Error::ClockWiseViolation | Error::CoLinearViolation),
          "Expected CoLinearViolation or ClockWiseViolation, got: {:?}", e);
      }
    }
  }

  #[test]
  fn triangle_view_new_ccw_orientation() {
    let p0 = point(0i32, 0);
    let p1 = point(2, 0);
    let p2 = point(1, 2);
    let view = TriangleView::new_ccw([&p0, &p1, &p2]);
    assert_eq!(view.orientation(), Orientation::CounterClockWise);
  }

  #[test]
  fn triangle_view_new_ccw_flips_clockwise() {
    let p0 = point(0i32, 0);
    let p1 = point(1, 2);
    let p2 = point(2, 0);
    let view = TriangleView::new_ccw([&p0, &p1, &p2]);
    assert_eq!(view.orientation(), Orientation::CounterClockWise);
  }

  #[test]
  fn triangle_view_locate_consistency() {
    let pts = [point(0i32, 0), point(2, 0), point(1, 2)];
    let tri = Triangle::new(pts).unwrap();
    let view = tri.view();

    let test_pt = point(1, 1);
    assert_eq!(tri.locate(&test_pt), view.locate(&test_pt));
  }

  #[test]
  fn triangle_view_validate() {
    let pts = [point(0i32, 0), point(2, 0), point(1, 2)];
    let tri = Triangle::new(pts).unwrap();
    assert!(tri.validate().is_ok());
  }

  #[test]
  fn triangle_view_validate_clockwise() {
    let pts = [point(0i32, 0), point(1, 2), point(2, 0)];
    let tri = Triangle::new_ccw(pts);
    let result = tri.validate();
    assert!(result.is_ok());
  }

  fn arb_point_i32() -> impl Strategy<Value = Point<i32, 2>> {
    (-100..=100i32, -100..=100i32).prop_map(|(x, y)| point(x, y))
  }

  fn arb_triangle_i32() -> impl Strategy<Value = [Point<i32, 2>; 3]> {
    prop::array::uniform3(arb_point_i32())
  }

  proptest! {
    #[test]
    fn triangle_signed_area_positive_for_ccw(tri in arb_triangle_i32()) {
      if let Ok(tri) = Triangle::new(tri) {
        let area: i32 = tri.view().signed_area_2x();
        prop_assert!(area > 0, "CCW triangles should have positive area");
      }
    }

    #[test]
    fn triangle_bounding_box_contains_vertices(tri in arb_triangle_i32()) {
      if let Ok(tri) = Triangle::new(tri) {
        let (min, max) = tri.view().bounding_box();
        for pt in &tri.0 {
          prop_assert!(
            pt.x_coord() >= min.x_coord() && pt.x_coord() <= max.x_coord() &&
            pt.y_coord() >= min.y_coord() && pt.y_coord() <= max.y_coord(),
            "bounding box should contain all vertices"
          );
        }
      }
    }

    #[test]
    fn triangle_view_new_unchecked_no_validation(tri in arb_triangle_i32()) {
      let view = TriangleView::new_unchecked([&tri[0], &tri[1], &tri[2]]);
      prop_assert_eq!(view.0, [&tri[0], &tri[1], &tri[2]]);
    }
  }

  #[test]
  fn triangle_new_with_duplicate_points() {
    let pts = [point(0i32, 0), point(0, 0), point(1, 1)];
    let result = Triangle::new(pts);
    assert!(result.is_err());
  }

  #[test]
  fn triangle_new_with_same_point_repeated() {
    let pts = [point(0i32, 0), point(1, 1), point(0, 0)];
    let result = Triangle::new(pts);
    assert!(result.is_err());
  }

  #[test]
  fn triangle_locate_center_of_mass() {
    let pts = [point(0i32, 0), point(2, 0), point(1, 2)];
    let tri = Triangle::new(pts).unwrap();
    let centroid = point(1, 2i32 / 3);
    assert!(matches!(tri.locate(&centroid), PointLocation::Inside | PointLocation::OnBoundary));
  }

  #[test]
  fn triangle_rejection_sampling() {
    let pts = [point(0i32, 0), point(2, 0), point(1, 2)];
    let tri = Triangle::new(pts).unwrap();
    let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
    let sampled = tri.view().rejection_sampling(&mut rng);
    assert!(matches!(tri.locate(&sampled), PointLocation::Inside | PointLocation::OnBoundary));
  }


  #[test]
  fn triangle_signed_area_with_f64() {
    let pts = [point(0.0f64, 0.0), point(2.0, 0.0), point(1.0, 2.0)];
    let tri = Triangle::new(pts).unwrap();
    let area: f64 = tri.view().signed_area();
    assert_eq!(area, 2.0);
  }

  #[test]
  fn triangle_signed_area_with_i64() {
    let pts = [point(0i64, 0), point(4, 0), point(2, 3)];
    let tri = Triangle::new(pts).unwrap();
    let area: i64 = tri.view().signed_area_2x();
    assert_eq!(area, 12);
  }
}
