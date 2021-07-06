use array_init::{array_init, try_array_init};
use rand::Rng;
use std::convert::TryInto;
use std::ops::{Index, IndexMut};

pub type SparseIndex = usize;
pub struct SparseVec<T> {
  dense: DenseCollection,
  arr: Vec<T>,
  free: Vec<SparseIndex>,
}

impl<T: Copy + Default> SparseVec<T> {
  pub fn new() -> SparseVec<T> {
    SparseVec {
      dense: DenseCollection::new(),
      arr: Vec::new(),
      free: Vec::new(),
    }
  }

  fn alloc(&mut self) -> SparseIndex {
    let idx = self.free.pop().unwrap_or(self.arr.len());
    self.dense.push(idx);
    idx
  }

  pub fn push(&mut self, elt: T) -> SparseIndex {
    let idx = self.alloc();
    self[idx] = elt;
    idx
  }

  pub fn remove(&mut self, idx: SparseIndex) -> T {
    self.dense.remove(idx);
    self.free.push(idx);
    let ret = self.arr[idx];
    self.arr[idx] = T::default();
    ret
  }

  pub fn random<R>(&self, rng: &mut R) -> Option<SparseIndex>
  where
    R: Rng + ?Sized,
  {
    self.dense.random(rng)
  }

  pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
    self.dense.iter().map(move |idx| self.arr.index(idx))
  }
}

impl<T: Copy + Default> Index<SparseIndex> for SparseVec<T> {
  type Output = T;
  fn index(&self, index: SparseIndex) -> &T {
    self.arr.index(index)
  }
}
impl<T: Copy + Default> IndexMut<SparseIndex> for SparseVec<T> {
  fn index_mut(&mut self, index: SparseIndex) -> &mut T {
    if index >= self.arr.len() {
      self.arr.resize(index + 1, T::default());
    }
    self.arr.index_mut(index)
  }
}

type Dense = usize;
type Sparse = usize;
struct DenseCollection {
  dense: Vec<Sparse>,
  dense_rev: Vec<Dense>,
}

impl DenseCollection {
  fn new() -> DenseCollection {
    DenseCollection {
      dense: Vec::new(),
      dense_rev: Vec::new(),
    }
  }

  fn push(&mut self, elt: Sparse) {
    let idx = self.dense.len();
    self.dense.push(elt);
    self
      .dense_rev
      .resize(std::cmp::max(self.dense_rev.len(), elt + 1), usize::MAX);
    self.dense_rev[elt] = idx;
  }

  // Swap the dense entry for 'elt' with the last entry.
  // Update reverse mapping for the swapped entry to point to the new idx.
  fn remove(&mut self, elt: Sparse) {
    let elt_dense_idx = self.dense_rev[elt];
    let last_sparse = *self.dense.last().unwrap();
    self.dense.swap_remove(elt_dense_idx);
    self.dense_rev[last_sparse] = elt_dense_idx;
  }

  fn random<R>(&self, rng: &mut R) -> Option<Sparse>
  where
    R: Rng + ?Sized,
  {
    if self.dense.is_empty() {
      return None;
    }
    let idx = rng.gen_range(0..self.dense.len());
    Some(self.dense[idx])
  }

  fn iter(&self) -> impl Iterator<Item = Sparse> + '_ {
    self.dense.iter().copied()
  }
}

///////////////////////////////////////////////////////////////////////////////
// Iterator permutations

#[cfg(test)]
pub fn permutations<T, const N: usize>(source: [&[T]; N]) -> impl Iterator<Item = [T; N]> + '_
where
  T: Copy,
{
  let mut indices: [usize; N] = [0; N];
  let mut done = false;
  std::iter::from_fn(move || {
    if done {
      return None;
    }
    let out = array_init(|i| source[i][indices[i]]);
    indices[0] += 1;
    let mut i = 0;
    while !(indices[i] < source[i].len()) {
      indices[i] = 0;
      i += 1;
      if i == N {
        done = true;
        break;
      } else {
        indices[i] += 1;
      }
    }
    Some(out)
  })
}
