#![allow(dead_code)]

use std::{
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use crate::{cfn::solution::Solution, CostFunctionNetwork, FactorOrigin};

// Interface for working with messages.
// Note: reparametrizations are stored as messages, since they can be viewed as "initial" messages,
// or as messages from factors to themselves.
pub trait Message: Index<usize> + IndexMut<usize> {
    // When computing a reparametrization following equation (4) in the SRMP paper or another similar one,
    // one may need to subtract outgoing messages, which have a different dimension from the reparametrization vector.
    // This is handled by what is essentially tensor multiplication: each entry of an outgoing message is subtracted
    // from all entries of the reparametrization with the same label restriction.
    // `OutgoingAlignment` is a data structure that facilitates such operations on messages of different dimensions.
    type OutgoingAlignment;

    fn new_outgoing_alignment(
        cfn: &CostFunctionNetwork,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self::OutgoingAlignment;

    fn iter(&self) -> Iter<f64>;
    fn iter_mut(&mut self) -> IterMut<f64>;

    fn max(&self) -> &f64;
    fn min(&self) -> &f64;
    fn index_min(&self) -> usize;

    fn add_assign_incoming(&mut self, rhs: &Self);
    fn sub_assign_incoming(&mut self, rhs: &Self);
    fn add_assign_outgoing(&mut self, rhs: &Self, outgoing_alignment: &Self::OutgoingAlignment);
    fn sub_assign_outgoing(&mut self, rhs: &Self, outgoing_alignment: &Self::OutgoingAlignment);

    fn mul_assign_scalar(&mut self, rhs: f64);
    fn add_assign_scalar(&mut self, rhs: f64);

    fn update_with_minimization(
        &mut self,
        rhs: &Self,
        outgoing_alignment: &Self::OutgoingAlignment,
    ) -> f64;

    fn restricted_min(
        &self,
        cfn: &CostFunctionNetwork,
        solution: &Solution,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self;

    fn update_solution_restricted_minimum(
        &self,
        cfn: &CostFunctionNetwork,
        beta: &FactorOrigin,
        solution: &mut Solution,
    );
}
