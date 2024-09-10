#![allow(dead_code)]

use ndarray::{Array, Array1, ArrayD, Ix1};
use std::fmt::Display;

pub trait Factor {
    fn arity(&self) -> usize;
    fn map(&self, mapping: fn(f64) -> f64) -> Self;
    fn map_inplace(&mut self, mapping: fn(&mut f64));

    fn new_message(&self) -> Self;
    fn clone(&self) -> Self;
}

pub struct NullaryFactor {
    value: f64,
}

impl NullaryFactor {
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Factor for NullaryFactor {
    fn arity(&self) -> usize {
        0
    }

    fn map(&self, mapping: fn(f64) -> f64) -> NullaryFactor {
        NullaryFactor {
            value: mapping(self.value),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        mapping(&mut self.value);
    }

    fn new_message(&self) -> Self {
        NullaryFactor { value: 0. }
    }

    fn clone(&self) -> Self {
        NullaryFactor { value: self.value }
    }
}

impl Display for NullaryFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub struct UnaryFactor {
    pub function_table: Array1<f64>,
}

impl Factor for UnaryFactor {
    fn arity(&self) -> usize {
        1
    }

    fn map(&self, mapping: fn(f64) -> f64) -> UnaryFactor {
        UnaryFactor {
            function_table: self.function_table.map(|&value| mapping(value)),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        self.function_table.map_inplace(mapping);
    }

    fn new_message(&self) -> Self {
        UnaryFactor {
            function_table: Array::zeros(self.function_table.raw_dim()),
        }
    }

    fn clone(&self) -> Self {
        UnaryFactor {
            function_table: self.function_table.clone(),
        }
    }
}

impl From<Vec<f64>> for UnaryFactor {
    fn from(value: Vec<f64>) -> Self {
        UnaryFactor {
            function_table: value.into(),
        }
    }
}

impl From<Array1<f64>> for UnaryFactor {
    fn from(value: Array1<f64>) -> Self {
        UnaryFactor {
            function_table: value,
        }
    }
}

impl From<ArrayD<f64>> for UnaryFactor {
    fn from(value: ArrayD<f64>) -> Self {
        UnaryFactor {
            function_table: value
                .into_dimensionality::<Ix1>()
                .expect("Function table should be 1-dimensional"),
        }
    }
}

impl Display for UnaryFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.function_table
                .iter()
                .map(|&value| value.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

pub struct GeneralFactor {
    pub function_table: ArrayD<f64>,
}

impl Factor for GeneralFactor {
    fn arity(&self) -> usize {
        self.function_table.ndim()
    }

    fn map(&self, mapping: fn(f64) -> f64) -> GeneralFactor {
        GeneralFactor {
            function_table: Array::from_shape_vec(
                self.function_table.shape(),
                self.function_table
                    .iter()
                    .map(|&value| mapping(value))
                    .collect(),
            )
            .unwrap(),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        self.function_table.map_inplace(mapping);
    }

    fn new_message(&self) -> Self {
        GeneralFactor {
            function_table: Array::zeros(self.function_table.raw_dim()),
        }
    }

    fn clone(&self) -> Self {
        GeneralFactor {
            function_table: self.function_table.clone(),
        }
    }
}

impl From<ArrayD<f64>> for GeneralFactor {
    fn from(value: ArrayD<f64>) -> Self {
        GeneralFactor {
            function_table: value,
        }
    }
}

impl Display for GeneralFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.function_table
                .iter()
                .map(|&value| value.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

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

impl Factor for FactorType {
    fn arity(&self) -> usize {
        match_factor_action!(self, factor, factor.arity())
    }

    fn map(&self, mapping: fn(f64) -> f64) -> FactorType {
        match_factor_wrapped_action!(self, factor, factor.map(mapping))
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        match_factor_action!(self, factor, factor.map_inplace(mapping))
    }

    fn new_message(&self) -> Self {
        match_factor_wrapped_action!(self, factor, factor.new_message())
    }

    fn clone(&self) -> Self {
        match_factor_wrapped_action!(self, factor, factor.clone())
    }
}
