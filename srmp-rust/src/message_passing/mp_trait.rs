#![allow(dead_code)]

use crate::{factor_types::factor_trait::Factor, FactorOrigin, GeneralCFN};

pub trait MessagePassing
where
    Self: Factor,
{
    type MessageData;

    fn new_message_data(
        cfn: &GeneralCFN,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self::MessageData;

    fn add_incoming_message(&mut self, message: &Self, message_data: &Self::MessageData);
    fn subtract_outgoing_message(&mut self, message: &Self, message_data: &Self::MessageData);
    fn add_outgoing_message(&mut self, message: &Self, message_data: &Self::MessageData);
    fn update_message_with_min(&self, message: &mut Self, message_data: &Self::MessageData) -> f64;
    fn renormalize_message(&mut self, delta: f64);
}
