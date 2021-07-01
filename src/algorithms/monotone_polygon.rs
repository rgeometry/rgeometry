// https://en.wikipedia.org/wiki/Monotone_polygon
use crate::data::Point;
use crate::{Error, PolygonScalar};
use std::vec;
use std::collections::HashMap;

/// Checks if points form a y-monotone polygon, returns its paths if true
pub fn get_y_monotone_polygons<T>(points: &Vec<Point<T, 2>>) -> Result<[Vec<&Point<T, 2>>;2], Error>
where
  T: PolygonScalar,
{
    if points.len() < 3 { return Err(Error::InsufficientVertices) }

    let mut monotone_polygons : [Vec<&Point<T, 2>>;2] = Default::default();
    let mut reordered_polygon  : Vec<&Point<T, 2>> = Vec::new();

    let max_y_index = points.iter().enumerate()
        .max_by(|(_,curr),(_,nxt)| curr.array[1].cmp(&nxt.array[1]))
        .map(|(idx,_)| idx).expect("Empty Polygon");

    //Reorder polygon on Y index
    for i in max_y_index..points.len() {
        reordered_polygon.push(&points[i]);
    }
    for i in 0..max_y_index {
        reordered_polygon.push(&points[i]);
    }

    //Get monotone polygons if any
    let mut idx = 1;
    while idx < reordered_polygon.len() {
        let curr = reordered_polygon[idx];
        let prev = reordered_polygon[idx-1];

        if curr.array[1] > prev.array[1] {
            break;
        }
        idx +=1;
    }
    monotone_polygons[0].extend_from_slice(&reordered_polygon[0..idx]);

    for i in idx..reordered_polygon.len() {
        let curr = reordered_polygon[i];
        let prev = reordered_polygon[i-1];
        if curr.array[1] < prev.array[1] {
            return Err(Error::InsufficientVertices); // ToDo: replace with proper error
        }
    }
    monotone_polygons[1].extend_from_slice(&reordered_polygon[idx-1..]);
    monotone_polygons[1].push(&points.last().unwrap());
    Ok(monotone_polygons)
}

//testing
#[cfg(test)]
mod monotone_testing {
    use crate::data::{Point, Polygon, PolygonConvex};
    use crate::algorithms::monotone_polygon::get_y_monotone_polygons;
    use std::assert;

    #[test]
    fn convex_polygon_is_montone()
    {
        let mut rng = rand::thread_rng();
        let convex_polygon = PolygonConvex::random(10,100,&mut rng);
        let res = get_y_monotone_polygons(&convex_polygon.points);
        assert!(res.is_ok());
    }

    #[test]
    fn non_y_monotone()
    {
        let polygon = Polygon::new(
            vec![Point::new([0,1]),Point::new([1,2]),Point::new([1,-2]),Point::new([0,-1]),Point::new([-1,-2]),Point::new([-1,2])]).unwrap();
        let res = get_y_monotone_polygons(&polygon.points);
        assert!(res.is_err());
    }
    #[test]
    fn convex_polygon_monotone_paths()
    {
        let polygon = Polygon::new(
            vec![Point::new([0,3]),Point::new([1,2]),Point::new([1,-2]),Point::new([0,-3]),Point::new([-1,-2]),Point::new([-1,2])]).unwrap();
        let res = get_y_monotone_polygons(&polygon.points);
        assert!(res.is_ok());
        let res_paths = res.unwrap();
        assert_eq!(res_paths[0].len(),  4);
        assert_eq!(res_paths[1].len(),  4);
    }
    #[test]
    fn monotone_mountain()
    {
        let polygon = Polygon::new(
            vec![Point::new([0,3]),Point::new([1,2]),Point::new([1,-2]),Point::new([0,-3])]).unwrap();
        let res = get_y_monotone_polygons(&polygon.points);
        assert!(res.is_ok());
        let res_paths = res.unwrap();
        assert_eq!(res_paths[0].len(),  4);
        assert_eq!(res_paths[1].len(),  2);
    }
}
