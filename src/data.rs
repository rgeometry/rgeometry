mod line_segment;
pub mod polygon;
mod triangle;

pub use line_segment::*;
pub use triangle::*;

pub use crate::point::Point;
pub use crate::transformation::Transform;
pub use crate::vector::{Vector, VectorView};
