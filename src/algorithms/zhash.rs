use crate::data::Point;

///////////////////////////////////////////////////////////////////////////////
// Z-Order Hash

// A        = aaaa =  a a a a
// B        = bbbb = b b b b
// zhash_pair(a,b) = babababa

#[derive(Debug)]
pub struct ZHashBox<'a, T> {
  min_x: &'a T,
  max_x: &'a T,
  min_y: &'a T,
  max_y: &'a T,
}
impl<'a, T> Copy for ZHashBox<'a, T> {}

impl<'a, T> Clone for ZHashBox<'a, T> {
  fn clone(&self) -> ZHashBox<'a, T> {
    ZHashBox {
      min_x: self.min_x,
      max_x: self.max_x,
      min_y: self.min_y,
      max_y: self.max_y,
    }
  }
}

pub trait ZHashable: Sized {
  type ZHashKey;
  fn zhash_key(zbox: ZHashBox<'_, Self>) -> Self::ZHashKey;
  fn zhash_fn(key: Self::ZHashKey, point: &Point<Self, 2>) -> u64;
}

impl ZHashable for f64 {
  type ZHashKey = (f64, f64, f64, f64);
  fn zhash_key(zbox: ZHashBox<'_, f64>) -> Self::ZHashKey {
    let width = zbox.max_x - zbox.min_x;
    let height = zbox.max_y - zbox.min_y;
    (*zbox.min_x, *zbox.min_y, width, height)
  }
  fn zhash_fn(key: Self::ZHashKey, point: &Point<Self, 2>) -> u64 {
    let (min_x, min_y, width, height) = key;
    let z_hash_max = u32::MAX as f64;
    let x = ((point.x_coord() - min_x) / width * z_hash_max) as u32;
    let y = ((point.y_coord() - min_y) / height * z_hash_max) as u32;
    zhash_pair(x, y)
  }
}

impl ZHashable for i64 {
  type ZHashKey = (i64, i64, u32, u32);
  fn zhash_key(zbox: ZHashBox<'_, i64>) -> Self::ZHashKey {
    let width = zbox.max_x.wrapping_sub(*zbox.min_x) as u64;
    let height = zbox.max_y.wrapping_sub(*zbox.min_y) as u64;
    let x_r_shift = 32u32.saturating_sub(width.leading_zeros());
    let y_r_shift = 32u32.saturating_sub(height.leading_zeros());
    (*zbox.min_x, *zbox.min_y, x_r_shift, y_r_shift)
  }
  fn zhash_fn(key: Self::ZHashKey, point: &Point<Self, 2>) -> u64 {
    let (min_x, min_y, x_r_shift, y_r_shift) = key;
    let x = ((point.x_coord().wrapping_sub(min_x) as u64) >> x_r_shift) as u32;
    let y = ((point.y_coord().wrapping_sub(min_y) as u64) >> y_r_shift) as u32;
    zhash_pair(x, y)
  }
}

impl ZHashable for i8 {
  type ZHashKey = (i8, i8);
  fn zhash_key(zbox: ZHashBox<'_, i8>) -> Self::ZHashKey {
    (*zbox.min_x, *zbox.min_y)
  }
  fn zhash_fn(key: Self::ZHashKey, point: &Point<Self, 2>) -> u64 {
    let (min_x, min_y) = key;
    let x = (point.x_coord().wrapping_sub(min_x) as u8) as u32;
    let y = (point.y_coord().wrapping_sub(min_y) as u8) as u32;
    dbg!(point.x_coord(), min_x, x);
    dbg!(point.y_coord(), min_y, y);
    zhash_pair(x, y)
  }
}

impl ZHashable for u64 {
  type ZHashKey = (u64, u64, u32, u32);
  fn zhash_key(zbox: ZHashBox<'_, u64>) -> Self::ZHashKey {
    let width = zbox.max_x - zbox.min_x;
    let height = zbox.max_y - zbox.min_y;
    let x_r_shift = 32u32.saturating_sub(width.leading_zeros());
    let y_r_shift = 32u32.saturating_sub(height.leading_zeros());
    (*zbox.min_x, *zbox.min_y, x_r_shift, y_r_shift)
  }
  fn zhash_fn(key: Self::ZHashKey, point: &Point<Self, 2>) -> u64 {
    let (min_x, min_y, x_r_shift, y_r_shift) = key;
    let x = ((*point.x_coord() - min_x) >> x_r_shift) as u32;
    let y = ((*point.y_coord() - min_y) >> y_r_shift) as u32;
    zhash_pair(x, y)
  }
}

impl ZHashable for u32 {
  type ZHashKey = ();
  fn zhash_key(_zbox: ZHashBox<'_, u32>) -> Self::ZHashKey {}
  fn zhash_fn(_key: Self::ZHashKey, point: &Point<Self, 2>) -> u64 {
    zhash_pair(*point.x_coord(), *point.y_coord())
  }
}

pub fn zunhash_pair(w: u64) -> (u32, u32) {
  (zunhash_u32(w), zunhash_u32(w >> 1))
}

fn zunhash_u32(w: u64) -> u32 {
  let w = w & 0x5555555555555555;
  let w = (w | w >> 1) & 0x3333333333333333;
  let w = (w | w >> 2) & 0x0F0F0F0F0F0F0F0F;
  let w = (w | w >> 4) & 0x00FF00FF00FF00FF;
  let w = (w | w >> 8) & 0x0000FFFF0000FFFF;
  let w = (w | w >> 16) & 0x00000000FFFFFFFF;
  w as u32
}

pub fn zhash_pair(a: u32, b: u32) -> u64 {
  zhash_u32(a) | zhash_u32(b) << 1
}

fn zhash_u32(w: u32) -> u64 {
  let w = w as u64; // & 0x00000000FFFFFFFF;
  let w = (w | w << 16) & 0x0000FFFF0000FFFF;
  let w = (w | w << 8) & 0x00FF00FF00FF00FF;
  let w = (w | w << 4) & 0x0F0F0F0F0F0F0F0F;
  let w = (w | w << 2) & 0x3333333333333333;
  (w | w << 1) & 0x5555555555555555
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::data::Point;
  use crate::data::Triangle;
  use crate::testing::*;

  use proptest::prelude::*;
  use rand::SeedableRng;

  proptest! {
    #[test]
    fn hash_unhash_prop(a in any::<u32>(), b in any::<u32>()) {
      prop_assert_eq!(zunhash_pair(zhash_pair(a,b)), (a,b))
    }

    #[test]
    fn cmp_prop_i8(trig in any_triangle::<i8>()) {
      let (min, max) = trig.view().bounding_box();
      let zbox = ZHashBox {
        min_x: min.x_coord(),
        max_x: max.x_coord(),
        min_y: min.y_coord(),
        max_y: max.y_coord(),
      };
      let key = ZHashable::zhash_key(zbox);
      let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
      let middle = trig.view().rejection_sampling(&mut rng);
      let min_hash = ZHashable::zhash_fn(key, &min);
      let max_hash = ZHashable::zhash_fn(key, &max);
      let mid_hash = ZHashable::zhash_fn(key, &middle);
      prop_assert!( min_hash <= mid_hash );
      prop_assert!( mid_hash <= max_hash );
    }

    #[test]
    fn cmp_prop_i64(trig in any_triangle::<i64>()) {
      let (min, max) = trig.view().bounding_box();
      let zbox = ZHashBox {
        min_x: min.x_coord(),
        max_x: max.x_coord(),
        min_y: min.y_coord(),
        max_y: max.y_coord(),
      };
      let key = ZHashable::zhash_key(zbox);
      let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
      let middle = trig.view().rejection_sampling(&mut rng);
      let min_hash = ZHashable::zhash_fn(key, &min);
      let max_hash = ZHashable::zhash_fn(key, &max);
      let mid_hash = ZHashable::zhash_fn(key, &middle);
      prop_assert!( min_hash <= mid_hash );
      prop_assert!( mid_hash <= max_hash );
    }
  }
}
