#![allow(dead_code)]

use std::{
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use log::debug;

use crate::{
    cfn::solution::Solution, factor_types::factor_type::FactorType, CostFunctionNetwork,
    FactorOrigin,
};

use super::message_trait::Message;

pub struct GeneralAlignment {
    first_align: Vec<usize>,
    second_align: Vec<usize>,
}

impl GeneralAlignment {
    fn compute_strides(
        cfn: &CostFunctionNetwork,
        alpha_variables: &Vec<usize>,
        beta_variables: &Vec<usize>,
    ) -> Vec<usize> {
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

    fn compute_index_adjustment(
        cfn: &CostFunctionNetwork,
        alpha_variables: &Vec<usize>,
        beta_variables: &Vec<usize>,
        beta_function_table_len: usize,
    ) -> Vec<usize> {
        debug!(
            "In compute_index_adjustment() for alpha_variables {:?} and beta_variables {:?}",
            alpha_variables, beta_variables
        );

        let strides = GeneralAlignment::compute_strides(&cfn, &alpha_variables, &beta_variables);

        let mut beta_labeling = vec![0; beta_variables.len()];
        let mut index_adjustment_table = vec![0; beta_function_table_len];
        index_adjustment_table[0] = 0;
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
                index_adjustment_table[table_idx] = k;
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

        index_adjustment_table
    }

    pub fn new(cfn: &CostFunctionNetwork, alpha: &FactorOrigin, beta: &FactorOrigin) -> Self {
        let alpha_variables = cfn.factor_variables(alpha);
        let beta_variables = cfn.factor_variables(beta);
        let diff_variables = cfn.get_variables_difference(alpha, beta);
        let alpha_ft_len = cfn.full_function_table_size(alpha);
        let beta_ft_len = cfn.full_function_table_size(beta);
        let diff_ft_len = alpha_ft_len / beta_ft_len;

        let first_align =
            Self::compute_index_adjustment(cfn, alpha_variables, beta_variables, beta_ft_len);

        let second_align =
            Self::compute_index_adjustment(cfn, alpha_variables, &diff_variables, diff_ft_len);

        GeneralAlignment {
            first_align,
            second_align,
        }
    }
}

pub struct GeneralMessage {
    value: Vec<f64>,
}

impl Message for GeneralMessage {
    type OutgoingAlignment = GeneralAlignment;

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

    fn max(&self) -> &f64 {
        self.value.iter().max_by(|a, b| a.total_cmp(b)).unwrap()
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
        debug!(
            "In add_assign_incoming() for self {:?} rhs {:?}",
            self.value, rhs.value
        );
        for (val, rhs_val) in self.iter_mut().zip(rhs.iter()) {
            *val += rhs_val;
        }
    }

    fn sub_assign_incoming(&mut self, rhs: &Self) {
        debug!(
            "In sub_assign_incoming() for self {:?} rhs {:?}",
            self.value, rhs.value
        );
        for (val, rhs_val) in self.iter_mut().zip(rhs.iter()) {
            *val -= rhs_val;
        }
    }

    fn add_assign_outgoing(&mut self, rhs: &Self, outgoing_alignment: &Self::OutgoingAlignment) {
        debug!(
            "In add_assign_outgoing() for self {:?} rhs {:?} align1: {:?} align2: {:?}",
            self.value, rhs.value, outgoing_alignment.first_align, outgoing_alignment.second_align
        );
        for (b, b_index) in outgoing_alignment.first_align.iter().enumerate() {
            for c_index in outgoing_alignment.second_align.iter() {
                self.value[*b_index + *c_index] += rhs[b];
            }
        }
    }

    fn sub_assign_outgoing(&mut self, rhs: &Self, outgoing_alignment: &Self::OutgoingAlignment) {
        debug!(
            "In sub_assign_outgoing() for self {:?} rhs {:?} align1: {:?} align2: {:?}",
            self.value, rhs.value, outgoing_alignment.first_align, outgoing_alignment.second_align
        );
        for (b, b_index) in outgoing_alignment.first_align.iter().enumerate() {
            for c_index in outgoing_alignment.second_align.iter() {
                self.value[*b_index + *c_index] -= rhs[b];
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

    fn update_with_minimization(
        &mut self,
        rhs: &Self,
        outgoing_alignment: &Self::OutgoingAlignment,
    ) -> f64 {
        let mut rhs_min = f64::INFINITY;
        for (b, b_index) in outgoing_alignment.first_align.iter().enumerate() {
            let tmp_min = outgoing_alignment
                .second_align
                .iter()
                .map(|c_index| rhs.value[*b_index + *c_index])
                .min_by(|a, b| a.total_cmp(b))
                .unwrap();
            self.value[b] = tmp_min;
            rhs_min = rhs_min.min(tmp_min);
        }
        rhs_min
    }

    fn restricted_min(
        &self,
        cfn: &CostFunctionNetwork,
        partial_labeling: &Solution,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self {
        let alpha_arity = cfn.arity(alpha);
        let alpha_vars = cfn.factor_variables(alpha);
        let beta_vars = cfn.factor_variables(beta);

        let mut kb_factor_array = Vec::with_capacity(alpha_arity);
        let mut k_factor_array = Vec::with_capacity(alpha_arity);
        let mut k_array = Vec::with_capacity(alpha_arity);
        let mut labeling = Vec::with_capacity(alpha_arity);

        let mut k_factor = 1;
        let mut k = 0;
        let mut kb = 0;
        let mut beta_var_idx = beta_vars.len() - 1;

        // todo: precompute full kb_factor_array (doesn't depend on partial labeling) and save it similar to existing MessageData
        for alpha_var_idx in (0..alpha_arity).rev() {
            let mut kb_factor = 1;
            while alpha_vars[alpha_var_idx] != beta_vars[beta_var_idx] {
                kb_factor *= cfn.domain_size(beta_vars[beta_var_idx]);
                if beta_var_idx == 0 {
                    kb_factor *= 0;
                    break;
                }
                beta_var_idx -= 1;
            }

            if let Some(label) = partial_labeling[alpha_var_idx] {
                kb += label * kb_factor;
                k += label * k_factor;
            } else {
                kb_factor_array.push(kb_factor);
                k_factor_array.push(k_factor);
                k_array.push(cfn.domain_size(alpha_vars[alpha_var_idx]));
                labeling.push(0);
            }
            k_factor *= cfn.domain_size(alpha_vars[alpha_var_idx]);
        }

        let mut theta_beta: GeneralMessage = cfn.new_zero_message(beta).into();
        theta_beta[kb] = self.value[k];
        let mut i = 0;
        while i < labeling.len() {
            if labeling[i] < k_array[i] - 1 {
                // "Advance" to next label
                labeling[i] += 1;
                k += k_factor_array[i];
                kb += kb_factor_array[i];
                theta_beta[kb] = theta_beta[kb].min(self.value[k]);
                i = 0;
            } else {
                // "Carry over" to initial label
                k -= labeling[i] * k_factor_array[i];
                kb -= labeling[i] * kb_factor_array[i];
                labeling[i] = 0;
                i += 1;
            }
        }
        theta_beta
    }

    fn update_solution_restricted_minimum(
        &self,
        cfn: &CostFunctionNetwork,
        beta: &FactorOrigin,
        solution: &mut Solution,
    ) {
        let arity = cfn.arity(beta);

        let mut k = 0;
        let mut k_factor_array = Vec::with_capacity(arity);
        let mut k_array = Vec::with_capacity(arity);
        let mut index_array = Vec::with_capacity(arity);
        let mut labeling = Vec::with_capacity(arity);

        let mut k_factor = 1;

        for i in (0..arity).rev() {
            if let Some(label) = solution[i] {
                k += label * k_factor
            } else {
                solution[i] = Some(0);
                k_array.push(cfn.domain_size(i));
                k_factor_array.push(k_factor);
                index_array.push(i);
                labeling.push(0);
            }
            k_factor *= cfn.domain_size(i);
        }

        let n = labeling.len();

        if n == arity {
            // Everything is unlabeled
            let mut k_best = self.index_min();
            for i in (0..arity).rev() {
                solution[i] = Some(k_best % cfn.domain_size(i));
                if i == 0 {
                    return;
                }
                k_best /= cfn.domain_size(i);
            }
        }

        let mut v_best = self.value[k];
        let mut i = 0;
        loop {
            if labeling[i] < k_array[i] - 1 {
                labeling[i] += 1;
                k += k_factor_array[i];
                if v_best > self.value[k] {
                    v_best = self.value[k];
                    for j in 0..n {
                        solution[index_array[j]] = Some(labeling[j]);
                    }
                }
                i = 0;
            } else {
                k -= labeling[i] * k_factor_array[i];
                labeling[i] = 0;
                i += 1;
                if i == n {
                    break;
                }
            }
        }
    }
}

impl Index<usize> for GeneralMessage {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.value[index]
    }
}

impl IndexMut<usize> for GeneralMessage {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.value[index]
    }
}

impl GeneralMessage {
    pub fn zero_from_size(_factor: Option<&FactorType>, size: usize) -> Self {
        // todo: match on factortype, return corresponding messagetype, individual implementations of zero_from_size
        GeneralMessage {
            value: vec![0.; size],
        }
    }

    pub fn clone_factor(factor: Option<&FactorType>, size: usize) -> Self {
        // todo: match on factortype, return corresponding messagetype, individual implementations of clone_factor
        match factor {
            Some(factor) => match factor {
                FactorType::Unary(factor) => GeneralMessage {
                    value: factor.function_table.clone(),
                },
                FactorType::General(factor) => GeneralMessage {
                    value: factor.function_table.clone(),
                },
            },
            None => GeneralMessage::zero_from_size(factor, size),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_index_adjustment() {
        let domain_sizes = vec![3, 4, 5];
        let alpha_variables = vec![0, 1, 2];
        let beta_variables = vec![1];

        let mut cfn = CostFunctionNetwork::from_domain_sizes(&domain_sizes, false, 0);
        cfn.add_non_unary_factor(alpha_variables, FactorType::General(vec![0.; 1].into()));
        cfn.add_unary_factor(beta_variables[0], FactorType::Unary(vec![0.; 1].into()));

        let alpha_origin = FactorOrigin::NonUnaryFactor(0);
        let beta_origin = FactorOrigin::Variable(beta_variables[0]);

        let alignment = GeneralAlignment::new(&cfn, &alpha_origin, &beta_origin);
        let expected = GeneralAlignment {
            first_align: vec![0, 5, 10, 15],
            second_align: vec![0, 1, 2, 3, 4, 20, 21, 22, 23, 24, 40, 41, 42, 43, 44],
        };

        assert_eq!(alignment.first_align, expected.first_align);
        assert_eq!(alignment.second_align, expected.second_align);
    }
}
