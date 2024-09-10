#![allow(dead_code)]

use crate::{
    factor_types::{factor_trait::Factor, general_factor::GeneralFactor},
    CostFunctionNetwork, FactorOrigin, GeneralCFN,
};

use super::mp_trait::MessagePassing;

pub struct GeneralMessageData {
    first_index_alignment: Vec<usize>,
    second_index_alignment: Vec<usize>,
}

impl GeneralMessageData {
    fn _compute_index_adjustment(
        cfn: &GeneralCFN,
        alpha_variables: &Vec<usize>,
        beta_variables: &Vec<usize>,
        beta_function_table_len: usize,
    ) -> Vec<usize> {
        let mut k_array = vec![0; beta_variables.len() + 1];
        k_array[beta_variables.len()] = 1; // barrier element
        let mut alpha_var_idx = alpha_variables.len() - 1;
        for (beta_var_idx, &beta_var) in beta_variables.iter().rev().enumerate() {
            k_array[beta_var_idx] = k_array[beta_var_idx + 1];
            while beta_var != alpha_variables[alpha_var_idx] {
                k_array[beta_var_idx] *= cfn.domain_size(alpha_variables[alpha_var_idx]);
                alpha_var_idx -= 1;
            }
        }

        let mut beta_labeling = vec![0; beta_variables.len()];
        let mut index_adjustment_table = vec![0; beta_function_table_len];
        index_adjustment_table[0] = 0;
        let mut beta_var_idx = beta_variables.len() - 1;
        let mut table_idx = 0;
        let mut k = 0;
        loop {
            if beta_labeling[beta_var_idx] < cfn.domain_size(beta_variables[beta_var_idx]) - 1 {
                // Move to next variable label
                beta_labeling[beta_var_idx] += 1;
                k += k_array[beta_var_idx];
                table_idx += 1;
                index_adjustment_table[table_idx] = k;
                beta_var_idx = beta_variables.len() - 1;
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

        index_adjustment_table
    }

    pub fn new(cfn: &GeneralCFN, alpha: &FactorOrigin, beta: &FactorOrigin) -> Self {
        let alpha_variables = cfn.get_factor_variables(alpha);
        let beta_variables = cfn.get_factor_variables(beta);

        let alpha_ft_len = cfn.get_function_table_len(alpha);
        let beta_ft_len = cfn.get_function_table_len(beta);
        let difference_ft_len = alpha_ft_len / beta_ft_len;

        let first_block_indices = GeneralMessageData::_compute_index_adjustment(
            cfn,
            alpha_variables,
            beta_variables,
            beta_ft_len,
        );

        let difference = cfn.get_variables_difference(alpha, beta);
        let second_block_indices = GeneralMessageData::_compute_index_adjustment(
            cfn,
            alpha_variables,
            &difference,
            difference_ft_len,
        );

        GeneralMessageData {
            first_index_alignment: first_block_indices,
            second_index_alignment: second_block_indices,
        }
    }
}

impl MessagePassing for GeneralFactor {
    type MessageData = GeneralMessageData;

    fn new_message_data(
        cfn: &GeneralCFN,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self::MessageData {
        GeneralMessageData::new(cfn, alpha, beta)
    }

    fn add_incoming_message(&mut self, message: &Self, _message_data: &Self::MessageData) {
        self.add_assign(message);
    }

    fn subtract_outgoing_message(&mut self, message: &Self, message_data: &Self::MessageData) {
        for (b, b_index) in message_data.first_index_alignment.iter().enumerate() {
            for c_index in message_data.second_index_alignment.iter() {
                self[*b_index + *c_index] -= message[b];
            }
        }
    }

    fn add_outgoing_message(&mut self, message: &Self, message_data: &Self::MessageData) {
        for (b, b_index) in message_data.first_index_alignment.iter().enumerate() {
            for c_index in message_data.second_index_alignment.iter() {
                self[*b_index + *c_index] += message[b];
            }
        }
    }

    fn update_message_with_min(
        &self,
        message: &mut GeneralFactor,
        message_data: &Self::MessageData,
    ) -> f64 {
        for (b, b_index) in message_data.first_index_alignment.iter().enumerate() {
            let mut v_min = self[*b_index];
            for c_index in message_data.second_index_alignment.iter() {
                v_min = v_min.min(self[*b_index + *c_index]);
            }
            message[b] = v_min;
        }
        self.min()
    }

    fn renormalize_message(&mut self, delta: f64) {
        self.add_assign_number(-delta);
    }
}
