pub(crate) mod monotone;
mod star;
mod two_opt;

pub use monotone::new_monotone_polygon;
pub use star::new_star_polygon;
pub use two_opt::resolve_self_intersections;
pub use two_opt::two_opt_moves;

// pub use monotone;
// pub use star;
// pub use two_opt;
