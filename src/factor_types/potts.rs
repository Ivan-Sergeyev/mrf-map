#![allow(dead_code)]

use std::{
    fmt::Display,
    fs::File,
    io::{self, Write},
};

use crate::{
    cfn::{
        solution::Solution,
        uai::{vec_mapping_to_string, vec_to_string},
    },
    CostFunctionNetwork,
};

use super::factor_trait::Factor;

// Stores a Potts factor
pub struct Potts {
    variables: Vec<usize>,        // the two variables associated with this factor
    function_table_len: usize,    // the length of the function table that this factor expands to
    domain_sizes: (usize, usize), // the domain sizes of this factor's variables
    value: f64, // the value of the Potts factor whenever the labels of this factor's variables match
}

impl Potts {
    pub fn new(cfn: &CostFunctionNetwork, variables: Vec<usize>, value: f64) -> Self {
        assert_eq!(
            variables.len(),
            2,
            "Potts factor must be defined on exactly 2 variables."
        );
        let domain_sizes = (cfn.domain_size(variables[0]), cfn.domain_size(variables[1]));
        Potts {
            variables,
            function_table_len: domain_sizes.0 * domain_sizes.1,
            domain_sizes,
            value,
        }
    }
}

impl Factor for Potts {
    fn arity(&self) -> usize {
        2
    }

    fn function_table_len(&self) -> usize {
        self.function_table_len
    }

    fn variables(&self) -> &Vec<usize> {
        &self.variables
    }

    fn clone_function_table(&self) -> Vec<f64> {
        (0..self.domain_sizes.0)
            .zip(0..self.domain_sizes.1)
            .map(|(a, b)| (a == b) as usize as f64 * self.value)
            .collect()
    }

    fn map(&self, mapping: fn(f64) -> f64) -> Potts {
        Potts {
            variables: self.variables.clone(),
            function_table_len: self.function_table_len,
            domain_sizes: self.domain_sizes.clone(),
            value: mapping(self.value),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        mapping(&mut self.value);
    }

    fn cost(&self, _cfn: &CostFunctionNetwork, solution: &Solution) -> f64 {
        solution[self.variables[0]]
            .is_some_and(|solution_0| solution[self.variables[1]] == Some(solution_0))
            as usize as f64
            * self.value
    }

    fn write_uai(&self, file: &mut File, mapping: fn(&f64) -> f64) -> Result<(), io::Error> {
        write!(
            file,
            "\n{}\n{}\n",
            self.function_table_len(),
            vec_mapping_to_string(&self.clone_function_table(), mapping)
        )
    }
}

impl Display for Potts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", vec_to_string(&self.clone_function_table()))
    }
}
