#![allow(dead_code)]

use std::slice::Iter;

use petgraph::{graph::NodeIndex, Direction::Incoming};

use super::relaxation::Relaxation;

// Stores the sequence of factors considered in the SRMP algorithm
pub struct FactorSequence {
    sequence: Vec<NodeIndex<usize>>, // contains node indices in the relaxation grpah
}

impl FactorSequence {
    // Creates a factor sequence for the given relaxation
    // (i.e., all unary factors and all factors with at least one incoming edge)
    pub fn new(relaxation: &Relaxation) -> Self {
        FactorSequence {
            sequence: relaxation
                .node_indices()
                .filter(|node_index| {
                    relaxation.is_unary_factor(*node_index)
                        || relaxation.has_edges(*node_index, Incoming)
                })
                .collect(),
        }
    }

    // Sorts the factor sequence
    pub fn sort(mut self) -> Self {
        // todo: add options for different sorting criteria
        self.sequence.sort_unstable();
        self
    }

    pub fn iter(&self) -> Iter<NodeIndex<usize>> {
        self.sequence.iter()
    }
}
