use crate::data::Polygon;
use std::fmt;

/// A pretty-printer for `Polygon<i8>` using Braille Unicode symbols.
///
/// Braille characters (U+2800 to U+28FF) encode 2×4 dot patterns, making them perfect
/// for rendering small polygons as text-based graphics. Each Braille character represents
/// a 2-pixel-wide by 4-pixel-tall cell.
///
/// # Braille Dot Numbering
///
/// Standard Braille numbering:
/// ```text
/// 1  4
/// 2  5
/// 3  6
/// 7  8
/// ```
///
/// # Examples
///
/// ```
/// use rgeometry::data::{Point, Polygon};
/// use rgeometry::data::polygon::BraillePrinter;
///
/// let triangle = Polygon::new(vec![
///   Point::new([0, 0]),
///   Point::new([10, 0]),
///   Point::new([5, 8]),
/// ]).unwrap();
///
/// let braille = BraillePrinter::new(&triangle);
/// println!("{}", braille);
/// ```
pub struct BraillePrinter<'a> {
  polygon: &'a Polygon<i8>,
}

impl<'a> BraillePrinter<'a> {
  /// Create a new Braille printer for a polygon.
  pub fn new(polygon: &'a Polygon<i8>) -> Self {
    BraillePrinter { polygon }
  }

  /// Render the polygon to a string using Braille characters.
  ///
  /// # Examples
  ///
  /// ```
  /// use rgeometry::data::{Point, Polygon};
  /// use rgeometry::data::polygon::BraillePrinter;
  ///
  /// // Create a small triangle
  /// let triangle = Polygon::new(vec![
  ///   Point::new([0i8, 0]),
  ///   Point::new([4, 0]),
  ///   Point::new([2, 3]),
  /// ]).unwrap();
  ///
  /// let printer = BraillePrinter::new(&triangle);
  /// let output = format!("{}", printer);
  ///
  /// // The output should match this Braille pattern
  /// let expected = "\
  /// ⠐⡖⡖⠀
  /// ⠀⠈⠀⠀
  /// ";
  /// assert_eq!(output, expected);
  /// ```
  fn render(&self) -> String {
    // Get bounding box
    let (min, max) = self.polygon.bounding_box();
    let min_x = min.x_coord();
    let min_y = min.y_coord();
    let max_x = max.x_coord();
    let max_y = max.y_coord();

    // Calculate dimensions with padding
    let width = (max_x - min_x + 1) as usize + 2; // +2 for padding
    let height = (max_y - min_y + 1) as usize + 2; // +2 for padding

    // Create a pixel grid (false = empty, true = filled)
    let mut grid = vec![vec![false; width]; height];

    // Rasterize polygon edges using Bresenham's line algorithm
    for edge in self.polygon.iter_boundary_edges() {
      let x0 = (edge.src.x_coord() - min_x + 1) as i32;
      let y0 = (edge.src.y_coord() - min_y + 1) as i32;
      let x1 = (edge.dst.x_coord() - min_x + 1) as i32;
      let y1 = (edge.dst.y_coord() - min_y + 1) as i32;

      bresenham_line(x0, y0, x1, y1, |x, y| {
        if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
          grid[y as usize][x as usize] = true;
        }
      });
    }

    // Convert grid to Braille characters
    self.grid_to_braille(&grid)
  }

  /// Convert a pixel grid to Braille characters.
  fn grid_to_braille(&self, grid: &[Vec<bool>]) -> String {
    let height = grid.len();
    let width = if height > 0 { grid[0].len() } else { 0 };

    // Braille characters are 2 pixels wide and 4 pixels tall
    let braille_rows = height.div_ceil(4);
    let braille_cols = width.div_ceil(2);

    let mut result = String::new();

    for row in 0..braille_rows {
      for col in 0..braille_cols {
        let base_y = row * 4;
        let base_x = col * 2;

        // Map grid pixels to Braille dots
        // Braille dot numbering (in Unicode order):
        // 0: dot 1 (top-left)
        // 1: dot 2 (middle-left)
        // 2: dot 3 (bottom-left)
        // 3: dot 4 (top-right)
        // 4: dot 5 (middle-right)
        // 5: dot 6 (bottom-right)
        // 6: dot 7 (below bottom-left)
        // 7: dot 8 (below bottom-right)

        let mut dots = 0u8;

        // Left column
        if self.get_pixel(grid, base_x, base_y) {
          dots |= 0b00000001; // dot 1
        }
        if self.get_pixel(grid, base_x, base_y + 1) {
          dots |= 0b00000010; // dot 2
        }
        if self.get_pixel(grid, base_x, base_y + 2) {
          dots |= 0b00000100; // dot 3
        }
        if self.get_pixel(grid, base_x, base_y + 3) {
          dots |= 0b01000000; // dot 7
        }

        // Right column
        if self.get_pixel(grid, base_x + 1, base_y) {
          dots |= 0b00001000; // dot 4
        }
        if self.get_pixel(grid, base_x + 1, base_y + 1) {
          dots |= 0b00010000; // dot 5
        }
        if self.get_pixel(grid, base_x + 1, base_y + 2) {
          dots |= 0b00100000; // dot 6
        }
        if self.get_pixel(grid, base_x + 1, base_y + 3) {
          dots |= 0b10000000; // dot 8
        }

        // Convert to Braille Unicode character
        // U+2800 is the base Braille pattern (blank)
        let braille_char = char::from_u32(0x2800 + dots as u32).unwrap_or('?');
        result.push(braille_char);
      }
      result.push('\n');
    }

    result
  }

  /// Get a pixel from the grid, returning false if out of bounds.
  fn get_pixel(&self, grid: &[Vec<bool>], x: usize, y: usize) -> bool {
    grid
      .get(y)
      .and_then(|row| row.get(x))
      .copied()
      .unwrap_or(false)
  }
}

impl<'a> fmt::Display for BraillePrinter<'a> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.render())
  }
}

/// Bresenham's line drawing algorithm.
///
/// Calls the provided closure for each point along the line from (x0, y0) to (x1, y1).
fn bresenham_line<F>(x0: i32, y0: i32, x1: i32, y1: i32, mut plot: F)
where
  F: FnMut(i32, i32),
{
  let dx = (x1 - x0).abs();
  let dy = -(y1 - y0).abs();
  let sx = if x0 < x1 { 1 } else { -1 };
  let sy = if y0 < y1 { 1 } else { -1 };
  let mut error = dx + dy;

  let mut x = x0;
  let mut y = y0;

  loop {
    plot(x, y);

    if x == x1 && y == y1 {
      break;
    }

    let e2 = 2 * error;

    if e2 >= dy {
      if x == x1 {
        break;
      }
      error += dy;
      x += sx;
    }

    if e2 <= dx {
      if y == y1 {
        break;
      }
      error += dx;
      y += sy;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::Point;

  #[test]
  fn test_braille_triangle() {
    let triangle = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([10, 0]),
      Point::new([5, 8]),
    ])
    .unwrap();

    let printer = BraillePrinter::new(&triangle);
    let output = printer.render();

    // The output should be non-empty and contain Braille characters
    assert!(!output.is_empty());
    assert!(
      output
        .chars()
        .any(|c| ('\u{2800}'..='\u{28FF}').contains(&c))
    );
  }

  #[test]
  fn test_braille_square() {
    let square = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([10, 0]),
      Point::new([10, 10]),
      Point::new([0, 10]),
    ])
    .unwrap();

    let printer = BraillePrinter::new(&square);
    let output = printer.render();

    assert!(!output.is_empty());
    assert!(
      output
        .chars()
        .any(|c| ('\u{2800}'..='\u{28FF}').contains(&c))
    );
  }

  #[test]
  fn test_braille_small_triangle() {
    let triangle = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([4, 0]),
      Point::new([2, 3]),
    ])
    .unwrap();

    let printer = BraillePrinter::new(&triangle);
    let output = printer.render();

    assert!(!output.is_empty());
  }

  #[test]
  fn test_bresenham_horizontal() {
    let mut points = Vec::new();
    bresenham_line(0, 0, 5, 0, |x, y| points.push((x, y)));

    assert_eq!(points.len(), 6);
    assert_eq!(points[0], (0, 0));
    assert_eq!(points[5], (5, 0));
  }

  #[test]
  fn test_bresenham_vertical() {
    let mut points = Vec::new();
    bresenham_line(0, 0, 0, 5, |x, y| points.push((x, y)));

    assert_eq!(points.len(), 6);
    assert_eq!(points[0], (0, 0));
    assert_eq!(points[5], (0, 5));
  }

  #[test]
  fn test_bresenham_diagonal() {
    let mut points = Vec::new();
    bresenham_line(0, 0, 3, 3, |x, y| points.push((x, y)));

    assert_eq!(points.len(), 4);
    assert_eq!(points[0], (0, 0));
    assert_eq!(points[3], (3, 3));
  }

  #[test]
  fn test_display_trait() {
    let triangle = Polygon::new(vec![
      Point::new([0, 0]),
      Point::new([8, 0]),
      Point::new([4, 6]),
    ])
    .unwrap();

    let printer = BraillePrinter::new(&triangle);
    let display_output = format!("{}", printer);

    assert!(!display_output.is_empty());
    assert!(
      display_output
        .chars()
        .any(|c| ('\u{2800}'..='\u{28FF}').contains(&c))
    );
  }
}
