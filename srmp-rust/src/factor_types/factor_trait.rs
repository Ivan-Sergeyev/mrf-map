#![allow(dead_code)]

use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{cfn::solution::Solution, GeneralCFN};

pub trait Factor: Display + Index<usize> + IndexMut<usize> {
    fn arity(&self) -> usize; // todo: move to CFN
    fn function_table_len(&self) -> usize; // todo: replicate for CFN, distinguish "own" len (representation) vs "max" len (product of domain sizes)

    fn map(&self, mapping: fn(f64) -> f64) -> Self;
    fn map_inplace(&mut self, mapping: fn(&mut f64));

    fn new_zero_message(&self) -> Self; // todo: update to return message? constructor for messages?
    fn clone_for_message_passing(&self) -> Self; // same here

    fn get_cost(&self, cfn: &GeneralCFN, solution: &Solution, variables: &Vec<usize>) -> f64;
}
