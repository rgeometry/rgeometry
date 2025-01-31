use crate::data::{Point, Polygon};
use crate::{Error, PolygonScalar};

pub enum BooleanOperation {
    And,
    Or,
    Not,
    Xor,
}

impl BooleanOperation {
    pub fn apply<T>(self, a: &Polygon<T>, b: &Polygon<T>) -> Result<Polygon<T>, Error>
    where
        T: PolygonScalar,
    {
        match self {
            BooleanOperation::And => and(a, b),
            BooleanOperation::Or => or(a, b),
            BooleanOperation::Not => not(a, b),
            BooleanOperation::Xor => xor(a, b),
        }
    }
}

fn and<T>(a: &Polygon<T>, b: &Polygon<T>) -> Result<Polygon<T>, Error>
where
    T: PolygonScalar,
{
    // Implement AND operation
    unimplemented!()
}

fn or<T>(a: &Polygon<T>, b: &Polygon<T>) -> Result<Polygon<T>, Error>
where
    T: PolygonScalar,
{
    // Implement OR operation
    unimplemented!()
}

fn not<T>(a: &Polygon<T>, b: &Polygon<T>) -> Result<Polygon<T>, Error>
where
    T: PolygonScalar,
{
    // Implement NOT operation
    unimplemented!()
}

fn xor<T>(a: &Polygon<T>, b: &Polygon<T>) -> Result<Polygon<T>, Error>
where
    T: PolygonScalar,
{
    // Implement XOR operation
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Point;
    use crate::Polygon;

    #[test]
    fn test_and() {
        let a = Polygon::new(vec![
            Point::new([0, 0]),
            Point::new([4, 0]),
            Point::new([4, 4]),
            Point::new([0, 4]),
        ])
        .unwrap();
        let b = Polygon::new(vec![
            Point::new([2, 2]),
            Point::new([6, 2]),
            Point::new([6, 6]),
            Point::new([2, 6]),
        ])
        .unwrap();
        let result = BooleanOperation::And.apply(&a, &b);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.points.len(), 4);
    }

    #[test]
    fn test_or() {
        let a = Polygon::new(vec![
            Point::new([0, 0]),
            Point::new([4, 0]),
            Point::new([4, 4]),
            Point::new([0, 4]),
        ])
        .unwrap();
        let b = Polygon::new(vec![
            Point::new([2, 2]),
            Point::new([6, 2]),
            Point::new([6, 6]),
            Point::new([2, 6]),
        ])
        .unwrap();
        let result = BooleanOperation::Or.apply(&a, &b);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.points.len(), 8);
    }

    #[test]
    fn test_not() {
        let a = Polygon::new(vec![
            Point::new([0, 0]),
            Point::new([4, 0]),
            Point::new([4, 4]),
            Point::new([0, 4]),
        ])
        .unwrap();
        let b = Polygon::new(vec![
            Point::new([2, 2]),
            Point::new([6, 2]),
            Point::new([6, 6]),
            Point::new([2, 6]),
        ])
        .unwrap();
        let result = BooleanOperation::Not.apply(&a, &b);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.points.len(), 8);
    }

    #[test]
    fn test_xor() {
        let a = Polygon::new(vec![
            Point::new([0, 0]),
            Point::new([4, 0]),
            Point::new([4, 4]),
            Point::new([0, 4]),
        ])
        .unwrap();
        let b = Polygon::new(vec![
            Point::new([2, 2]),
            Point::new([6, 2]),
            Point::new([6, 6]),
            Point::new([2, 6]),
        ])
        .unwrap();
        let result = BooleanOperation::Xor.apply(&a, &b);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.points.len(), 8);
    }
}
