#![allow(dead_code)]

use std::{fmt::Display, fs::File, io};

use crate::{CostFunctionNetwork, Solution};

use super::{
    factor_trait::Factor, function_table::FunctionTable, potts::Potts,
    uniform_constant::UniformConstant,
};

pub enum FactorType {
    FunctionTable(FunctionTable),
    UniformConstant(UniformConstant),
    Potts(Potts),
}

macro_rules! match_factor_action {
    ($factor_type:ident, $factor_match:ident, $action:expr) => {
        match $factor_type {
            FactorType::FunctionTable($factor_match) => $action,
            FactorType::UniformConstant($factor_match) => $action,
            FactorType::Potts($factor_match) => $action,
        }
    };
}

macro_rules! match_factor_wrapped_action {
    ($factor_type:ident, $factor_match:ident, $action:expr) => {
        match $factor_type {
            FactorType::FunctionTable($factor_match) => FactorType::FunctionTable($action),
            FactorType::UniformConstant($factor_match) => FactorType::UniformConstant($action),
            FactorType::Potts($factor_match) => FactorType::Potts($action),
        }
    };
}

impl Factor for FactorType {
    fn arity(&self) -> usize {
        match_factor_action!(self, factor, factor.arity())
    }

    fn function_table_len(&self) -> usize {
        match_factor_action!(self, factor, factor.function_table_len())
    }

    fn variables(&self) -> &Vec<usize> {
        match_factor_action!(self, factor, factor.variables())
    }

    fn clone_function_table(&self) -> Vec<f64> {
        match_factor_action!(self, factor, factor.clone_function_table())
    }

    fn map(&self, mapping: fn(f64) -> f64) -> FactorType {
        match_factor_wrapped_action!(self, factor, factor.map(mapping))
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        match_factor_action!(self, factor, factor.map_inplace(mapping))
    }

    fn cost(&self, cfn: &CostFunctionNetwork, solution: &Solution) -> f64 {
        match_factor_action!(self, factor, factor.cost(cfn, solution))
    }

    fn write_uai(&self, file: &mut File, mapping: fn(&f64) -> f64) -> Result<(), io::Error> {
        match_factor_action!(self, factor, factor.write_uai(file, mapping))
    }
}

impl Display for FactorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match_factor_action!(self, factor, factor.fmt(f))
    }
}
