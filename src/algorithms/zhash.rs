use crate::data::Point;

///////////////////////////////////////////////////////////////////////////////
// Z-Order Hash

// A        = aaaa =  a a a a
// B        = bbbb = b b b b
// zhash_pair(a,b) = babababa

pub struct ZHashBox<'a, T> {
  min_x: &'a T,
  max_x: &'a T,
  min_y: &'a T,
  max_y: &'a T,
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

  use proptest::prelude::*;

  proptest! {
    #[test]
    fn hash_unhash(a in any::<u32>(), b in any::<u32>()) {
      prop_assert_eq!(zunhash_pair(zhash_pair(a,b)), (a,b))

    }
  }
}
