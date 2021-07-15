use rgeometry::data::{planar_graph as pg, PlanarGraph};
use rgeometry::Error;

fn main() -> Result<(), Error> {
  let pg = PlanarGraph::new_from_faces(vec![vec![0, 1, 2, 3], vec![4, 3, 2, 1]])?;
  // let pg = PlanarGraph::new_from_faces(vec![vec![0, 1, 2, 3]])?;

  dbg!(pg.faces().collect::<Vec<pg::FaceId>>());
  // dbg!(&pg);

  dbg!(pg.tutte_embedding());

  Ok(())
}
