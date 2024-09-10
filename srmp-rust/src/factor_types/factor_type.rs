#![allow(dead_code)]

use core::panic;
use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use super::{
    factor_trait::Factor, general_factor::GeneralFactor, nullary_factor::NullaryFactor,
    unary_factor::UnaryFactor,
};

pub enum FactorType {
    Nullary(NullaryFactor),
    Unary(UnaryFactor),
    General(GeneralFactor),
}

// todo: macro to implement "Into"
impl<'a> Into<&'a NullaryFactor> for &'a FactorType {
    fn into(self) -> &'a NullaryFactor {
        match self {
            FactorType::Nullary(nullary_factor) => nullary_factor,
            _ => {
                panic!("Trying to convert FactorType to NullaryFactor, but it has a different type")
            }
        }
    }
}

impl<'a> Into<&'a mut NullaryFactor> for &'a mut FactorType {
    fn into(self) -> &'a mut NullaryFactor {
        match self {
            FactorType::Nullary(nullary_factor) => nullary_factor,
            _ => {
                panic!("Trying to convert FactorType to NullaryFactor, but it has a different type")
            }
        }
    }
}

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
        match_factor_action!(self, factor, factor.add_assign(rhs.into()));
    }

    fn sub_assign(&mut self, rhs: &Self) {
        match_factor_action!(self, factor, factor.sub_assign(rhs.into()));
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

    fn max(&self) -> f64 {
        match_factor_action!(self, factor, factor.max())
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
