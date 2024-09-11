#![allow(dead_code)]

use std::{
    fmt::Display,
    fs::File,
    io,
    ops::{Index, IndexMut},
};

use crate::{cfn::solution::Solution, CostFunctionNetwork};

pub trait Factor: Display + Index<usize> + IndexMut<usize> {
    fn map(&self, mapping: fn(f64) -> f64) -> Self;
    fn map_inplace(&mut self, mapping: fn(&mut f64));

    fn cost(&self, cfn: &CostFunctionNetwork, solution: &Solution, variables: &Vec<usize>) -> f64;

    fn write_uai(&self, file: &mut File, mapping: fn(f64) -> f64) -> Result<(), io::Error>;
}
