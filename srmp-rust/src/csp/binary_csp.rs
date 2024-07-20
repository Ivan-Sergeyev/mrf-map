#![allow(dead_code)]

use crate::data_structures::compressed_bit_table::CompressedBitTable;
use crate::data_structures::jagged_table::JaggedTable;

/// implementation error: BinaryCSP works only if all domain sizes are the same
/// to fix:
/// 1. implement JaggedBitArrayD (or temporarily use ArrayD<bool>, then upgrade),
/// 2. rewrite BinaryCSP analogous to CFN (need to re-use connection graph to save memory => BoolCSP rather than general BinaryCSP)

/// A data structure for working with binary constraint satisfaction problems
///
/// convention for storing constraints: 1 = consistent, 0 = inconsistent, None (binary only) = no constraint = consistent
///
/// unary constraints are indexed by variable, then label
///
/// binary constraints are indexed by two variables (var_x, var_y) in upper-triangular fashion:
/// * var_x == var_y is impossible (because not binary),
/// * var_x > var_y is swapped to ensure var_x <= var_y (because order doesn't matter),
/// * var_y is replaced by var_y - var_x - 1, because all previous entries (with var_y' <= var_x) do not exist
///
/// todo: rewrite using BitVec instead of CompressedBitTable
/// todo: avoid flipping variable order when accessing binary constraints?
/// -- re-implement using Rc/Box?
pub struct BinaryCSP {
    unary_constraints: CompressedBitTable,
    binary_constraints: JaggedTable<Option<CompressedBitTable>>,
}

fn empty_binary_constraints(num_variables: usize) -> JaggedTable<Option<CompressedBitTable>> {
    (0..num_variables)
        .map(|var| {
            std::iter::repeat_with(|| None)
                .take(num_variables - var - 1)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<_>>>()
        .into()
}

impl BinaryCSP {
    pub fn new(domain_sizes: &Vec<usize>) -> Self {
        // initializes binary CSP with consistent unary constraints and no binary constraints
        let num_variables = domain_sizes.len();
        let unary_constraints = (0..num_variables)
            .map(|var| vec![true; domain_sizes[var]])
            .collect::<Vec<Vec<bool>>>();
        BinaryCSP {
            unary_constraints: unary_constraints.into(),
            binary_constraints: empty_binary_constraints(num_variables),
        }
    }

    pub fn from_unary_constraints(unary_constraints: Vec<Vec<bool>>) -> Self {
        // initializes binary CSP with given unary constraints and no binary constraints
        let num_variables = unary_constraints.len();
        BinaryCSP {
            unary_constraints: unary_constraints.into(),
            binary_constraints: empty_binary_constraints(num_variables),
        }
    }

    pub fn num_variables(&self) -> usize {
        self.unary_constraints.len()
    }

    pub fn var_range(&self) -> impl Iterator<Item = usize> {
        0..self.num_variables()
    }

    pub fn var_range_from(&self, var: usize) -> impl Iterator<Item = usize> {
        var + 1..self.num_variables()
    }

    pub fn domain_size(&self, var: usize) -> usize {
        self.unary_constraints.inner_len()
    }

    pub fn domain_range(&self, var: usize) -> impl Iterator<Item = usize> {
        0..self.domain_size(var)
    }

    fn sorted_vars(&self, var_x: usize, var_y: usize) -> (usize, usize) {
        match var_x <= var_y {
            true => (var_x, var_y),
            false => (var_y, var_x),
        }
    }

    fn binary_constraint_index(&self, var_x: usize, var_y: usize) -> (usize, usize) {
        let (var_x, var_y) = self.sorted_vars(var_x, var_y);
        assert_ne!(var_x, var_y);
        assert!(var_x < var_y);
        assert!(var_x < self.num_variables());
        assert!(var_y < self.num_variables());
        (var_x, var_y - var_x - 1)
    }

    pub fn add_binary_constraint(
        &mut self,
        var_x: usize,
        var_y: usize,
        binary_constraint: Vec<Vec<bool>>,
    ) -> &mut Self {
        let (var_x, var_y) = self.binary_constraint_index(var_x, var_y);
        // todo: assert that input (binary_constraint) has correct shape
        // todo: assert that no previous binary constraint exists
        self.binary_constraints[[var_x, var_y]] = Some(binary_constraint.into());
        self
    }

    pub fn is_unary_satisfied(&self, var: usize, label: usize) -> &bool {
        assert!(var < self.num_variables());
        assert!(label < self.domain_size(var));
        self.unary_constraints.get([var, label])
    }

    pub fn is_binary_satisfied(
        &self,
        var_x: usize,
        var_y: usize,
        label_x: usize,
        label_y: usize,
    ) -> bool {
        let (var_x, var_y) = self.binary_constraint_index(var_x, var_y);
        self.binary_constraints[[var_x, var_y]]
            .as_ref()
            .map_or_else(|| true, |table| *table.get([label_x, label_y]))
    }

    pub fn exists_binary_constraint(&self, var_x: usize, var_y: usize) -> bool {
        let (var_x, var_y) = self.binary_constraint_index(var_x, var_y);
        self.binary_constraints[[var_x, var_y]].is_some()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn new() {
    //     // todo: add tests
    // }
}
