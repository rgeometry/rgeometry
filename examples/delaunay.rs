use rand::prelude::*;
use rgeometry::algorithms::triangulation::delaunay::*;
use rgeometry::data::Point;

fn print_net(net: &TriangularNetwork<f64>) {
  for (i, t) in net.triangles.iter().enumerate() {
    if t.is_super() {
      continue;
    }
    println!(
      "triangle #{}: [{:?}, {:?}, {:?}]",
      i, t.vertices[0], t.vertices[1], t.vertices[2],
    );
  }
}

pub fn main() {
  let view = 10.0;
  let v = view * 4.0;

  // construct TriangularNetwork
  let mut net = TriangularNetwork::new(
    Point::new([-v, -v]),
    Point::new([v, -v]),
    Point::new([0.0, v]),
  );

  let num_vertices = 10;

  // add random points to the network
  let mut rng = rand::thread_rng();
  for _i in 0..num_vertices {
    let x = rng.gen_range(-view..view);
    let y = rng.gen_range(-view..view);
    let p = Point::new([x, y]);

    net.insert(&p);
  }

  println!("delaunay triangles");
  print_net(&net);

  // constrainted delaunay triangulation: add constrainted edge
  {
    // add contraints between added edges
    let v0 = VertIdx(rng.gen_range(0..num_vertices) + 3);
    let v1 = VertIdx(rng.gen_range(0..num_vertices) + 3);

    assert!(!v0.is_super());
    assert!(!v1.is_super());

    println!(
      "delaunay triangles, after contrainting edge between {:?} and {:?}",
      v0, v1
    );
    net.constrain_edge(v0, v1);

    print_net(&net);
  }
}
