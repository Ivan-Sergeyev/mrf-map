#![allow(dead_code)]

use core::panic;
use std::{
    fmt::Display,
    fs::File,
    io,
    ops::{Index, IndexMut},
};

use super::{factor_trait::Factor, general_factor::GeneralFactor, unary_factor::UnaryFactor};

pub enum FactorType {
    Unary(UnaryFactor),
    General(GeneralFactor),
}

// todo: macro to implement "Into"
impl<'a> Into<&'a UnaryFactor> for &'a FactorType {
    fn into(self) -> &'a UnaryFactor {
        match self {
            FactorType::Unary(unary_factor) => unary_factor,
            _ => panic!("Trying to convert FactorType to UnaryFactor, but it has a different type"),
        }
    }
}

impl<'a> Into<&'a mut UnaryFactor> for &'a mut FactorType {
    fn into(self) -> &'a mut UnaryFactor {
        match self {
            FactorType::Unary(unary_factor) => unary_factor,
            _ => panic!("Trying to convert FactorType to UnaryFactor, but it has a different type"),
        }
    }
}

impl<'a> Into<&'a GeneralFactor> for &'a FactorType {
    fn into(self) -> &'a GeneralFactor {
        match self {
            FactorType::General(general_factor) => general_factor,
            _ => {
                panic!("Trying to convert FactorType to GeneralFactor, but it has a different type")
            }
        }
    }
}

impl<'a> Into<&'a mut GeneralFactor> for &'a mut FactorType {
    fn into(self) -> &'a mut GeneralFactor {
        match self {
            FactorType::General(general_factor) => general_factor,
            _ => {
                panic!("Trying to convert FactorType to GeneralFactor, but it has a different type")
            }
        }
    }
}

macro_rules! match_factor_action {
    ($factor_type:ident, $factor_match:ident, $action:expr) => {
        match $factor_type {
            FactorType::Unary($factor_match) => $action,
            FactorType::General($factor_match) => $action,
        }
    };
}

macro_rules! match_factor_wrapped_action {
    ($factor_type:ident, $factor_match:ident, $action:expr) => {
        match $factor_type {
            FactorType::Unary($factor_match) => FactorType::Unary($action),
            FactorType::General($factor_match) => FactorType::General($action),
        }
    };
}

impl Factor for FactorType {
    fn map(&self, mapping: fn(f64) -> f64) -> FactorType {
        match_factor_wrapped_action!(self, factor, factor.map(mapping))
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        match_factor_action!(self, factor, factor.map_inplace(mapping))
    }

    fn cost(
        &self,
        cfn: &crate::CostFunctionNetwork,
        solution: &crate::cfn::solution::Solution,
        variables: &Vec<usize>,
    ) -> f64 {
        match_factor_action!(self, factor, factor.cost(cfn, solution, variables))
    }

    fn write_uai(&self, file: &mut File, mapping: fn(f64) -> f64) -> Result<(), io::Error> {
        match_factor_action!(self, factor, factor.write_uai(file, mapping))
    }
}

impl Display for FactorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match_factor_action!(self, factor, factor.fmt(f))
    }
}

impl Index<usize> for FactorType {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match_factor_action!(self, factor, &factor[index])
    }
}

impl IndexMut<usize> for FactorType {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match_factor_action!(self, factor, &mut factor[index])
    }
}
