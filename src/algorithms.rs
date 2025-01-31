pub mod convex_hull;
pub mod intersection;
pub mod polygonization;
pub mod triangulation;
pub mod visibility;
pub mod zhash;
pub mod boolean_operations;

#[doc(inline)]
pub use convex_hull::graham_scan::convex_hull;

#[doc(inline)]
pub use intersection::naive::segment_intersections;
