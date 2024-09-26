#![allow(dead_code)]

use std::{
    fmt::Display,
    fs::File,
    io::{self, Write},
};

use crate::cfn::solution::Solution;
use crate::{
    cfn::uai::{vec_mapping_to_string, vec_to_string},
    CostFunctionNetwork,
};

use super::factor_trait::Factor;

// Stores the factor as a complete function table
pub struct FunctionTable {
    variables: Vec<usize>, // the variables associated with this factor
    strides: Vec<usize>,   // the offsets used for indexing in the function table
    value: Vec<f64>,       // the function table itself
}

impl FunctionTable {
    // Initializes
    pub fn new(cfn: &CostFunctionNetwork, variables: Vec<usize>, value: Vec<f64>) -> Self {
        let mut strides = vec![1; variables.len()];
        for index in 1..variables.len() {
            strides[index] = strides[index - 1] * cfn.domain_size(variables[index]);
        }

        FunctionTable {
            variables,
            strides,
            value,
        }
    }
}

impl Factor for FunctionTable {
    fn arity(&self) -> usize {
        self.variables.len()
    }

    fn function_table_len(&self) -> usize {
        self.value.len()
    }

    fn variables(&self) -> &Vec<usize> {
        &self.variables
    }

    fn clone_function_table(&self) -> Vec<f64> {
        self.value.clone()
    }

    fn map(&self, mapping: fn(f64) -> f64) -> FunctionTable {
        FunctionTable {
            variables: self.variables.clone(),
            strides: self.strides.clone(),
            value: self.value.iter().map(|value| mapping(*value)).collect(),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        self.value.iter_mut().for_each(mapping);
    }

    fn cost(&self, _cfn: &CostFunctionNetwork, solution: &Solution) -> f64 {
        let mut index = 0;
        for (variable_index, variable) in self.variables.iter().rev().enumerate() {
            index += self.strides[variable_index]
                * solution[*variable]
                    .expect("Solution is undefined on a variable involved in this factor");
        }
        self.value[index]
    }

    fn write_uai(&self, file: &mut File, mapping: fn(&f64) -> f64) -> Result<(), io::Error> {
        write!(
            file,
            "\n{}\n{}\n",
            self.value.len(),
            vec_mapping_to_string(&self.value, mapping)
        )
    }
}

impl Display for FunctionTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", vec_to_string(&self.value))
    }
}
