#![allow(dead_code)]

use core::panic;

use crate::{
    factor_types::{
        factor_type::FactorType, general_factor::GeneralFactor, unary_factor::UnaryFactor,
    },
    CostFunctionNetwork, FactorOrigin, GeneralCFN,
};

use super::{
    mp_general_factor::GeneralMessageData, mp_trait::MessagePassing,
    mp_unary_factor::UnaryMessageData,
};

pub enum MessageData {
    GeneralFactor(GeneralMessageData),
    UnaryFactor(UnaryMessageData),
}

// todo: macro to implement "Into"
impl<'a> Into<&'a GeneralMessageData> for &'a MessageData {
    fn into(self) -> &'a GeneralMessageData {
        match self {
            MessageData::GeneralFactor(general_factor_message_data) => {
                general_factor_message_data
            }
            _ => panic!("Trying to convert MessageData to GeneralFactorMessageData, but it has a different type"),
        }
    }
}

impl<'a> Into<&'a UnaryMessageData> for &'a MessageData {
    fn into(self) -> &'a UnaryMessageData {
        match self {
            MessageData::UnaryFactor(unary_factor_message_data) => {
                unary_factor_message_data
            }
            _ => panic!("Trying to convert MessageData to UnaryFactorMessageData, but it has a different type"),
        }
    }
}

macro_rules! match_factor_action {
    ($factor_type:ident, $factor_match:ident, $action:expr, $nullary_action:expr) => {
        match $factor_type {
            FactorType::Nullary(_) => $nullary_action,
            FactorType::Unary($factor_match) => $action,
            FactorType::General($factor_match) => $action,
        }
    };
}

impl MessagePassing for FactorType {
    type MessageData = MessageData;

    fn new_message_data(
        cfn: &GeneralCFN,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self::MessageData {
        match cfn.get_factor(beta) {
            Some(beta_factor_type) => match beta_factor_type {
                FactorType::Nullary(_) => {
                    panic!("Trying to create new message targeting a nullary factor")
                }
                FactorType::Unary(_) => {
                    MessageData::UnaryFactor(UnaryFactor::new_message_data(cfn, alpha, beta))
                }
                FactorType::General(_) => {
                    MessageData::GeneralFactor(GeneralFactor::new_message_data(cfn, alpha, beta))
                }
            },
            None => MessageData::UnaryFactor(UnaryFactor::new_message_data(cfn, alpha, beta)),
        }
    }

    fn add_incoming_message(&mut self, message: &Self, message_data: &Self::MessageData) {
        match_factor_action!(
            self,
            factor,
            factor.add_incoming_message(message.into(), message_data.into()),
            panic!("Trying to add incoming message targeting a nullary factor")
        )
    }

    fn subtract_outgoing_message(&mut self, message: &Self, message_data: &Self::MessageData) {
        match_factor_action!(
            self,
            factor,
            factor.subtract_outgoing_message(message.into(), message_data.into()),
            panic!("Trying to subtract outgoing message targeting a nullary factor")
        )
    }

    fn add_outgoing_message(&mut self, message: &Self, message_data: &Self::MessageData) {
        match_factor_action!(
            self,
            factor,
            factor.add_outgoing_message(message.into(), message_data.into()),
            panic!("Trying to add outgoing message targeting a nullary factor")
        )
    }

    fn update_message_with_min(&self, message: &mut Self, message_data: &Self::MessageData) -> f64 {
        match_factor_action!(
            self,
            factor,
            factor.update_message_with_min(message.into(), message_data.into()),
            panic!("Trying to update message with min targeting a nullary factor")
        )
    }

    fn renormalize_message(&mut self, delta: f64) {
        match_factor_action!(
            self,
            factor,
            factor.renormalize_message(delta),
            panic!("Trying to renormalize message targeting a nullary factor")
        )
    }
}
