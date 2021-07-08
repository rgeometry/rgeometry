pub mod convex_hull;
pub mod intersection;
pub mod monotone_polygon;
pub mod polygonization;
pub mod triangulation;
pub mod zhash;

#[doc(inline)]
pub use convex_hull::graham_scan::convex_hull;

#[doc(inline)]
pub use polygonization::*;

#[doc(inline)]
pub use intersection::naive::segment_intersections;
