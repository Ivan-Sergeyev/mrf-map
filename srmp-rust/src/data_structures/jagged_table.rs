#![allow(dead_code)]

use std::convert::From;
use std::ops::{Index, IndexMut};
use std::slice::Iter;

/// A 2-dimensional table stored contiguously in memory and indexed manually.
/// Serves to replace Vec<Vec<T>> in cases when inner Vec's might have different lengths.
/// todo: add example usage
/// todo: slice indexing
/// todo: store data as Box<[T]> instead of Vec<T>?
pub struct JaggedTable<T> {
    index_shift: Vec<usize>,
    data: Vec<T>,
}

impl<T> JaggedTable<T> {
    pub fn new() -> Self {
        JaggedTable {
            index_shift: vec![0; 1],
            data: Vec::new(),
        }
    }

    pub fn flat_iter(&self) -> Iter<T> {
        self.data.iter()
    }

    fn internal_index(&self, index: [usize; 2]) -> usize {
        // compute internal index
        let internal_index = self.index_shift[index[0]] + index[1];
        // check bounds
        assert!(index[0] < self.len());
        assert!(internal_index < self.index_shift[index[0] + 1]);
        // return internal index
        internal_index
    }

    pub fn len(&self) -> usize {
        self.index_shift.len() - 1
    }

    pub fn inner_len(&self, index: usize) -> usize {
        let _ = self.internal_index([index, 0]); // check bounds
        self.index_shift[index + 1] - self.index_shift[index]
    }
}

impl<T> From<Vec<Vec<T>>> for JaggedTable<T> {
    fn from(value: Vec<Vec<T>>) -> Self {
        // precompute index shifts
        let mut index_shift = Vec::with_capacity(value.len() + 1);
        index_shift.push(0);
        for i in 0..value.len() {
            index_shift.push(index_shift[i] + value[i].len());
        }
        // flatten data
        let data = value.into_iter().flatten().collect();
        // return table
        JaggedTable {
            index_shift: index_shift,
            data: data,
        }
    }
}

impl<T> Index<[usize; 2]> for JaggedTable<T> {
    type Output = T;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        let internal_index = self.internal_index(index);
        &self.data[internal_index]
    }
}

impl<T> IndexMut<[usize; 2]> for JaggedTable<T> {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut T {
        let internal_index = self.internal_index(index);
        &mut self.data[internal_index]
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn new() {
    //     // todo: add tests
    // }
}
