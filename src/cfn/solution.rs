#![allow(dead_code)]

use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{CostFunctionNetwork, FactorOrigin};

pub struct Solution {
    labels: Vec<Option<usize>>, // indexed by variables, None = variable is unlabeled, usize = label of variable
}

impl Solution {
    // Creates a new solution with each variable unassigned
    pub fn new(cfn: &CostFunctionNetwork) -> Self {
        Solution {
            labels: vec![None; cfn.num_variables()],
        }
    }

    // Checks if every variable is labeled
    pub fn is_fully_labeled(&self, cfn: &CostFunctionNetwork, factor: &FactorOrigin) -> bool {
        // Assumption: `factor` is from `cfn`
        cfn.factor_variables(factor)
            .iter()
            .all(|variable| self.labels[*variable].is_some())
    }

    // Returns number of labeled variables
    pub fn num_labeled(&self, cfn: &CostFunctionNetwork, factor: &FactorOrigin) -> usize {
        // Assumption: `factor` is from `cfn`
        cfn.factor_variables(factor)
            .iter()
            .fold(0, |num_labeled, variable| {
                num_labeled + self.labels[*variable].is_some() as usize
            })
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

fn label_to_str(label: Option<usize>) -> String {
    match label {
        Some(label) => label.to_string(),
        None => "None".to_string(),
    }
}

impl std::fmt::Debug for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            self.labels
                .iter()
                .map(|label| label_to_str(*label))
                .collect::<Vec<_>>()
        )
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            self.labels
                .iter()
                .map(|label| label_to_str(*label))
                .collect::<Vec<_>>()
        )
    }
}
