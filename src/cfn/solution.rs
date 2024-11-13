#![allow(dead_code)]

use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{factors::factor_trait::Factor, CostFunctionNetwork};

use super::uai::option_to_string;

// Stores a solution to a cost function network
pub struct Solution {
    labels: Vec<Option<usize>>, // indexed by variables, None = variable is unlabeled, Some(usize) = variable's label
}

impl Solution {
    // Creates a new solution for a given cost function network with each variable unassigned
    pub fn new(cfn: &CostFunctionNetwork) -> Self {
        Solution {
            labels: vec![None; cfn.num_variables()],
        }
    }

    // Checks if every variable in a given Vec is labeled
    pub fn is_fully_labeled(&self, variables: &Vec<usize>) -> bool {
        variables
            .iter()
            .all(|variable| self.labels[*variable].is_some())
    }

    // Returns number of labeled variables in a given Vec
    pub fn num_labeled(&self, variables: &Vec<usize>) -> usize {
        variables.iter().fold(0, |num_labeled, variable| {
            num_labeled + self.labels[*variable].is_some() as usize
        })
    }

    // Returns a Vec of Strings encoding the labels
    fn labels_to_vec_string(&self) -> Vec<String> {
        self.labels
            .iter()
            .map(|label| option_to_string(*label))
            .collect::<Vec<_>>()
    }

    // Returns the solution's cost with respect to a given cost function network
    pub fn cost(&self, cfn: &CostFunctionNetwork) -> f64 {
        cfn.factors_iter()
            .map(|factor| factor.cost(cfn, self))
            .sum()
    }
}

impl Index<usize> for Solution {
    type Output = Option<usize>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.labels[index]
    }
}

impl IndexMut<usize> for Solution {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.labels[index]
    }
}

impl std::fmt::Debug for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.labels_to_vec_string())
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.labels_to_vec_string())
    }
}

impl From<Vec<Option<usize>>> for Solution {
    fn from(value: Vec<Option<usize>>) -> Self {
        Solution { labels: value }
    }
}
