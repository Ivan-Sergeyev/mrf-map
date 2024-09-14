#![allow(dead_code)]

use std::{fmt::Display, fs::File, io};

use crate::{CostFunctionNetwork, Solution};

pub trait Factor: Display {
    fn arity(&self) -> usize;
    fn function_table_len(&self) -> usize;
    fn variables(&self) -> &Vec<usize>;

    fn clone_function_table(&self) -> Vec<f64>;

    fn map(&self, mapping: fn(f64) -> f64) -> Self;
    fn map_inplace(&mut self, mapping: fn(&mut f64));

    fn cost(&self, cfn: &CostFunctionNetwork, solution: &Solution) -> f64;

    fn write_uai(&self, file: &mut File, mapping: fn(&f64) -> f64) -> Result<(), io::Error>;
}
