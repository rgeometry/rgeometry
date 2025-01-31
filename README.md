# RGeometry

Computational Geometry in Rust!

[![Crates.io](https://img.shields.io/crates/v/rgeometry?color=4d76ae)](https://crates.io/crates/rgeometry)
[![API](https://docs.rs/rgeometry/badge.svg)](https://docs.rs/rgeometry)
[![API](https://img.shields.io/badge/docs-head-4d76ae.svg)](https://rgeometry.org/rgeometry/rgeometry/)
[![codecov](https://codecov.io/gh/rgeometry/rgeometry/branch/main/graph/badge.svg?token=A0EFH689BR)](https://codecov.io/gh/rgeometry/rgeometry)
[![GitHub branch checks state](https://img.shields.io/github/checks-status/rgeometry/rgeometry/main?label=tests&logo=github)](https://github.com/rgeometry/rgeometry/actions/workflows/ci.yml)
[![dependency status](https://deps.rs/repo/github/rgeometry/rgeometry/status.svg)](https://deps.rs/repo/github/rgeometry/rgeometry)
[![Discord](https://img.shields.io/discord/731822102935502908)](https://discord.gg/vZZmxwWjeZ)

<!-- [![](https://tokei.rs/b1/github/rgeometry/rgeometry?category=code)](https://github.com/XAMPPRocky/tokei#badges) -->

--------------------------------

## What is RGeometry?

RGeometry is a collection of data types such as points, polygons, lines, and segments, and a variety of algorithms for manipulating them. This crate will be of use to you if you've ever wondered if a point is inside a polygon or if a bank is adequately covered by surveillance cameras.

Check out the API documentation for more details. Under each function, there is an interactive example (powered by rust->wasm).

## Boolean Operations on Polygons

RGeometry now includes Boolean operations on polygons, such as AND, OR, NOT, and XOR. These operations allow you to perform set operations on polygons, which are widely used in computer graphics, CAD, and EDA.

### Examples

```rust
use rgeometry::data::{Point, Polygon};
use rgeometry::algorithms::boolean_operations::BooleanOperation;

let a = Polygon::new(vec![
    Point::new([0, 0]),
    Point::new([4, 0]),
    Point::new([4, 4]),
    Point::new([0, 4]),
]).unwrap();

let b = Polygon::new(vec![
    Point::new([2, 2]),
    Point::new([6, 2]),
    Point::new([6, 6]),
    Point::new([2, 6]),
]).unwrap();

let result = BooleanOperation::And.apply(&a, &b).unwrap();
println!("AND result: {:?}", result);

let result = BooleanOperation::Or.apply(&a, &b).unwrap();
println!("OR result: {:?}", result);

let result = BooleanOperation::Not.apply(&a, &b).unwrap();
println!("NOT result: {:?}", result);

let result = BooleanOperation::Xor.apply(&a, &b).unwrap();
println!("XOR result: {:?}", result);
```

## MSRV

rust-1.59

## Contribute

If you want to learn Rust or computational geometry or both, hit me up in discord and I'll (@lemmih) mentor you. There is a long list of algorithms (ranging from easy to difficult) yet to be implemented in Rust.

## Resources:
 * Website: https://rgeometry.org/
 * Discord Chat: https://discord.gg/vZZmxwWjeZ
 * API Documentation: https://docs.rs/rgeometry
