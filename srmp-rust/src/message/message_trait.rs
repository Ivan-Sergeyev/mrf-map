#![allow(dead_code)]

use std::{
    ops::{Index, IndexMut},
    slice::{Iter, IterMut},
};

use crate::{cfn::solution::Solution, FactorOrigin, GeneralCFN};

// note: reparametrizations can be treated as messages
// (can think of them as "initial" messages, or messages from factors to themselves)

// todo:

pub trait Message: Index<usize> + IndexMut<usize> {
    type OutgoingAlignment;

    fn new_outgoing_alignment(
        cfn: &GeneralCFN,
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
    // self = theta_beta, rhs = theta_alpha, update_with_minimization sets self = min rhs over xa ~ xb

    fn restricted_min(
        &self,
        cfn: &GeneralCFN,
        partial_labeling: &Solution,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Self;
}
