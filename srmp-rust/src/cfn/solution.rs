#![allow(dead_code)]

use std::{fmt::Display, ops::{Index, IndexMut}};

use crate::{CostFunctionNetwork, FactorOrigin, GeneralCFN};

#[derive(Debug)]
pub struct Solution {
    labels: Vec<Option<usize>>, // indexed by variables, None = variable is unlabeled, usize = label of variable
}

impl Solution {
    pub fn new(cfn: &GeneralCFN) -> Self {
        Solution {
            labels: vec![None; cfn.num_variables()],
        }
    }

    pub fn is_fully_labeled(&self, cfn: &GeneralCFN, beta_origin: &FactorOrigin) -> bool {
        cfn.factor_variables(beta_origin)
            .iter()
            .all(|variable| self.labels[*variable].is_some())
    }

    pub fn num_labeled(&self, cfn: &GeneralCFN, beta_origin: &FactorOrigin) -> usize {
        cfn.factor_variables(beta_origin)
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

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.labels)
    }
}
