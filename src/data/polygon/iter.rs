// use claim::debug_assert_ok;

use crate::data::Cursor;
use crate::data::DirectedEdge;
use crate::data::Point;

pub struct Iter<'a, T: 'a> {
  pub(crate) iter: std::slice::Iter<'a, Point<T, 2>>,
}
impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a Point<T, 2>;
  fn next(&mut self) -> Option<&'a Point<T, 2>> {
    self.iter.next()
  }
}

pub struct IterMut<'a, T: 'a> {
  pub(crate) points: std::slice::IterMut<'a, Point<T, 2>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
  type Item = &'a mut Point<T, 2>;
  fn next(&mut self) -> Option<&'a mut Point<T, 2>> {
    self.points.next()
  }
}

pub struct EdgeIter<'a, T: 'a> {
  pub(crate) iter: CursorIter<'a, T>,
}

impl<'a, T: Clone> Iterator for EdgeIter<'a, T> {
  type Item = DirectedEdge<'a, T, 2>;
  fn next(&mut self) -> Option<Self::Item> {
    let cursor = self.iter.next()?;
    let this_point = cursor.point();
    let next_point = cursor.next().point();
    Some(DirectedEdge {
      src: this_point,
      dst: next_point,
    })
  }
}

pub struct CursorIter<'a, T: 'a> {
  pub(crate) cursor_head: Cursor<'a, T>,
  pub(crate) cursor_tail: Cursor<'a, T>, // inclusive
  pub(crate) exhausted: bool,
}

impl<'a, T> Iterator for CursorIter<'a, T> {
  type Item = Cursor<'a, T>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.exhausted {
      None
    } else {
      let out = self.cursor_head;
      if out == self.cursor_tail {
        self.exhausted = true;
      } else {
        self.cursor_head.move_next();
      }
      Some(out)
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (self.len(), Some(self.len()))
  }
}

impl<'a, T> ExactSizeIterator for CursorIter<'a, T> {
  fn len(&self) -> usize {
    let pos_head = self.cursor_head.position;
    let pos_tail = self.cursor_tail.position;
    if pos_head.position_id.0 <= pos_tail.position_id.0 {
      pos_tail.position_id.0 - pos_head.position_id.0 + 1
    } else {
      pos_head.size - pos_head.position_id.0 + pos_tail.position_id.0 + 1
    }
  }
}

impl<'a, T> DoubleEndedIterator for CursorIter<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.exhausted {
      None
    } else {
      let out = self.cursor_tail;
      if out == self.cursor_head {
        self.exhausted = true;
      } else {
        self.cursor_tail.move_next();
      }
      Some(out)
    }
  }
}
