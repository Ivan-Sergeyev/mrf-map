#![allow(dead_code)]

use std::convert::From;
use std::ops::{Index, IndexMut};

use bitvec::vec::BitVec;

/// A 2-dimensional table stored contiguously in memory and indexed manually.
/// Serves to replace Vec<Vec<T>> in cases when inner Vec's might have different lengths.
/// todo: add example usage
pub struct JaggedArray2<T> {
    index_shift: Vec<usize>,
    data: Vec<T>,
}

impl<T> JaggedArray2<T> {
    pub fn new() -> Self {
        JaggedArray2 {
            index_shift: vec![0; 1],
            data: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.index_shift.len() - 1
    }

    pub fn inner_len(&self, index: usize) -> usize {
        assert!(index < self.len());
        self.index_shift[index + 1] - self.index_shift[index]
    }

    fn internal_index(&self, index: [usize; 2]) -> usize {
        assert!(index[0] < self.len());
        assert!(index[1] < self.index_shift[index[0] + 1] - self.index_shift[index[0]]);
        self.index_shift[index[0]] + index[1]
    }

    pub fn get(&self, index: [usize; 2]) -> &T {
        &self.data[self.internal_index(index)]
    }

    pub fn get_mut(&mut self, index: [usize; 2]) -> &mut T {
        let idx = self.internal_index(index);
        &mut self.data[idx]
    }

    pub fn set(&mut self, index: [usize; 2], value: T) {
        let idx = self.internal_index(index);
        self.data[idx] = value;
    }
}

impl<T> Index<[usize; 2]> for JaggedArray2<T> {
    type Output = T;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        self.get(index)
    }
}

impl<T> IndexMut<[usize; 2]> for JaggedArray2<T> {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut T {
        self.get_mut(index)
    }
}

impl<T> From<Vec<Vec<T>>> for JaggedArray2<T> {
    fn from(value: Vec<Vec<T>>) -> Self {
        // precompute index shifts
        let mut index_shift = Vec::with_capacity(value.len() + 1);
        index_shift.push(0);
        for i in 0..value.len() {
            index_shift.push(index_shift[i] + value[i].len());
        }

        // flatten data
        let data = value.into_iter().flatten().collect();

        // construct jagged table
        JaggedArray2 {
            index_shift: index_shift,
            data: data,
        }
    }
}

/// A 2-dimensional bool table stored contiguously in memory and indexed manually.
/// Ensures that each bool takes exactly one bit of memory.
/// Serves to replace Vec<Vec<bool>> in cases when all inner Vec's have the same length.
/// Analogous to BitVec, but 2-dimensional.
/// todo: add example usage
pub struct JaggedBitArray2 {
    index_shift: Vec<usize>,
    data: BitVec,
}

impl JaggedBitArray2 {
    pub fn new() -> Self {
        JaggedBitArray2 {
            index_shift: vec![0; 1],
            data: BitVec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.index_shift.len() - 1
    }

    pub fn inner_len(&self, index: usize) -> usize {
        assert!(index < self.len());
        self.index_shift[index + 1] - self.index_shift[index]
    }

    fn internal_index(&self, index: [usize; 2]) -> usize {
        assert!(index[0] < self.len());
        assert!(index[1] < self.index_shift[index[0] + 1] - self.index_shift[index[0]]);
        self.index_shift[index[0]] + index[1]
    }

    pub fn get(&self, index: [usize; 2]) -> &bool {
        &self.data[self.internal_index(index)]
    }

    pub fn set(&mut self, index: [usize; 2], value: bool) {
        let idx = self.internal_index(index);
        self.data.set(idx, value);
    }
}

impl Index<[usize; 2]> for JaggedBitArray2 {
    type Output = bool;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        &self.get(index)
    }
}

impl From<Vec<Vec<bool>>> for JaggedBitArray2 {
    fn from(value: Vec<Vec<bool>>) -> Self {
        // precompute index shifts
        let mut index_shift = Vec::with_capacity(value.len() + 1);
        index_shift.push(0);
        for i in 0..value.len() {
            index_shift.push(index_shift[i] + value[i].len());
        }

        // flatten data
        let data = value.into_iter().flatten().collect();

        // construct jagged bit table
        JaggedBitArray2 {
            index_shift: index_shift,
            data: data,
        }
    }
}

#[cfg(test)]
mod tests {
    // todo: tests
}
