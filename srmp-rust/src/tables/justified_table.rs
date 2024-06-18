#![allow(dead_code)]

use std::convert::From;
use std::ops::{Index, IndexMut};
use std::slice::Iter;

/// A 2-dimensional table stored contiguously in memory and indexed manually.
/// Serves to replace Vec<Vec<T>> in cases when all inner Vec's have the same length.
/// todo: add example usage
/// todo: slice indexing
/// todo: store data as Box<[T]> instead of Vec<T>?
/// Note: [ndarray::ArrayBase](https://docs.rs/ndarray/latest/ndarray/struct.ArrayBase.html) exists.
pub struct JustifiedTable<T> {
    index_shift: usize,
    data: Vec<T>,
}

impl<T> JustifiedTable<T> {
    pub fn new() -> Self {
        JustifiedTable {
            index_shift: 0,
            data: Vec::new(),
        }
    }

    pub fn flat_iter(&self) -> Iter<T> {
        self.data.iter()
    }

    fn internal_index(&self, index: [usize; 2]) -> usize {
        // compute internal index
        let internal_index = self.index_shift * index[0] + index[1];
        // check bounds
        assert!(index[1] < self.index_shift);
        assert!(internal_index < self.data.len());
        // return internal index
        internal_index
    }

    pub fn len(&self) -> usize {
        if self.index_shift != 0 { self.data.len() / self.index_shift } else { 0 }
    }

    pub fn inner_len(&self, index: usize) -> usize {
        let _ = self.internal_index([index, 0]);  // check bounds
        self.index_shift
    }
}

impl<T> From<Vec<Vec<T>>> for JustifiedTable<T> {
    fn from(value: Vec<Vec<T>>) -> Self {
        if let Some(index_shift) = value.get(0).and_then(|v| Some(v.len())) {
            // check that value is justified
            assert!(value.iter().all(|v| v.len() == index_shift));
            // flatten data
            let data = value.into_iter().flatten().collect();
            // return table
            JustifiedTable {
                index_shift: index_shift,
                data: data,
            }
        } else {
            // value is empty, so return empty table
            JustifiedTable::new()
        }
    }
}

impl<T> Index<[usize; 2]> for JustifiedTable<T> {
    type Output = T;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        let internal_index = self.internal_index(index);
        &self.data[internal_index]
    }
}

impl<T> IndexMut<[usize; 2]> for JustifiedTable<T> {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut T {
        let internal_index = self.internal_index(index);
        &mut self.data[internal_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        // todo: add tests
    }
}
