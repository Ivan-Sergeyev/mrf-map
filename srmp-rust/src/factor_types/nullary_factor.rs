#![allow(dead_code)]

use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use super::factor_trait::Factor;

pub struct NullaryFactor {
    value: f64,
}

impl NullaryFactor {
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Factor for NullaryFactor {
    fn arity(&self) -> usize {
        0
    }

    fn function_table_len(&self) -> usize {
        1
    }

    fn map(&self, mapping: fn(f64) -> f64) -> NullaryFactor {
        NullaryFactor {
            value: mapping(self.value),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        mapping(&mut self.value);
    }

    fn new_zero_message(&self) -> Self {
        NullaryFactor { value: 0. }
    }

    fn clone_for_message_passing(&self) -> Self {
        NullaryFactor { value: self.value }
    }

    fn add_assign(&mut self, rhs: &Self) {
        self.value += rhs.value;
    }

    fn sub_assign(&mut self, rhs: &Self) {
        self.value -= rhs.value;
    }

    fn mul_assign(&mut self, rhs: f64) {
        self.value *= rhs;
    }

    fn add_assign_number(&mut self, rhs: f64) {
        self.value += rhs;
    }

    fn min(&self) -> f64 {
        self.value
    }

    fn max(&self) -> f64 {
        self.value
    }
}

impl Display for NullaryFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Index<usize> for NullaryFactor {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert_eq!(index, 0);
        &self.value
    }
}

impl IndexMut<usize> for NullaryFactor {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert_eq!(index, 0);
        &mut self.value
    }
}
