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

pub struct Permutations<T: IntoIterator, const N: usize> {
  done: bool,
  src: [T; N],
  iters: [std::iter::Peekable<<T as IntoIterator>::IntoIter>; N],
}

impl<T, const N: usize> Iterator for Permutations<T, N>
where
  T: IntoIterator + Clone,
  <T as IntoIterator>::Item: Copy,
{
  type Item = [<T as IntoIterator>::Item; N];
  fn next(&mut self) -> Option<Self::Item> {
    let mut out: Vec<<T as IntoIterator>::Item> = Vec::new();
    let mut idx = 0;
    while idx < N {
      match self.iters[idx].peek() {
        Some(val) => {
          eprintln!("Good value at: {}", idx);
          out.push(val);
          idx += 1;
          break;
        }
        None => {
          eprintln!("Overflow at: {}", idx);
          self.iters[idx] = self.src[idx].clone().into_iter().peekable();
          let val = self.iters[idx].next()?;
          out.push(val);
          idx += 1;
          self.done = idx == N;
        }
      }
    }
    while idx < N {
      eprintln!("Peeking at: {}", idx);
      let val = *self.iters[idx].peek()?;
      out.push(val);
      idx += 1;
    }
    eprintln!("Elements: {}", out.len());
    out.try_into().ok()
  }
}

pub fn permutations<T, const N: usize>(iters: [T; N]) -> Permutations<T, N>
where
  T: IntoIterator + Clone,
{
  Permutations {
    done: false,
    src: iters.clone(),
    iters: array_init(|i| iters[i].clone().into_iter().peekable()),
  }
}
