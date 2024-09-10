#![allow(dead_code)]

use std::convert::From;
use std::mem::size_of;
use std::slice::Iter;

/// Underlying data type used to store data in CompressedBitTable.
type CompressedStorageType = u8;

/// A 2-dimensional bool table stored contiguously in memory and indexed manually.
/// Ensures that each bool takes exactly one bit of memory.
/// Serves to replace Vec<Vec<bool>> in cases when all inner Vec's have the same length.
/// todo: add example usage
/// todo: slice indexing
/// todo: store data as Box<[T]> instead of Vec<T>?
/// Note: [BitVec](https://docs.rs/bitvec/latest/bitvec/vec/struct.BitVec.html) exists.
pub struct CompressedBitTable {
    index_shift: usize,
    len: usize,
    data: Vec<CompressedStorageType>,
}

impl CompressedBitTable {
    pub fn new() -> Self {
        CompressedBitTable {
            index_shift: 0,
            len: 0,
            data: Vec::new(),
        }
    }

    pub fn flat_iter(&self) -> Iter<CompressedStorageType> {
        // warning: last element may have extra bits beyond len
        self.data.iter()
    }

    fn internal_index(&self, index: [usize; 2]) -> usize {
        // compute internal index
        let internal_index = self.index_shift * index[0] + index[1];
        // check bounds
        assert!(index[1] < self.index_shift);
        assert!(internal_index < self.len);
        // return internal index
        internal_index
    }

    fn array_bit_index(&self, index: [usize; 2]) -> (usize, usize) {
        let internal_index = self.internal_index(index);
        (
            internal_index / compressed_storage_size(),
            internal_index % compressed_storage_size(),
        )
    }

    pub fn get(&self, index: [usize; 2]) -> u8 {
        let (array_index, bit_index) = self.array_bit_index(index);
        (self.data[array_index] >> bit_index) & 1
    }

    pub fn set(&mut self, index: [usize; 2], value: u8) {
        let (array_index, bit_index) = self.array_bit_index(index);
        self.data[array_index] &= !(1 << bit_index) | (value & 1) << bit_index;
    }

    pub fn len(&self) -> usize {
        if self.index_shift != 0 {
            self.data.len() / self.index_shift
        } else {
            0
        }
    }

    pub fn inner_len(&self, index: usize) -> usize {
        let _ = self.internal_index([index, 0]); // check bounds
        self.index_shift
    }
}

/// Shorthand for size of CompressedStorageType.
fn compressed_storage_size() -> usize {
    size_of::<CompressedStorageType>()
}

fn compress_chunk(chunk: &[u8]) -> CompressedStorageType {
    // todo: implement with iter().map()?
    assert_eq!(chunk.len(), compressed_storage_size());
    let mut value = 0;
    for (bit_index, x) in chunk.iter().enumerate() {
        value |= (x & 1) << bit_index;
    }
    value
}

/// Divides a by b and rounds up if needed.
/// If a = k * b, returns k.
/// If a = k * b + c, returns k + 1.
/// todo: code examples and tests
fn div_up(a: usize, b: usize) -> usize {
    (a + (b - 1)) / b
}

/// Computes the difference between a and the next multiple of b.
/// If a = k * b, returns 0.
/// If a = k * b + c, returns b - c.
/// Equivalent to div_up(a, b) * b - a.
/// todo: code examples and tests
fn diff_to_next_multiple(a: usize, b: usize) -> usize {
    (b - 1) - (a + (b - 1)) % b
}

impl From<Vec<Vec<u8>>> for CompressedBitTable {
    fn from(value: Vec<Vec<u8>>) -> Self {
        if let Some(index_shift) = value.get(0).and_then(|v| Some(v.len())) {
            // check that value is justified
            assert!(value.iter().all(|v| v.len() == index_shift));

            // compute total number of bits
            let len = value.len() * index_shift;

            // flatten and extend value if len is not aligned
            let mut value: Vec<u8> = value.into_iter().flatten().collect();
            value.extend(vec![
                0;
                diff_to_next_multiple(len, compressed_storage_size())
            ]);

            // compress bit data
            let data = value
                .chunks_exact(compressed_storage_size())
                .map(|chunk| compress_chunk(chunk))
                .collect();

            // create table
            CompressedBitTable {
                index_shift: index_shift,
                len: len,
                data: data,
            }
        } else {
            // value is empty, so return empty table
            CompressedBitTable::new()
        }
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
