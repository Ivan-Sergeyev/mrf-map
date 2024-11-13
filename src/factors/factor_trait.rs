#![allow(dead_code)]

use std::{fmt::Display, fs::File, io};

use crate::{cfn::solution::Solution, CostFunctionNetwork};

// Interface for factors in a cost function network
pub trait Factor: Display {
    // Returns the factors' arity
    fn arity(&self) -> usize;

    // Returns the length of a complete function table that this factor expands to
    fn function_table_len(&self) -> usize;

    // Returns the variables associated with this factor
    fn variables(&self) -> &Vec<usize>;

    // Returns the complete funciton table that this factor expands to
    fn clone_function_table(&self) -> Vec<f64>;

    // Applies the given mapping to the factor and returns the result
    fn map(&self, mapping: fn(f64) -> f64) -> Self;

    // Modifies the factor in-place using the given mapping
    fn map_inplace(&mut self, mapping: fn(&mut f64));

    // Returns the cost that this factor incurs in the given cost function network for the given solution
    fn cost(&self, cfn: &CostFunctionNetwork, solution: &Solution) -> f64;

    // Outputs the factor in UAI format to the given file after applying the given mapping to it
    fn write_uai(&self, file: &mut File, mapping: fn(&f64) -> f64) -> Result<(), io::Error>;
}
