#![allow(dead_code)]

use std::{convert::From, ops::Index};
use bitvec::prelude::*;

/// A 2-dimensional bool table stored contiguously in memory and indexed manually.
/// Ensures that each bool takes exactly one bit of memory.
/// Serves to replace Vec<Vec<bool>> in cases when all inner Vec's have the same length.
/// todo: add example usage
pub struct CompressedBitTable {
    inner_len: usize,
    data: BitVec,
}

impl CompressedBitTable {
    pub fn new() -> Self {
        CompressedBitTable {
            inner_len: 0,
            data: BitVec::new(),
        }
    }

    fn internal_index(&self, index: [usize; 2]) -> usize {
        // compute internal index
        let internal_index = self.inner_len * index[0] + index[1];
        // check bounds
        assert!(index[1] < self.inner_len);
        assert!(internal_index < self.data.len());
        // return internal index
        internal_index
    }

    pub fn get(&self, index: [usize; 2]) -> &bool {
        &self.data[self.internal_index(index)]
    }

    pub fn set(&mut self, index: [usize; 2], value: bool) {
        let idx = self.internal_index(index);
        self.data.set(idx, value);
    }

    pub fn len(&self) -> usize {
        if self.inner_len != 0 {
            self.data.len() / self.inner_len
        } else {
            0
        }
    }

    pub fn inner_len(&self) -> usize {
        self.inner_len
    }
}

impl From<Vec<Vec<bool>>> for CompressedBitTable {
    fn from(value: Vec<Vec<bool>>) -> Self {
        if let Some(index_shift) = value.get(0).and_then(|v| Some(v.len())) {
            // check that value is justified
            assert!(value.iter().all(|v| v.len() == index_shift));

            // compute total number of bits
            let data = value.into_iter().flatten().collect();

            // create table
            CompressedBitTable {
                inner_len: index_shift,
                data: data,
            }
        } else {
            // value is empty, so return empty table
            CompressedBitTable::new()
        }
    }
}

impl Index<[usize; 2]> for CompressedBitTable {
    type Output = bool;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        &self.get(index)
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
