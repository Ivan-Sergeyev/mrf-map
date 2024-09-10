#![allow(dead_code)]

use ndarray::{Array, ArrayD};
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{cfn::solution::Solution, CostFunctionNetwork, GeneralCFN};

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

    fn get_cost(&self, cfn: &GeneralCFN, solution: &Solution, variables: &Vec<usize>) -> f64 {
        let mut k_factor = 1;
        let mut index = 0;
        for variable in variables.iter().rev() {
            index += k_factor * solution[*variable].expect("Solution is undefined on a variable involved in this factor");
            k_factor *= cfn.domain_size(*variable);
        }
        self.function_table[index]
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
