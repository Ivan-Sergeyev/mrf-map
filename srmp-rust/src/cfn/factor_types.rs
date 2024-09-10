#![allow(dead_code)]

use core::panic;
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

pub mod factor_trait;
use factor_trait::Factor;

pub mod general_factor;
use general_factor::GeneralFactor;

pub mod nullary_factor;
use nullary_factor::NullaryFactor;

pub mod unary_factor;
use unary_factor::UnaryFactor;

pub enum FactorType {
    Nullary(NullaryFactor),
    Unary(UnaryFactor),
    General(GeneralFactor),
}

macro_rules! match_factor_action {
    ($factor_type:ident, $factor_match:ident, $action:expr) => {
        match $factor_type {
            FactorType::Nullary($factor_match) => $action,
            FactorType::Unary($factor_match) => $action,
            FactorType::General($factor_match) => $action,
        }
    };
}

macro_rules! match_factor_wrapped_action {
    ($factor_type:ident, $factor_match:ident, $action:expr) => {
        match $factor_type {
            FactorType::Nullary($factor_match) => FactorType::Nullary($action),
            FactorType::Unary($factor_match) => FactorType::Unary($action),
            FactorType::General($factor_match) => FactorType::General($action),
        }
    };
}

macro_rules! match_two_factors_same_type_action {
    ($factor_one:ident, $factor_one_match:ident, $factor_two:ident, $factor_two_match:ident, $action:expr) => {
        match $factor_one {
            FactorType::Nullary($factor_one_match) => match $factor_two {
                FactorType::Nullary($factor_two_match) => $action,
                FactorType::Unary(_) => panic!("Factor types don't match"),
                FactorType::General(_) => panic!("Factor types don't match"),
            },
            FactorType::Unary($factor_one_match) => match $factor_two {
                FactorType::Nullary(_) => panic!("Factor types don't match"),
                FactorType::Unary($factor_two_match) => $action,
                FactorType::General(_) => panic!("Factor types don't match"),
            },
            FactorType::General($factor_one_match) => match $factor_two {
                FactorType::Nullary(_) => panic!("Factor types don't match"),
                FactorType::Unary(_) => panic!("Factor types don't match"),
                FactorType::General($factor_two_match) => $action,
            },
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

    fn map(&self, mapping: fn(f64) -> f64) -> FactorType {
        match_factor_wrapped_action!(self, factor, factor.map(mapping))
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        match_factor_action!(self, factor, factor.map_inplace(mapping))
    }

    fn new_zero_message(&self) -> Self {
        match_factor_wrapped_action!(self, factor, factor.new_zero_message())
    }

    fn clone_for_message_passing(&self) -> Self {
        match_factor_wrapped_action!(self, factor, factor.clone_for_message_passing())
    }

    fn add_assign(&mut self, rhs: &Self) {
        match_two_factors_same_type_action!(
            self,
            factor,
            rhs,
            rhs_factor,
            factor.add_assign(rhs_factor)
        );
    }

    fn sub_assign(&mut self, rhs: &Self) {
        match_two_factors_same_type_action!(
            self,
            factor,
            rhs,
            rhs_factor,
            factor.sub_assign(rhs_factor)
        );
    }

    fn mul_assign(&mut self, rhs: f64) {
        match_factor_action!(self, factor, factor.mul_assign(rhs));
    }

    fn add_assign_number(&mut self, rhs: f64) {
        match_factor_action!(self, factor, factor.add_assign_number(rhs));
    }

    fn min(&self) -> f64 {
        match_factor_action!(self, factor, factor.min())
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
