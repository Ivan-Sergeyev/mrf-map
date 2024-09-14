#![allow(dead_code)]

use std::{
    fmt::Display,
    fs::File,
    io::{self, Write},
};

use crate::{cfn::uai::repeat_float_to_string, CostFunctionNetwork, Solution};

use super::factor_trait::Factor;

pub struct UniformConstant {
    variables: Vec<usize>,
    function_table_len: usize,
    value: f64,
}

impl UniformConstant {
    pub fn new(variables: Vec<usize>, function_table_len: usize, value: f64) -> Self {
        UniformConstant {
            variables,
            function_table_len,
            value,
        }
    }
}

impl Factor for UniformConstant {
    fn arity(&self) -> usize {
        self.variables.len()
    }

    fn function_table_len(&self) -> usize {
        self.function_table_len
    }

    fn variables(&self) -> &Vec<usize> {
        &self.variables
    }

    fn clone_function_table(&self) -> Vec<f64> {
        vec![self.value; self.function_table_len]
    }

    fn map(&self, mapping: fn(f64) -> f64) -> UniformConstant {
        UniformConstant {
            variables: self.variables.clone(),
            function_table_len: self.function_table_len,
            value: mapping(self.value),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        mapping(&mut self.value);
    }

    fn cost(&self, _cfn: &CostFunctionNetwork, solution: &Solution) -> f64 {
        for variable in &self.variables {
            solution[*variable]
                .expect("Solution is undefined on a variable involved in this factor");
        }
        self.value
    }

    fn write_uai(&self, file: &mut File, mapping: fn(&f64) -> f64) -> Result<(), io::Error> {
        let var_name = write!(
            file,
            "\n{}\n{}\n",
            self.function_table_len,
            repeat_float_to_string(self.function_table_len, mapping(&self.value))
        );
        var_name
    }
}

impl Display for UniformConstant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            repeat_float_to_string(self.function_table_len, self.value)
        )
    }
}
