// https://en.wikipedia.org/wiki/Monotone_polygon
use crate::data::Point;
use crate::{Error, PolygonScalar};
use std::vec;
use std::collections::HashMap;

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
            return Err(Error::InsufficientVertices);
        }
    }
    monotone_polygons[1].extend_from_slice(&reordered_polygon[idx..]);
    Ok(monotone_polygons)
}
