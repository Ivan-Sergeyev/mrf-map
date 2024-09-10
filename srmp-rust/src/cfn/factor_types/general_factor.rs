#![allow(dead_code)]

use ndarray::{Array, ArrayD};
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Index, IndexMut, MulAssign, SubAssign},
};

use super::factor_trait::Factor;

pub struct GeneralFactor {
    pub function_table: ArrayD<f64>,
}

impl Factor for GeneralFactor {
    fn arity(&self) -> usize {
        self.function_table.ndim()
    }

    fn function_table_len(&self) -> usize {
        self.function_table.len()
    }

    fn map(&self, mapping: fn(f64) -> f64) -> GeneralFactor {
        GeneralFactor {
            function_table: Array::from_shape_vec(
                self.function_table.shape(),
                self.function_table
                    .iter()
                    .map(|&value| mapping(value))
                    .collect(),
            )
            .unwrap(),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        self.function_table.map_inplace(mapping);
    }

    fn new_zero_message(&self) -> Self {
        GeneralFactor {
            function_table: Array::zeros(self.function_table.raw_dim()),
        }
    }

    fn clone_for_message_passing(&self) -> Self {
        GeneralFactor {
            function_table: self.function_table.clone(),
        }
    }

    fn add_assign(&mut self, rhs: &Self) {
        self.function_table.add_assign(&rhs.function_table);
    }

    fn sub_assign(&mut self, rhs: &Self) {
        self.function_table.sub_assign(&rhs.function_table);
    }

    fn mul_assign(&mut self, rhs: f64) {
        self.function_table.mul_assign(rhs);
    }

    fn add_assign_number(&mut self, rhs: f64) {
        for elem in self.function_table.iter_mut() {
            elem.add(rhs);
        }
    }

    fn min(&self) -> f64 {
        *self.function_table.iter().min_by(|a, b| a.total_cmp(b)).unwrap()
    }
}

impl Display for GeneralFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.function_table
                .iter()
                .map(|&value| value.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

impl Index<usize> for GeneralFactor {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.function_table[index]
    }
}

impl IndexMut<usize> for GeneralFactor {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.function_table[index]
    }
}

impl From<ArrayD<f64>> for GeneralFactor {
    fn from(value: ArrayD<f64>) -> Self {
        GeneralFactor {
            function_table: value,
        }
    }
}
