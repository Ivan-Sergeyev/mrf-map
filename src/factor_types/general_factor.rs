#![allow(dead_code)]

use std::{
    fmt::Display,
    fs::File,
    io::{self, Write},
    ops::{Index, IndexMut},
};

use crate::{cfn::solution::Solution, CostFunctionNetwork};

use super::factor_trait::Factor;

pub struct GeneralFactor {
    pub function_table: Vec<f64>,
}

impl Factor for GeneralFactor {
    fn map(&self, mapping: fn(f64) -> f64) -> GeneralFactor {
        GeneralFactor {
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

    fn cost(&self, cfn: &CostFunctionNetwork, solution: &Solution, variables: &Vec<usize>) -> f64 {
        let mut k_factor = 1;
        let mut index = 0;
        for variable in variables.iter().rev() {
            index += k_factor
                * solution[*variable]
                    .expect("Solution is undefined on a variable involved in this factor");
            k_factor *= cfn.domain_size(*variable);
        }
        self.function_table[index]
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

impl From<Vec<f64>> for GeneralFactor {
    fn from(value: Vec<f64>) -> Self {
        GeneralFactor {
            function_table: value,
        }
    }
}
