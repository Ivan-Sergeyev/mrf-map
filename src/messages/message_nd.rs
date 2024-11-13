#![allow(dead_code)]

use std::{
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use crate::{
    cfn::solution::Solution, factors::factor_trait::Factor, CostFunctionNetwork, FactorOrigin,
};

use super::message_trait::Message;

// Stores the complete reindexing information for performing binary operations on messages of different dimensions
// See MessageND::add_assign_outgoing() and sub_assign_outgoing() on how the indices are used
// todo: better desc
pub struct AlignmentIndexing {
    index_first: Vec<usize>,
    index_second: Vec<usize>,
}

impl AlignmentIndexing {
    // Computes the offsets used for indexing in the message
    fn compute_strides(
        cfn: &CostFunctionNetwork,
        alpha_variables: &Vec<usize>,
        beta_variables: &Vec<usize>,
    ) -> Vec<usize> {
        // Compute strides[i] = product of domain sizes of variables in alpha
        // starting with the smallest variable in alpha that is greater than beta[i]
        // and ending with the last variable in alpha

        let beta_arity = beta_variables.len();
        let mut strides = vec![0; beta_arity + 1];
        strides[beta_arity] = 1; // barrier element
        let mut alpha_var_rev_iter = alpha_variables.iter().rev().peekable();

        for (beta_var_idx, beta_var) in beta_variables.iter().rev().enumerate() {
            let stride_index = beta_arity - 1 - beta_var_idx;
            strides[stride_index] = strides[stride_index + 1];
            while alpha_var_rev_iter
                .peek()
                .is_some_and(|alpha_var| *beta_var != **alpha_var)
            {
                strides[stride_index] *= cfn.domain_size(*alpha_var_rev_iter.next().unwrap());
            }
        }

        strides
    }

    // Computes the indexing table corresponding to the two given sets of variables
    fn compute_indexing(
        cfn: &CostFunctionNetwork,
        alpha_variables: &Vec<usize>,
        beta_variables: &Vec<usize>,
        beta_function_table_len: usize,
    ) -> Vec<usize> {
        // Assumption: alpha_variables contains beta_variables, beta_variables is not empty

        let strides = AlignmentIndexing::compute_strides(&cfn, &alpha_variables, &beta_variables);

        let mut beta_labeling = vec![0; beta_variables.len()];
        let mut indexing_table = vec![0; beta_function_table_len];
        indexing_table[0] = 0;
        let beta_var_idx_start = beta_variables.len() - 1;
        let mut beta_var_idx = beta_var_idx_start;
        let mut table_idx = 0;
        let mut k = 0;

        loop {
            if beta_labeling[beta_var_idx] < cfn.domain_size(beta_variables[beta_var_idx]) - 1 {
                // "Advance" to next label
                beta_labeling[beta_var_idx] += 1;
                k += strides[beta_var_idx];
                table_idx += 1;
                indexing_table[table_idx] = k;
                beta_var_idx = beta_var_idx_start;
            } else {
                // "Carry over" to initial label
                k -= beta_labeling[beta_var_idx] * strides[beta_var_idx];
                beta_labeling[beta_var_idx] = 0;
                if beta_var_idx == 0 {
                    break;
                }
                beta_var_idx -= 1;
            }
        }

        indexing_table
    }

    // Initializes the alignment structure for the given cost function network,
    // with `alpha` as the source factor and `beta` as the target factor
    pub fn new(cfn: &CostFunctionNetwork, alpha: &FactorOrigin, beta: &FactorOrigin) -> Self {
        // Assumption: alpha strictly contains all variables in beta

        let alpha_vars = cfn.factor_variables(alpha);
        let beta_vars = cfn.factor_variables(beta);
        let diff_vars = cfn.get_variables_difference(alpha, beta);
        let alpha_ft_len = cfn.function_table_len(alpha);
        let beta_ft_len = cfn.function_table_len(beta);
        let diff_ft_len = alpha_ft_len / beta_ft_len;

        AlignmentIndexing {
            index_first: Self::compute_indexing(cfn, &alpha_vars, &beta_vars, beta_ft_len),
            index_second: Self::compute_indexing(cfn, &alpha_vars, &diff_vars, diff_ft_len),
        }
    }
}

// Stores a message for a general factor, using complete reindexing information for handling messages of different dimensions
#[derive(Debug, PartialEq)]
pub struct MessageND {
    value: Vec<f64>,
}

impl Message for MessageND {
    type OutgoingAlignment = AlignmentIndexing;

    fn new_outgoing_alignment(
        cfn: &CostFunctionNetwork,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self::OutgoingAlignment {
        Self::OutgoingAlignment::new(cfn, alpha, beta)
    }

    fn iter(&self) -> Iter<f64> {
        self.value.iter()
    }

    fn iter_mut(&mut self) -> IterMut<f64> {
        self.value.iter_mut()
    }

    fn min(&self) -> &f64 {
        self.value.iter().min_by(|a, b| a.total_cmp(b)).unwrap()
    }

    fn index_min(&self) -> usize {
        self.value
            .iter()
            .enumerate()
            .fold((0, f64::INFINITY), |(idx_max, val_min), (idx, &val)| {
                if val < val_min {
                    (idx, val)
                } else {
                    (idx_max, val_min)
                }
            })
            .0
    }

    fn add_assign_incoming(&mut self, rhs: &Self) {
        for (val, rhs_val) in self.iter_mut().zip(rhs.iter()) {
            *val += rhs_val;
        }
    }

    fn sub_assign_incoming(&mut self, rhs: &Self) {
        for (val, rhs_val) in self.iter_mut().zip(rhs.iter()) {
            *val -= rhs_val;
        }
    }

    fn add_assign_outgoing(&mut self, rhs: &Self, outgoing_alignment: &Self::OutgoingAlignment) {
        for (first_index, first) in outgoing_alignment.index_first.iter().enumerate() {
            for second in outgoing_alignment.index_second.iter() {
                self.value[*first + *second] += rhs[first_index];
            }
        }
    }

    fn sub_assign_outgoing(&mut self, rhs: &Self, outgoing_alignment: &Self::OutgoingAlignment) {
        for (first_index, first) in outgoing_alignment.index_first.iter().enumerate() {
            for second in outgoing_alignment.index_second.iter() {
                self.value[*first + *second] -= rhs[first_index];
            }
        }
    }

    fn mul_assign_scalar(&mut self, rhs: f64) {
        for elem in self.value.iter_mut() {
            *elem *= rhs;
        }
    }

    fn add_assign_scalar(&mut self, rhs: f64) {
        for elem in self.value.iter_mut() {
            *elem += rhs;
        }
    }

    fn set_to_reparam_min(
        &mut self,
        rhs: &Self,
        outgoing_alignment: &Self::OutgoingAlignment,
    ) -> f64 {
        // todo: describe implementation details

        let mut rhs_min = f64::INFINITY;
        for (first_index, first) in outgoing_alignment.index_first.iter().enumerate() {
            let tmp_min = outgoing_alignment
                .index_second
                .iter()
                .map(|second| rhs.value[*first + *second])
                .min_by(|a, b| a.total_cmp(b))
                .unwrap();
            self.value[first_index] = tmp_min;
            rhs_min = rhs_min.min(tmp_min);
        }
        rhs_min
    }

    fn restricted_min(
        &self,
        cfn: &CostFunctionNetwork,
        solution: &Solution,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self {
        // todo: describe implementation details

        let alpha_vars = cfn.factor_variables(alpha);
        let beta_vars = cfn.factor_variables(beta);

        let alpha_arity = alpha_vars.len();
        let beta_arity = beta_vars.len();

        let mut self_strides = Vec::with_capacity(alpha_arity);
        let mut beta_strides = Vec::with_capacity(alpha_arity);
        let mut self_domain_sizes = Vec::with_capacity(alpha_arity);
        let mut labeling = Vec::with_capacity(alpha_arity);

        let mut self_stride = 1;
        let mut self_entry_index = 0;
        let mut beta_entry_index = 0;

        // todo: precompute strides, save similar to existing alaignment data, then select needed entries
        for alpha_var_index in (0..alpha_arity).rev() {
            let mut beta_stride = 1;
            let mut beta_var_index = beta_arity - 1;
            while alpha_vars[alpha_var_index] != beta_vars[beta_var_index] {
                beta_stride *= cfn.domain_size(beta_vars[beta_var_index]);
                if beta_var_index == 0 {
                    beta_stride = 0;
                    break;
                }
                beta_var_index -= 1;
            }

            if let Some(label) = solution[alpha_vars[alpha_var_index]] {
                self_entry_index += label * self_stride;
                beta_entry_index += label * beta_stride;
            } else {
                self_strides.push(self_stride);
                beta_strides.push(beta_stride);
                self_domain_sizes.push(cfn.domain_size(alpha_vars[alpha_var_index]));
                labeling.push(0);
            }

            self_stride *= cfn.domain_size(alpha_vars[alpha_var_index]);
        }

        let mut theta_beta = MessageND::inf(cfn, beta);
        theta_beta[beta_entry_index] = self.value[self_entry_index];

        let labeling_len = labeling.len();

        if labeling_len == 0 {
            return theta_beta;
        }

        let mut i = 0;
        loop {
            if labeling[i] < self_domain_sizes[i] - 1 {
                labeling[i] += 1;
                self_entry_index += self_strides[i];
                beta_entry_index += beta_strides[i];
                theta_beta[beta_entry_index] =
                    theta_beta[beta_entry_index].min(self.value[self_entry_index]);
                i = 0;
            } else {
                self_entry_index -= labeling[i] * self_strides[i];
                beta_entry_index -= labeling[i] * beta_strides[i];
                labeling[i] = 0;
                i += 1;
                if i == labeling_len {
                    break;
                }
            }
        }
        theta_beta
    }

    fn update_solution_restricted_min(
        &self,
        cfn: &CostFunctionNetwork,
        beta: &FactorOrigin,
        solution: &mut Solution,
    ) {
        if let FactorOrigin::Variable(variable_index) = beta {
            // Choose a label with the smallest cost
            solution[*variable_index] = Some(self.index_min());
            return;
        }

        // todo: describe implementation details
        let beta_variables = cfn.factor_variables(beta);
        let arity = beta_variables.len();

        let mut strides = Vec::with_capacity(arity);
        let mut unlabeled_domain_sizes = Vec::with_capacity(arity);
        let mut unlabeled_variables = Vec::with_capacity(arity);
        let mut labeling = Vec::with_capacity(arity);

        let mut entry_index = 0;
        let mut stride = 1;

        for variable in beta_variables.iter().rev() {
            if let Some(label) = solution[*variable] {
                entry_index += label * stride
            } else {
                solution[*variable] = Some(0);
                unlabeled_domain_sizes.push(cfn.domain_size(*variable));
                strides.push(stride);
                unlabeled_variables.push(*variable);
                labeling.push(0);
            }
            stride *= cfn.domain_size(*variable);
        }

        let num_unlabeled = labeling.len();

        if num_unlabeled == arity {
            // Everything is unlabeled
            let mut index_min = self.index_min();
            for variable in beta_variables.iter().rev() {
                solution[*variable] = Some(index_min % cfn.domain_size(*variable));
                index_min /= cfn.domain_size(*variable);
            }
            return;
        }

        let mut min_value = self.value[entry_index];
        let mut i = 0;
        loop {
            if labeling[i] < unlabeled_domain_sizes[i] - 1 {
                labeling[i] += 1;
                entry_index += strides[i];
                if self.value[entry_index] < min_value {
                    min_value = self.value[entry_index];
                    for j in 0..num_unlabeled {
                        solution[unlabeled_variables[j]] = Some(labeling[j]);
                    }
                }
                i = 0;
            } else {
                entry_index -= labeling[i] * strides[i];
                labeling[i] = 0;
                i += 1;
                if i == num_unlabeled {
                    break;
                }
            }
        }
    }
}

impl Index<usize> for MessageND {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.value[index]
    }
}

impl IndexMut<usize> for MessageND {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.value[index]
    }
}

// todo: match on factortype, return corresponding messagetype, individual implementations
impl MessageND {
    pub fn zero(cfn: &CostFunctionNetwork, factor_origin: &FactorOrigin) -> Self {
        MessageND {
            value: vec![0.; cfn.function_table_len(factor_origin)],
        }
    }

    pub fn inf(cfn: &CostFunctionNetwork, factor_origin: &FactorOrigin) -> Self {
        MessageND {
            value: vec![f64::INFINITY; cfn.function_table_len(factor_origin)],
        }
    }

    pub fn clone_factor(cfn: &CostFunctionNetwork, factor_origin: &FactorOrigin) -> Self {
        match cfn.get_factor(factor_origin) {
            Some(factor) => MessageND {
                value: factor.clone_function_table(),
            },
            None => MessageND::zero(cfn, factor_origin),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cfn::uai::UAI,
        factors::{factor_type::FactorType, function_table::FunctionTable},
    };

    use super::*;

    #[test]
    fn compute_index_adjustment() {
        let domain_sizes = vec![3, 4, 5];
        let alpha_variables = vec![0, 1, 2];
        let beta_variables = vec![1];

        let alpha_origin = FactorOrigin::NonUnaryFactor(0);
        let beta_origin = FactorOrigin::Variable(beta_variables[0]);

        let mut cfn = CostFunctionNetwork::from_domain_sizes(&domain_sizes, false, 0);
        cfn.add_factor(FactorType::FunctionTable(FunctionTable::new(
            &cfn,
            alpha_variables,
            vec![0.; 3 * 4 * 5],
        )));
        cfn.add_factor(FactorType::FunctionTable(FunctionTable::new(
            &cfn,
            beta_variables,
            vec![0.; 4],
        )));

        let alignment = AlignmentIndexing::new(&cfn, &alpha_origin, &beta_origin);
        let expected = AlignmentIndexing {
            index_first: vec![0, 5, 10, 15],
            index_second: vec![0, 1, 2, 3, 4, 20, 21, 22, 23, 24, 40, 41, 42, 43, 44],
        };

        assert_eq!(alignment.index_first, expected.index_first);
        assert_eq!(alignment.index_second, expected.index_second);
    }

    #[test]
    fn restricted_min() {
        // todo: create instance by hand for independence
        let cfn = CostFunctionNetwork::read_uai(
            "test_instances/frustrated_cycle_5_sym.uai".into(),
            false,
        );

        let alpha = FactorOrigin::NonUnaryFactor(1);
        let beta = FactorOrigin::Variable(2);
        let solution = vec![Some(0), Some(1), None, None, None].into();
        let message = MessageND {
            value: vec![3., 4., 0., 1.],
        };

        let restricted_min = message.restricted_min(&cfn, &solution, &alpha, &beta);
        let expected = MessageND {
            value: vec![0., 1.],
        };

        assert_eq!(restricted_min, expected);
    }

    // todo: add tests for remaining functions
}
