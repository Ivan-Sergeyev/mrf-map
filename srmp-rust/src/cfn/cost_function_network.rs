#![allow(dead_code)]

use ndarray::*;  // todo: replace * with what is actually used
use petgraph::graph::*;  // todo: replace * with what is actually used
// use hypergraph::Hypergraph;

/// todo: docs
/// todo: upgrade to higher-order factors
pub trait CostFunctionNetwork {
    fn new(domain_sizes: Vec<usize>) -> Self;
    fn from_unary_costs(unary_costs: Vec<Vec<f64>>) -> Self;

    fn set_nullary_cost(self, nullary_cost: f64) -> Self;
    fn set_unary_cost(self, var: usize, costs: Array1<f64>) -> Self;
    fn set_pairwise_cost(self, var1: usize, var2: usize, costs: Array2<f64>) -> Self;
    fn set_cost(self, vars: Vec<usize>, costs: ArrayD<f64>) -> Self;

    fn num_variables(&self) -> usize;
    fn domain_size(&self, var: usize) -> usize;
    fn num_binary_factors(&self) -> usize;
}

/// todo: docs

struct UnaryFactor {
    costs: Array1<f64>,
    cfn_graph_node_index: NodeIndex<usize>,
    factor_graph_node_index: Option<NodeIndex<usize>>,
}

struct BinaryFactor {
    costs: Array2<f64>,
    cfn_graph_edge_index: EdgeIndex<usize>,
    factor_graph_node_index: Option<NodeIndex<usize>>,
}

struct CFNPetGraph {
    nullary_cost: f64,
    unary_factors: Vec<UnaryFactor>,
    binary_factors: Vec<BinaryFactor>,
    cfn_graph: UnGraph<usize, usize, usize>,  // node data = index in unary_factors, edge data = index in binary_factors
}

impl CostFunctionNetwork for CFNPetGraph {
    fn new(domain_sizes: Vec<usize>) -> Self {
        let num_variables = domain_sizes.len();

        let mut graph = UnGraph::with_capacity(num_variables, 0);
        let node_indices: Vec<NodeIndex<usize>> = (0..num_variables).map(|var| graph.add_node(var)).collect();

        let unary_factors = domain_sizes.into_iter().enumerate().map(|(var, domain_size)| UnaryFactor {
            costs: vec![0.; domain_size].into(),
            cfn_graph_node_index: node_indices[var],
            factor_graph_node_index: None,
        }).collect();

        CFNPetGraph {
            nullary_cost: 0.,
            unary_factors: unary_factors,
            binary_factors: Vec::new(),
            cfn_graph: graph,
        }
    }

    fn from_unary_costs(unary_costs: Vec<Vec<f64>>) -> Self {
        let num_variables = unary_costs.len();

        let mut graph = UnGraph::with_capacity(num_variables, 0);
        let node_indices: Vec<NodeIndex<usize>> = (0..num_variables).map(|var| graph.add_node(var)).collect();

        let unary_factors = unary_costs.into_iter().enumerate().map(|(var, unary_cost)| UnaryFactor {
            costs: unary_cost.into(),
            cfn_graph_node_index: node_indices[var],
            factor_graph_node_index: None,
        }).collect();

        CFNPetGraph {
            nullary_cost: 0.,
            unary_factors: unary_factors,
            binary_factors: Vec::new(),
            cfn_graph: graph,
        }
    }

    fn set_nullary_cost(mut self, nullary_cost: f64) -> Self {
        self.nullary_cost = nullary_cost;
        self
    }

    fn set_unary_cost(mut self, var: usize, costs: Array1<f64>) -> Self {
        let &factor_index = self.cfn_graph.node_weight(var.into()).unwrap();
        self.unary_factors[factor_index].costs = costs;
        self
    }

    fn set_pairwise_cost(mut self, var1: usize, var2: usize, costs: Array2<f64>) -> Self {
        assert!(var1 < var2);  // todo: rotate costs table if var1 > var2
        if let Some(&factor_index) = self.cfn_graph.find_edge(var1.into(), var2.into()).and_then(|edge_index| self.cfn_graph.edge_weight(edge_index)) {
            self.binary_factors[factor_index].costs = costs;
        } else {
            let edge_index = self.cfn_graph.add_edge(var1.into(), var2.into(), self.binary_factors.len());
            self.binary_factors.push(BinaryFactor{
                costs: costs,
                cfn_graph_edge_index: edge_index,
                factor_graph_node_index: None,
            });
        }
        self
    }

    fn set_cost(self, vars: Vec<usize>, costs: ArrayD<f64>) -> Self {
        match vars.len() {
            0 => self.set_nullary_cost(costs[[0]]),
            1 => self.set_unary_cost(vars[0], costs.into_dimensionality::<Ix1>().expect("costs array should be 1-dimensional")),
            2 => self.set_pairwise_cost(vars[0], vars[1], costs.into_dimensionality::<Ix2>().expect("costs array should be 2-dimensional")),
            _ => unimplemented!("CFNPetGraph does not support factors of higher arity than 2."),
        }
    }

    fn num_variables(&self) -> usize {
        self.unary_factors.len()
    }

    fn domain_size(&self, var: usize) -> usize {
        self.unary_factors[var].costs.len()
    }

    fn num_binary_factors(&self) -> usize {
        self.binary_factors.len()
    }
}

// struct CFN1 {} // CFN2...
// impl CostFunctionNetwork for CFN1 {} // for CFN2...
// pub use cfn::CFN1 as MyCFN;

// with pub trait CostFunctionNetwork:
// Box<dyn CostFunctionNetwork> cfn;


/// todo: docs
/// todo: multiple variants and methods
struct FactorGraph {
    graph: DiGraph<usize, (), usize>,  // node data = index in unary_factors or binary_factors (latter: shifted by cfn.num_variables())
}

trait ConstructRelaxation<RelaxationType> {
    fn construct_relaxation(&mut self) -> FactorGraph;
}

struct MinimalEdges;

impl ConstructRelaxation<MinimalEdges> for CFNPetGraph {
    fn construct_relaxation(&mut self) -> FactorGraph {
        let mut factor_graph = DiGraph::with_capacity(self.num_variables() + self.num_binary_factors(), 2 * self.num_binary_factors());

        for var_node_index in self.cfn_graph.node_indices() {
            let &var_index = self.cfn_graph.node_weight(var_node_index).unwrap();
            self.unary_factors[var_index].factor_graph_node_index = Some(factor_graph.add_node(var_index));
        }

        for binary_factor_edge_index in self.cfn_graph.edge_indices() {
            let &binary_factor_index = self.cfn_graph.edge_weight(binary_factor_edge_index).unwrap();
            let fg_node_index = factor_graph.add_node(self.num_variables() + binary_factor_index);
            self.binary_factors[binary_factor_index].factor_graph_node_index = Some(fg_node_index);

            let (var1, var2) = self.cfn_graph.edge_endpoints(binary_factor_edge_index).unwrap();
            factor_graph.add_edge(fg_node_index, var1, ());
            factor_graph.add_edge(fg_node_index, var2, ());
        }

        FactorGraph {
            graph: factor_graph,
        }
    }
}
