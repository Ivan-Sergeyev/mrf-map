#![allow(dead_code)]

use std::{
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use log::{debug, warn};

use crate::{
    factor_types::{factor_trait::Factor, factor_type::FactorType},
    CostFunctionNetwork, FactorOrigin, Solution,
};

use super::message_trait::Message;

pub struct Alignment1D {
    index: usize,
}

pub struct Message1D {
    value: Vec<f64>,
}

impl Message for Message1D {
    type OutgoingAlignment = ();

    fn new_outgoing_alignment(
        _cfn: &CostFunctionNetwork,
        _alpha: &FactorOrigin,
        _beta: &FactorOrigin,
    ) -> Self::OutgoingAlignment {
        ()
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
            "In add_assign_outgoing() for self {:?} rhs {:?} align: {:?}",
            self.value, rhs.value, outgoing_alignment,
        );
        warn!("add_assign_outgoing() should never be called for Message1D, because unary factors should have no outgoing edges.");
    }

    fn sub_assign_outgoing(&mut self, rhs: &Self, outgoing_alignment: &Self::OutgoingAlignment) {
        debug!(
            "In add_assign_outgoing() for self {:?} rhs {:?} align: {:?}",
            self.value, rhs.value, outgoing_alignment,
        );
        warn!("add_assign_outgoing() should never be called for Message1D, because unary factors should have no outgoing edges.");
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
        todo!();

        let mut rhs_min = f64::INFINITY;
        for (b, b_index) in outgoing_alignment.first_index.iter().enumerate() {
            let tmp_min = outgoing_alignment
                .second_index
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
        todo!();

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

        let mut theta_beta: MessageND = cfn.new_zero_message(beta).into();
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
        todo!();

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

impl Index<usize> for Message1D {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.value[index]
    }
}

impl IndexMut<usize> for Message1D {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.value[index]
    }
}

impl Message1D {
    pub fn zero_from_len(_factor: Option<&FactorType>, len: usize) -> Self {
        // todo: match on factortype, return corresponding messagetype, individual implementations of zero_from_size
        Message1D {
            value: vec![0.; len],
        }
    }

    pub fn clone_factor(factor: Option<&FactorType>, len: usize) -> Self {
        // todo: match on factortype, return corresponding messagetype, individual implementations of clone_factor
        match factor {
            Some(factor) => Message1D {
                value: factor.clone_function_table(),
            },
            None => Message1D::zero_from_len(factor, len),
        }
    }
}
