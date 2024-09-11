#![allow(dead_code)]

use std::{
    fmt::Display,
    fs::File,
    io::{self, Write},
    ops::{Index, IndexMut},
};

use crate::{cfn::solution::Solution, CostFunctionNetwork};

use super::factor_trait::Factor;

pub struct UnaryFactor {
    pub function_table: Vec<f64>,
}

impl Factor for UnaryFactor {
    fn map(&self, mapping: fn(f64) -> f64) -> UnaryFactor {
        UnaryFactor {
            function_table: self
                .function_table
                .iter()
                .map(|value| mapping(*value))
                .collect(),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        self.function_table.iter_mut().for_each(mapping);
    }

    fn cost(&self, _cfn: &CostFunctionNetwork, solution: &Solution, variables: &Vec<usize>) -> f64 {
        self.function_table[solution[variables[0]]
            .expect("Solution is undefined on a variable involved in this factor")]
    }

    fn write_uai(&self, file: &mut File, mapping: fn(f64) -> f64) -> Result<(), io::Error> {
        write!(
            file,
            "\n{}\n{}\n",
            self.function_table.len(),
            self.map(mapping).to_string()
        )
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
            function_table: value,
        }
    }
}
