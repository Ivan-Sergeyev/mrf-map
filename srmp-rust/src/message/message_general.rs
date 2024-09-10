#![allow(dead_code)]

use std::{
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use log::debug;

use crate::{
    cfn::solution::Solution,
    factor_types::{
        factor_type::FactorType, general_factor::GeneralFactor, unary_factor::UnaryFactor,
    },
    CostFunctionNetwork, FactorOrigin, GeneralCFN,
};

use super::message_trait::Message;

pub struct GeneralOutgoingAlignment {
    first_align: Vec<usize>,
    second_align: Vec<usize>,
}

impl GeneralOutgoingAlignment {
    fn compute_index_adjustment(
        cfn: &GeneralCFN,
        alpha_variables: &Vec<usize>,
        beta_variables: &Vec<usize>,
        beta_function_table_len: usize,
    ) -> Vec<usize> {
        debug!("Connecting factors: {:?} and {:?}", alpha_variables, beta_variables);

        let mut k_array = vec![0; beta_variables.len() + 1];
        k_array[beta_variables.len()] = 1; // barrier element
        let mut alpha_var_idx = alpha_variables.len() - 1;
        for (beta_var_idx, &beta_var) in beta_variables.iter().rev().enumerate() {
            debug!("beta_var_idx {}, beta_var {}", beta_var_idx, beta_var);
            k_array[beta_var_idx] = k_array[beta_var_idx + 1];
            while beta_var != alpha_variables[alpha_var_idx] {
                k_array[beta_var_idx] *= cfn.domain_size(alpha_variables[alpha_var_idx]);
                if alpha_var_idx == 0 {
                    break;
                }
                alpha_var_idx -= 1;
            }
        }

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
                k += k_array[beta_var_idx];
                table_idx += 1;
                index_adjustment_table[table_idx] = k;
                beta_var_idx = beta_var_idx_start;
            } else {
                // "Carry over" to initial label
                k -= beta_labeling[beta_var_idx] * k_array[beta_var_idx];
                beta_labeling[beta_var_idx] = 0;
                if beta_var_idx == 0 {
                    break;
                }
                beta_var_idx -= 1;
            }
        }

        debug!("adjustment table {:?}", index_adjustment_table);
        index_adjustment_table
    }

    pub fn new(cfn: &GeneralCFN, alpha: &FactorOrigin, beta: &FactorOrigin) -> Self {
        let alpha_variables = cfn.factor_variables(alpha);
        let beta_variables = cfn.factor_variables(beta);
        debug!("New alignment for variables {:?} and {:?}", alpha_variables, beta_variables);

        let alpha_ft_len = cfn.get_function_table_len(alpha);
        let beta_ft_len = cfn.get_function_table_len(beta);
        let difference_ft_len = alpha_ft_len / beta_ft_len;

        let first_block_indices = GeneralOutgoingAlignment::compute_index_adjustment(
            cfn,
            alpha_variables,
            beta_variables,
            beta_ft_len,
        );

        let difference = cfn.get_variables_difference(alpha, beta);
        let second_block_indices = GeneralOutgoingAlignment::compute_index_adjustment(
            cfn,
            alpha_variables,
            &difference,
            difference_ft_len,
        );

        GeneralOutgoingAlignment {
            first_align: first_block_indices,
            second_align: second_block_indices,
        }
    }
}

pub struct GeneralMessage {
    value: Vec<f64>,
}

impl Message for GeneralMessage {
    type OutgoingAlignment = GeneralOutgoingAlignment;

    fn new_outgoing_alignment(
        cfn: &GeneralCFN,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self::OutgoingAlignment {
        GeneralOutgoingAlignment::new(cfn, alpha, beta)
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
        for (b, b_index) in outgoing_alignment.first_align.iter().enumerate() {
            for c_index in outgoing_alignment.second_align.iter() {
                self.value[*b_index + *c_index] += rhs[b];
            }
        }
    }

    fn sub_assign_outgoing(&mut self, rhs: &Self, outgoing_alignment: &Self::OutgoingAlignment) {
        debug!("value: {:?} align1: {:?} align2: {:?} rhs {:?}", self.value, outgoing_alignment.first_align, outgoing_alignment.second_align, rhs.value);
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
        cfn: &GeneralCFN,
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

impl From<GeneralFactor> for GeneralMessage {
    fn from(value: GeneralFactor) -> Self {
        GeneralMessage {
            value: value.function_table.into_iter().collect(),
        }
    }
}

impl From<UnaryFactor> for GeneralMessage {
    fn from(value: UnaryFactor) -> Self {
        GeneralMessage {
            value: value.function_table.into_iter().collect(),
        }
    }
}

impl From<FactorType> for GeneralMessage {
    fn from(value: FactorType) -> Self {
        match value {
            FactorType::Unary(unary_factor) => unary_factor.into(),
            FactorType::General(general_factor) => general_factor.into(),
        }
    }
}
