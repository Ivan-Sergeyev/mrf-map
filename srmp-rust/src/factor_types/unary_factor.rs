#![allow(dead_code)]

use ndarray::{Array, Array1, ArrayD, Ix1};
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{cfn::solution::Solution, GeneralCFN};

use super::factor_trait::Factor;

pub struct UnaryFactor {
    pub function_table: Array1<f64>,
}

impl Factor for UnaryFactor {
    fn arity(&self) -> usize {
        1
    }

    fn function_table_len(&self) -> usize {
        self.function_table.len()
    }

    fn map(&self, mapping: fn(f64) -> f64) -> UnaryFactor {
        UnaryFactor {
            function_table: self.function_table.map(|&value| mapping(value)),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        self.function_table.map_inplace(mapping);
    }

    fn new_zero_message(&self) -> Self {
        UnaryFactor {
            function_table: Array::zeros(self.function_table.raw_dim()),
        }
    }

    fn clone_for_message_passing(&self) -> Self {
        UnaryFactor {
            function_table: self.function_table.clone(),
        }
    }

    fn get_cost(&self, _cfn: &GeneralCFN, solution: &Solution, variables: &Vec<usize>) -> f64 {
        self.function_table[solution[variables[0]].expect("Solution is undefined on a variable involved in this factor")]
    }
}

impl Display for UnaryFactor {
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

impl Index<usize> for UnaryFactor {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.function_table[index]
    }
}

impl IndexMut<usize> for UnaryFactor {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.function_table[index]
    }
}

impl From<Vec<f64>> for UnaryFactor {
    fn from(value: Vec<f64>) -> Self {
        UnaryFactor {
            function_table: value.into(),
        }
    }
}

impl From<Array1<f64>> for UnaryFactor {
    fn from(value: Array1<f64>) -> Self {
        UnaryFactor {
            function_table: value,
        }
    }
}

impl From<ArrayD<f64>> for UnaryFactor {
    fn from(value: ArrayD<f64>) -> Self {
        UnaryFactor {
            function_table: value
                .into_dimensionality::<Ix1>()
                .expect("Function table should be 1-dimensional"),
        }
    }
}
