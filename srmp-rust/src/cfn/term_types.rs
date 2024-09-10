#![allow(dead_code)]

// todo: implement additional term types (Potts, ...)

use std::fmt::Display;

use ndarray::{Array, Array1, ArrayD, Ix1};

pub trait TermType {
    fn arity(&self) -> usize;
    fn map(&self, mapping: fn(f64) -> f64) -> Self;
    fn map_inplace(&mut self, mapping: fn(&mut f64));

    fn new_message(&self) -> Self;
}

pub struct NullaryTerm {
    value: f64,
}

impl NullaryTerm {
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl TermType for NullaryTerm {
    fn arity(&self) -> usize {
        0
    }

    fn map(&self, mapping: fn(f64) -> f64) -> NullaryTerm {
        NullaryTerm {
            value: mapping(self.value),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        mapping(&mut self.value);
    }

    fn new_message(&self) -> Self {
        NullaryTerm { value: 0. }
    }
}

impl Display for NullaryTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub struct UnaryTerm {
    pub function_table: Array1<f64>,
}

impl TermType for UnaryTerm {
    fn arity(&self) -> usize {
        1
    }

    fn map(&self, mapping: fn(f64) -> f64) -> UnaryTerm {
        UnaryTerm {
            function_table: self.function_table.map(|&value| mapping(value)),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        self.function_table.map_inplace(mapping);
    }

    fn new_message(&self) -> Self {
        UnaryTerm {
            function_table: Array::zeros(self.function_table.raw_dim()),
        }
    }
}

impl From<Vec<f64>> for UnaryTerm {
    fn from(value: Vec<f64>) -> Self {
        UnaryTerm {
            function_table: value.into(),
        }
    }
}

impl From<ArrayD<f64>> for UnaryTerm {
    fn from(value: ArrayD<f64>) -> Self {
        UnaryTerm {
            function_table: value
                .into_dimensionality::<Ix1>()
                .expect("Function table should be 1-dimensional"),
        }
    }
}

impl Display for UnaryTerm {
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

pub struct GeneralTerm {
    pub function_table: ArrayD<f64>,
}

impl TermType for GeneralTerm {
    fn arity(&self) -> usize {
        self.function_table.ndim()
    }

    fn map(&self, mapping: fn(f64) -> f64) -> GeneralTerm {
        GeneralTerm {
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
        GeneralTerm {
            function_table: Array::zeros(self.function_table.raw_dim()),
        }
    }
}

impl From<ArrayD<f64>> for GeneralTerm {
    fn from(value: ArrayD<f64>) -> Self {
        GeneralTerm {
            function_table: value,
        }
    }
}

impl Display for GeneralTerm {
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

pub enum Term {
    Nullary(NullaryTerm),
    Unary(UnaryTerm),
    General(GeneralTerm),
}

impl TermType for Term {
    fn arity(&self) -> usize {
        match self {
            Term::Nullary(term) => term.arity(),
            Term::Unary(term) => term.arity(),
            Term::General(term) => term.arity(),
        }
    }

    fn map(&self, mapping: fn(f64) -> f64) -> Term {
        match self {
            Term::Nullary(term) => Term::Nullary(term.map(mapping)),
            Term::Unary(term) => Term::Unary(term.map(mapping)),
            Term::General(term) => Term::General(term.map(mapping)),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        match self {
            Term::Nullary(term) => term.map_inplace(mapping),
            Term::Unary(term) => term.map_inplace(mapping),
            Term::General(term) => term.map_inplace(mapping),
        }
    }

    fn new_message(&self) -> Self {
        match self {
            Term::Nullary(term) => Term::Nullary(term.new_message()),
            Term::Unary(term) => Term::Unary(term.new_message()),
            Term::General(term) => Term::General(term.new_message()),
        }
    }
}
