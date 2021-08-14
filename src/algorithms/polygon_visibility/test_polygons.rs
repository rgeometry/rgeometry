use crate::data::{Point, Polygon};
pub struct TestingInfo<T> {
  pub polygon: Polygon<T>,
  pub visibility_polygon: Polygon<T>,
  pub point: Point<T, 2>,
}

//  /-----\ /----\  /-----\ /----\
//  |     | |    |  |     | |    |
//  |  x  \-/ /--/  |  x  \---\--/
//  |         |     |         |
//  \---------/     \---------/
#[allow(dead_code)]
pub fn test_info_1() -> TestingInfo<i32> {
  let polygon = Polygon::new(vec![
    Point::new([0, 0]),
    Point::new([8, 0]),
    Point::new([8, 3]),
    Point::new([10, 3]),
    Point::new([10, 6]),
    Point::new([6, 6]),
    Point::new([6, 3]),
    Point::new([4, 3]),
    Point::new([4, 6]),
    Point::new([0, 6]),
  ])
  .unwrap();
  let visibility_polygon = Polygon::new(vec![
    Point::new([0, 0]),
    Point::new([8, 0]),
    Point::new([8, 3]),
    Point::new([4, 3]),
    Point::new([4, 6]),
    Point::new([0, 6]),
  ])
  .unwrap();
  let point = Point::new([2, 3]);
  TestingInfo {
    polygon,
    visibility_polygon,
    point,
  }
}

//   Input            Output
//  /-----\ /----\  /-----\ /----\
//  |     | |    |  |     | |    |
//  |  x  \-/    |  |  x  \------\
//  |            |  |            |
//  \------------/  \------------/
#[allow(dead_code)]
pub fn test_info_2() -> TestingInfo<i32> {
  let point = Point::new([2, 3]);
  let polygon = Polygon::new(vec![
    Point::new([0, 0]),
    Point::new([10, 0]),
    Point::new([10, 6]),
    Point::new([6, 6]),
    Point::new([6, 3]),
    Point::new([4, 3]),
    Point::new([4, 6]),
    Point::new([0, 6]),
  ])
  .unwrap();
  let visibility_polygon = Polygon::new(vec![
    Point::new([0, 0]),
    Point::new([10, 0]),
    Point::new([10, 3]),
    Point::new([4, 3]),
    Point::new([4, 6]),
    Point::new([0, 6]),
  ])
  .unwrap();
  TestingInfo {
    polygon,
    visibility_polygon,
    point,
  }
}
