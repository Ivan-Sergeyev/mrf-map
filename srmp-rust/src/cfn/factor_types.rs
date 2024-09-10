#![allow(dead_code)]

use ndarray::{Array, Array1, ArrayD, Ix1};
use petgraph::{visit::EdgeRef, Direction::{Incoming, Outgoing}};
use std::fmt::Display;

use crate::{CostFunctionNetwork, GeneralCFN};

use super::relaxation::RelaxationGraph;

pub trait Factor {
    fn arity(&self) -> usize;
    fn function_table_len(&self) -> usize;

    fn map(&self, mapping: fn(f64) -> f64) -> Self;
    fn map_inplace(&mut self, mapping: fn(&mut f64));

    fn new_zero_message(&self) -> Self;
    fn clone_for_message_passing(&self) -> Self;
}

pub struct NullaryFactor {
    value: f64,
}

impl NullaryFactor {
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Factor for NullaryFactor {
    fn arity(&self) -> usize {
        0
    }

    fn function_table_len(&self) -> usize {
        1
    }

    fn map(&self, mapping: fn(f64) -> f64) -> NullaryFactor {
        NullaryFactor {
            value: mapping(self.value),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        mapping(&mut self.value);
    }

    fn new_zero_message(&self) -> Self {
        NullaryFactor { value: 0. }
    }

    fn clone_for_message_passing(&self) -> Self {
        NullaryFactor { value: self.value }
    }
}

impl Display for NullaryFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub struct UnaryFactor {
    pub function_table: Array1<f64>,
}

impl Factor for UnaryFactor {
    fn arity(&self) -> usize {
        1
    }

    fn function_table_len(&self) -> usize {
        self.function_table.len()
    }

    fn map(&self, mapping: fn(f64) -> f64) -> UnaryFactor {
        UnaryFactor {
            function_table: self.function_table.map(|&value| mapping(value)),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        self.function_table.map_inplace(mapping);
    }

    fn new_zero_message(&self) -> Self {
        UnaryFactor {
            function_table: Array::zeros(self.function_table.raw_dim()),
        }
    }

    fn clone_for_message_passing(&self) -> Self {
        UnaryFactor {
            function_table: self.function_table.clone(),
        }
    }
}

impl From<Vec<f64>> for UnaryFactor {
    fn from(value: Vec<f64>) -> Self {
        UnaryFactor {
            function_table: value.into(),
        }
    }
}

impl From<Array1<f64>> for UnaryFactor {
    fn from(value: Array1<f64>) -> Self {
        UnaryFactor {
            function_table: value,
        }
    }
}

impl From<ArrayD<f64>> for UnaryFactor {
    fn from(value: ArrayD<f64>) -> Self {
        UnaryFactor {
            function_table: value
                .into_dimensionality::<Ix1>()
                .expect("Function table should be 1-dimensional"),
        }
    }
}

impl Display for UnaryFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.function_table
                .iter()
                .map(|&value| value.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

pub struct GeneralFactor {
    pub function_table: ArrayD<f64>,
}

impl Factor for GeneralFactor {
    fn arity(&self) -> usize {
        self.function_table.ndim()
    }

    fn function_table_len(&self) -> usize {
        self.function_table.len()
    }

    fn map(&self, mapping: fn(f64) -> f64) -> GeneralFactor {
        GeneralFactor {
            function_table: Array::from_shape_vec(
                self.function_table.shape(),
                self.function_table
                    .iter()
                    .map(|&value| mapping(value))
                    .collect(),
            )
            .unwrap(),
        }
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        self.function_table.map_inplace(mapping);
    }

    fn new_zero_message(&self) -> Self {
        GeneralFactor {
            function_table: Array::zeros(self.function_table.raw_dim()),
        }
    }

    fn clone_for_message_passing(&self) -> Self {
        GeneralFactor {
            function_table: self.function_table.clone(),
        }
    }
}

// todo: implement for all factor types (raise unimplemented for Nullary (should never happen), simplify for Unary, use below for General, add serializer for generic type)
fn send_message(
    cfn: &GeneralCFN,
    relaxation_graph: &RelaxationGraph,
    messages: &mut Vec<FactorType>,
    alpha_beta: usize,
) -> f64 {
    // Assumptions:
    // - `relaxation_graph` is based on `cfn`
    // - `messages` are in the same order as edges in `relaxation_graph`
    // - `alpha_beta` is the index of an edge in `relaxation_graph`

    // Initialize current reparametrization
    let alpha = relaxation_graph.raw_edges()[alpha_beta].source();
    let alpha_origin = relaxation_graph.node_weight(alpha).unwrap();
    let mut theta_alpha = cfn.factor_clone_for_message_passing(alpha_origin);

    // Add incoming messages
    for gamma_alpha in relaxation_graph.edges_directed(alpha, Incoming) {
        theta_alpha += messages[gamma_alpha.id().index()];  // todo: implement += for messages/factors
    }

    // Subtract outgoing messages
    for gamma_alpha in relaxation_graph.edges_directed(alpha, Outgoing) {
        if gamma_alpha.id().index() == alpha_beta {
            continue;
        }
        // theta_alpha -= messages[gamma_alpha.id().index()];  // todo: use IndexAlignmentTable

        // ---------- convert from:
        // int KB = e2->B->K;
		// int KC = KA / KB;
		// int* TB = (int*) e2->send_message_data;
		// int* TC = TB + KB;
		// for (b=0; b<KB; b++)
		// for (c=0; c<KC; c++)
		// {
		// 	theta[TB[b] + TC[c]] -= e2->m[b];
		// }
    }

    // Renormalize
    let delta = 0.;
    // set message_alpha beta = for each b component, min over additional (a-b) components of theta, save smallest component as delta
    // subtract delta from all components of message_alpha beta

    // ---------- convert from:
    // int KB = e->B->K;
    // int KC = KA / KB;
    // int* TB = (int*) e->send_message_data;
    // int* TC = TB + KB;
    // for (b=0; b<KB; b++)
    // {
    //     double v_min = theta[TB[b]]; // TC[c] == 0
    //     for (c=1; c<KC; c++)
    //     {
    //         if (v_min > theta[TB[b] + TC[c]]) v_min = theta[TB[b] + TC[c]];
    //     }
    //     e->m[b] = v_min;
    //     if (b==0 || delta>v_min) delta = v_min;
    // }
    // for (b=0; b<KB; b++) e->m[b] -= delta;

    // Return renormalization delta
    delta
}

impl From<ArrayD<f64>> for GeneralFactor {
    fn from(value: ArrayD<f64>) -> Self {
        GeneralFactor {
            function_table: value,
        }
    }
}

impl Display for GeneralFactor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.function_table
                .iter()
                .map(|&value| value.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

pub enum FactorType {
    Nullary(NullaryFactor),
    Unary(UnaryFactor),
    General(GeneralFactor),
}

macro_rules! match_factor_action {
    ($factor_type:ident, $factor_match:ident, $action:expr) => {
        match $factor_type {
            FactorType::Nullary($factor_match) => $action,
            FactorType::Unary($factor_match) => $action,
            FactorType::General($factor_match) => $action,
        }
    };
}

macro_rules! match_factor_wrapped_action {
    ($factor_type:ident, $factor_match:ident, $action:expr) => {
        match $factor_type {
            FactorType::Nullary($factor_match) => FactorType::Nullary($action),
            FactorType::Unary($factor_match) => FactorType::Unary($action),
            FactorType::General($factor_match) => FactorType::General($action),
        }
    };
}

impl Factor for FactorType {
    fn arity(&self) -> usize {
        match_factor_action!(self, factor, factor.arity())
    }

    fn function_table_len(&self) -> usize {
        match_factor_action!(self, factor, factor.function_table_len())
    }

    fn map(&self, mapping: fn(f64) -> f64) -> FactorType {
        match_factor_wrapped_action!(self, factor, factor.map(mapping))
    }

    fn map_inplace(&mut self, mapping: fn(&mut f64)) {
        match_factor_action!(self, factor, factor.map_inplace(mapping))
    }

    fn new_zero_message(&self) -> Self {
        match_factor_wrapped_action!(self, factor, factor.new_zero_message())
    }

    fn clone_for_message_passing(&self) -> Self {
        match_factor_wrapped_action!(self, factor, factor.clone_for_message_passing())
    }
}
