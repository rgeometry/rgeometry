use super::{Point, PointLocation};
use crate::{Error, Orientation, PolygonScalar, PolygonScalarRef};
use claim::debug_assert_ok;
use num_traits::*;
use rand::distributions::uniform::SampleUniform;
use rand::Rng;

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
    match Orientation::new(&pts[0], &pts[1], &pts[2]) {
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

  pub fn view(&'_ self) -> TriangleView<'_, T> {
    TriangleView([&self.0[0], &self.0[1], &self.0[2]])
  }
}

pub struct TriangleView<'a, T>([&'a Point<T, 2>; 3]);

impl<'a, T> TriangleView<'a, T>
where
  T: PolygonScalar,
  // for<'x> &'x T: PolygonScalarRef<&'x T, T>,
{
  // O(1)
  pub fn new(pts: [&'a Point<T, 2>; 3]) -> Result<TriangleView<'a, T>, Error> {
    let triangle = TriangleView(pts);
    triangle.validate()?;
    Ok(triangle)
  }

  pub fn new_ccw(pts: [&'a Point<T, 2>; 3]) -> TriangleView<'a, T> {
    match Orientation::new(&pts[0], &pts[1], &pts[2]) {
      Orientation::CounterClockWise => TriangleView(pts),
      Orientation::ClockWise => TriangleView([&pts[2], &pts[1], &pts[0]]),
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
    Orientation::new(&arr[0], &arr[1], &arr[2])
  }

  // O(1)
  pub fn locate(&self, pt: &Point<T, 2>) -> PointLocation {
    use Orientation::*;
    debug_assert_ok!(self.validate());
    let [a, b, c] = self.0;
    let ab = a.orientation(b, pt);
    let bc = b.orientation(c, pt);
    let ca = c.orientation(a, pt);
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
      .min(self.0[1].x_coord())
      .min(self.0[1].x_coord())
      .clone();
    let max_x = self.0[0]
      .x_coord()
      .max(self.0[1].x_coord())
      .max(self.0[1].x_coord())
      .clone();
    let min_y = self.0[0]
      .y_coord()
      .min(self.0[1].y_coord())
      .min(self.0[1].y_coord())
      .clone();
    let max_y = self.0[0]
      .y_coord()
      .max(self.0[1].y_coord())
      .max(self.0[1].y_coord())
      .clone();
    (Point::new([min_x, min_y]), Point::new([max_x, max_y]))
  }

  pub fn rejection_sampling<R>(&self, rng: &mut R) -> Point<T, 2>
  where
    R: Rng + ?Sized,
    T: SampleUniform,
  {
    let (min, max) = self.bounding_box();
    loop {
      let [min_x, min_y] = min.array.clone();
      let [max_x, max_y] = max.array.clone();
      let x = rng.gen_range(min_x..=max_x);
      let y = rng.gen_range(min_y..=max_y);
      let pt = Point::new([x, y]);
      if self.locate(&pt) != PointLocation::Outside {
        return pt;
      }
    }
  }

  //   sampleTriangle :: (RandomGen g, Random r, Fractional r, Ord r) => Triangle 2 p r -> Rand g (Point 2 r)
  // sampleTriangle (Triangle v1 v2 v3) = do
  //   a' <- getRandomR (0, 1)
  //   b' <- getRandomR (0, 1)
  //   let (a, b) = if a' + b' > 1 then (1 - a', 1 - b') else (a', b')
  //   return $ v1^.core .+^ a*^u .+^ b*^v
  //   where
  //     u = v2^.core .-. v1^.core
  //     v = v3^.core .-. v1^.core
}
