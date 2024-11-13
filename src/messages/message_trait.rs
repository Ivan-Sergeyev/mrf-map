#![allow(dead_code)]

use std::{
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use crate::{cfn::solution::Solution, CostFunctionNetwork, FactorOrigin};

// Interface for messages
// Note: reparametrizations are stored as messages, as every reparametrization can be treated as an initial message,
// or as a message from a factor it itself.
pub trait Message: Index<usize> + IndexMut<usize> {
    // When computing a reparametrization following equation (4) in the SRMP paper or another similar one,
    // one may need to subtract outgoing messages, which have a different dimension from the reparametrization vector.
    // This is handled by what is essentially tensor multiplication: each entry of an outgoing message is subtracted
    // from all entries of the reparametrization with the same label restriction.
    // `OutgoingAlignment` is a data structure that facilitates such operations on messages of different dimensions.
    type OutgoingAlignment;

    // Creates a new alignment structure for the given cost function network,
    // with `alpha` as the source factor and `beta` as the target factor
    // Assumption: alpha contains all variables in beta
    fn new_outgoing_alignment(
        cfn: &CostFunctionNetwork,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self::OutgoingAlignment;

    // Returns an iterator over the entries of this message
    fn iter(&self) -> Iter<f64>;

    // Returns a mutable iterator over the entries of this message
    fn iter_mut(&mut self) -> IterMut<f64>;

    // Returns the smallest entry in the message
    fn min(&self) -> &f64;

    // Returns the index of the smallest entry in the message
    fn index_min(&self) -> usize;

    // Adds an incoming message to this message
    fn add_assign_incoming(&mut self, rhs: &Self);

    // Subtracts an incoming message from this message
    fn sub_assign_incoming(&mut self, rhs: &Self);

    // Adds an outgoing message to this message (with the help of the given alignment struct)
    // Assumption: `self` and `rhs` are aligned using `outgoing_alignment`
    fn add_assign_outgoing(&mut self, rhs: &Self, outgoing_alignment: &Self::OutgoingAlignment);

    // Subtracts an outgoing message from this message (with the help of the given alignment struct)
    // Assumption: `self` and `rhs` are aligned using `outgoing_alignment`
    fn sub_assign_outgoing(&mut self, rhs: &Self, outgoing_alignment: &Self::OutgoingAlignment);

    // Multiplies all entries of this message by the given value
    fn mul_assign_scalar(&mut self, rhs: f64);

    // Adds the given value to all entries of this message
    fn add_assign_scalar(&mut self, rhs: f64);

    // Computes the minimum from equation (17) in the SRMP paper over a given reparametrization,
    // assigns the result to this message, and returns the smallest value (for normalization purposes)
    // Assumption: `self` and `rhs` are aligned using `outgoing_alignment`
    fn set_to_reparam_min(
        &mut self,
        rhs: &Self,
        outgoing_alignment: &Self::OutgoingAlignment,
    ) -> f64;

    // Computes the restricted minimum for sending restricted messages // todo: more detailed desc
    // Assumption: `self` is a message from `alpha` to `beta`
    fn restricted_min(
        &self,
        cfn: &CostFunctionNetwork,
        solution: &Solution,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self;

    // Updates the given solution by computing the restricted minimum // todo: more detailed desc
    // Assumption: `self` is a reparametrization that is being restricted to `beta` using `solution`
    fn update_solution_restricted_min(
        &self,
        cfn: &CostFunctionNetwork,
        beta: &FactorOrigin,
        solution: &mut Solution,
    );
}
