// old implementation of cost function networks

// use core::panic;
// use std::{
//     fs::File, io::{self, BufRead, BufReader, Write}, str::FromStr
// };

// use ndarray::{Array, Array1, Array2, ArrayD, Ix1, Ix2};
// // use ndarray; // todo: replace * with what is actually used
// use petgraph::graph::*; // todo: replace * with what is actually used

// use super::cost_function_network::*;

// struct UnaryTerm {
//     costs: Array1<f64>,
//     cfn_graph_index: NodeIndex<usize>,
//     factor_graph_node_index: Option<NodeIndex<usize>>,
// }

// struct PairwiseTerm {
//     costs: Array2<f64>,
//     cfn_graph_index: EdgeIndex<usize>,
//     factor_graph_node_index: Option<NodeIndex<usize>>,
// }

// pub struct PairwiseCFN {
//     nullary_cost: f64,
//     unary_terms: Vec<UnaryTerm>,
//     pairwise_terms: Vec<PairwiseTerm>,
//     cfn_graph: UnGraph<usize, usize, usize>, // node data = index in unary_terms, edge data = index in pairwise_terms
// }

// impl CostFunctionNetwork for PairwiseCFN {
//     fn new() -> Self {
//         PairwiseCFN {
//             nullary_cost: 0.,
//             unary_terms: Vec::new(),
//             pairwise_terms: Vec::new(),
//             cfn_graph: UnGraph::with_capacity(0, 0),
//         }
//     }

//     fn from_domain_sizes(domain_sizes: Vec<usize>) -> Self {
//         let num_variables = domain_sizes.len();

//         let mut graph = UnGraph::with_capacity(num_variables, 0);
//         let node_indices: Vec<NodeIndex<usize>> =
//             (0..num_variables).map(|var| graph.add_node(var)).collect();

//         let unary_terms = domain_sizes
//             .into_iter()
//             .enumerate()
//             .map(|(var, domain_size)| UnaryTerm {
//                 costs: vec![0.; domain_size].into(),
//                 cfn_graph_index: node_indices[var],
//                 factor_graph_node_index: None,
//             })
//             .collect();

//         PairwiseCFN {
//             nullary_cost: 0.,
//             unary_terms: unary_terms,
//             pairwise_terms: Vec::new(),
//             cfn_graph: graph,
//         }
//     }

//     fn from_unary_costs(unary_costs: Vec<Vec<f64>>) -> Self {
//         let num_variables = unary_costs.len();

//         let mut graph = UnGraph::with_capacity(num_variables, 0);
//         let node_indices: Vec<NodeIndex<usize>> =
//             (0..num_variables).map(|var| graph.add_node(var)).collect();

//         let unary_terms = unary_costs
//             .into_iter()
//             .enumerate()
//             .map(|(var, unary_cost)| UnaryTerm {
//                 costs: unary_cost.into(),
//                 cfn_graph_index: node_indices[var],
//                 factor_graph_node_index: None,
//             })
//             .collect();

//         PairwiseCFN {
//             nullary_cost: 0.,
//             unary_terms: unary_terms,
//             pairwise_terms: Vec::new(),
//             cfn_graph: graph,
//         }
//     }

//     fn set_nullary_cost(mut self, nullary_cost: f64) -> Self {
//         self.nullary_cost = nullary_cost;
//         self
//     }

//     fn set_unary_cost(mut self, var: usize, costs: Array1<f64>) -> Self {
//         let &factor_index = self.cfn_graph.node_weight(var.into()).unwrap();
//         self.unary_terms[factor_index].costs = costs;
//         self
//     }

//     fn set_pairwise_cost(mut self, var1: usize, var2: usize, costs: Array2<f64>) -> Self {
//         assert!(var1 < var2); // todo: rotate costs table if var1 > var2
//         if let Some(&factor_index) = self
//             .cfn_graph
//             .find_edge(var1.into(), var2.into())
//             .and_then(|edge_index| self.cfn_graph.edge_weight(edge_index))
//         {
//             self.pairwise_terms[factor_index].costs = costs;
//         } else {
//             let edge_index =
//                 self.cfn_graph
//                     .add_edge(var1.into(), var2.into(), self.pairwise_terms.len());
//             self.pairwise_terms.push(PairwiseTerm {
//                 costs: costs,
//                 cfn_graph_index: edge_index,
//                 factor_graph_node_index: None,
//             });
//         }
//         self
//     }

//     fn set_cost(self, vars: &Vec<usize>, costs: ArrayD<f64>) -> Self {
//         match vars.len() {
//             0 => self.set_nullary_cost(costs[[0]]),
//             1 => self.set_unary_cost(
//                 vars[0],
//                 costs
//                     .into_dimensionality::<Ix1>()
//                     .expect("Costs array should be 1-dimensional"),
//             ),
//             2 => self.set_pairwise_cost(
//                 vars[0],
//                 vars[1],
//                 costs
//                     .into_dimensionality::<Ix2>()
//                     .expect("Costs array should be 2-dimensional"),
//             ),
//             _ => unimplemented!("PairwiseCFN does not support terms of higher arity than 2."),
//         }
//     }

//     fn num_variables(&self) -> usize {
//         self.unary_terms.len()
//     }

//     fn domain_size(&self, var: usize) -> usize {
//         self.unary_terms[var].costs.len()
//     }

//     fn num_terms(&self) -> usize {
//         self.unary_terms.len() + self.pairwise_terms.len()
//     }

//     fn num_non_unary_terms(&self) -> usize {
//         self.pairwise_terms.len()
//     }
// }

// impl UAI for PairwiseCFN {
//     fn read_from_uai(file: File) -> Self {
//         let lines = BufReader::new(file).lines();

//         let mut state = UAIState::GraphType;
//         let mut trimmed_line;

//         let mut num_variables = 0;
//         let mut num_functions = 0;
//         let mut cfn = PairwiseCFN::new();
//         let mut function_scopes = Vec::new();
//         let mut function_entries = Vec::new();

//         for line in lines {
//             let line = line.unwrap();
//             trimmed_line = line.trim();

//             // skip empty lines
//             if trimmed_line.is_empty() {
//                 continue;
//             }

//             match state {
//                 UAIState::GraphType => {
//                     if trimmed_line != "MARKOV" {
//                         unimplemented!("Only MARKOV graph type is supported.");
//                     }
//                     state = UAIState::NumberOfVariables;
//                 }
//                 UAIState::NumberOfVariables => {
//                     num_variables = trimmed_line.parse::<usize>().unwrap();
//                     state = UAIState::DomainSizes;
//                 }
//                 UAIState::DomainSizes => {
//                     let domain_sizes = string_to_vec(trimmed_line);
//                     assert_eq!(num_variables, domain_sizes.len());
//                     cfn = PairwiseCFN::from_domain_sizes(domain_sizes);
//                     state = UAIState::NumberOfFunctions;
//                 }
//                 UAIState::NumberOfFunctions => {
//                     num_functions = trimmed_line.parse::<usize>().unwrap();
//                     function_scopes = Vec::with_capacity(num_functions);
//                     state = UAIState::FunctionScopes(0);
//                 }
//                 UAIState::FunctionScopes(function_idx) => {
//                     let mut function_scope = string_to_vec(trimmed_line);
//                     let scope_len = function_scope.remove(0);
//                     assert_eq!(scope_len, function_scope.len());
//                     function_scopes.push(function_scope);

//                     if function_idx < num_functions - 1 {
//                         state = UAIState::FunctionScopes(function_idx + 1);
//                     } else {
//                         state = UAIState::NumberOfTableValues(0);
//                     }
//                 }
//                 UAIState::NumberOfTableValues(function_idx) => {
//                     assert!(function_idx < function_scopes.len());
//                     let num_entries = trimmed_line.parse::<usize>().unwrap();
//                     function_entries = Vec::with_capacity(num_entries);
//                     state = UAIState::TableValues(function_idx, num_entries);
//                 }
//                 UAIState::TableValues(function_idx, num_entries) => {
//                     assert!(function_idx < function_scopes.len());
//                     let mut new_entries = string_to_vec(trimmed_line);
//                     function_entries.append(&mut new_entries);

//                     let cur_num_entries = function_entries.len();
//                     assert!(cur_num_entries <= num_entries);
//                     if cur_num_entries < num_entries {
//                         // need to collect more table entries
//                         state = UAIState::TableValues(function_idx, num_entries);
//                         continue;
//                     }

//                     // collected all table entries, ready to add cost function to cfn
//                     let costs = Array::from_shape_vec(
//                         function_scopes[function_idx]
//                             .iter()
//                             .map(|&var| cfn.domain_size(var))
//                             .collect::<Vec<usize>>(),
//                         function_entries.drain(..).collect(),
//                     )
//                     .unwrap();
//                     cfn = cfn.set_cost(&function_scopes[function_idx], costs);

//                     if function_idx < function_scopes.len() - 1 {
//                         // need to read more functions
//                         state = UAIState::NumberOfTableValues(function_idx + 1);
//                     } else {
//                         // all functions read
//                         state = UAIState::EndOfFile;
//                     }
//                 }
//                 UAIState::EndOfFile => {
//                     // ignore trailing lines
//                     break;
//                 }
//             }
//         }

//         cfn
//     }

//     fn write_to_uai(&self, mut file: File) -> io::Result<()> {
//         // preamble
//         // - graph type, variables and domains
//         let num_variables = self.num_variables();
//         let domain_sizes: Vec<usize> = (0..self.num_variables())
//             .map(|var| self.domain_size(var))
//             .collect();
//         write!(
//             file,
//             "MARKOV\n{}\n{}\n",
//             num_variables,
//             vec_to_string(&domain_sizes)
//         )?;

//         // - function scopes
//         // -- number of functions
//         write!(
//             file,
//             "{}\n",
//             self.num_terms()
//         )?;
//         // -- unary function scopes
//         for var in 0..num_variables {
//             write!(file, "1 {var}\n")?;
//         }
//         // -- binary function scopes
//         for pairwise_term_edge_index in self.cfn_graph.edge_indices() {
//             let (node1, node2) = self
//                 .cfn_graph
//                 .edge_endpoints(pairwise_term_edge_index)
//                 .unwrap();
//             let (var1, var2) = (
//                 self.cfn_graph.node_weight(node1).unwrap(),
//                 self.cfn_graph.node_weight(node2).unwrap(),
//             );
//             let (var1, var2) = match var1 < var2 {
//                 true => (var1, var2),
//                 false => (var2, var1),
//             };
//             write!(file, "2 {var1} {var2}\n")?;
//         }

//         // function tables
//         // - unary function tables
//         for unary_term in &self.unary_terms {
//             // -- blank line, number of table values, table values
//             write!(
//                 file,
//                 "\n{}\n{}\n",
//                 unary_term.costs.len(),
//                 vec_to_string(&unary_term.costs.iter().collect::<Vec<_>>())
//             )?;
//         }
//         // - binary function tables
//         for pairwise_term in &self.pairwise_terms {
//             // -- blank line, number of table values, table values
//             write!(
//                 file,
//                 "\n{}\n{}\n",
//                 pairwise_term.costs.len(),
//                 vec_to_string(&pairwise_term.costs.iter().collect::<Vec<_>>())
//             )?;
//         }

//         Ok(())
//     }
// }

// /// todo: docs
// /// todo: multiple variants and methods
// pub struct FactorGraph {
//     term_node_index: Vec<NodeIndex<usize>>,
//     graph: DiGraph<usize, (), usize>, // node data = index in unary_terms or pairwise_terms (latter: shifted by cfn.num_variables())
// }

// pub struct MinimalEdges;

// pub enum RelaxationType {
//     MinimalEdges,
// }

// pub trait ConstructRelaxation<RelaxationType> {
//     fn construct_relaxation(&mut self) -> FactorGraph;
// }

// impl ConstructRelaxation<MinimalEdges> for PairwiseCFN {
//     fn construct_relaxation(&mut self) -> FactorGraph {
//         let mut factor_graph = DiGraph::with_capacity(
//             self.num_terms(),
//             2 * self.num_non_unary_terms(),
//         );

//         for var_node_index in self.cfn_graph.node_indices() {
//             let &var_index = self.cfn_graph.node_weight(var_node_index).unwrap();
//             self.unary_terms[var_index].factor_graph_node_index =
//                 Some(factor_graph.add_node(var_index));
//         }

//         for pairwise_term_edge_index in self.cfn_graph.edge_indices() {
//             let &pairwise_term_index = self
//                 .cfn_graph
//                 .edge_weight(pairwise_term_edge_index)
//                 .unwrap();
//             // todo: this is very ugly, should probably change how CFN is stored
//             let fg_node_index = factor_graph.add_node(self.num_variables() + pairwise_term_index);
//             self.pairwise_terms[pairwise_term_index].factor_graph_node_index = Some(fg_node_index);

//             let (var1, var2) = self
//                 .cfn_graph
//                 .edge_endpoints(pairwise_term_edge_index)
//                 .unwrap();
//             factor_graph.add_edge(fg_node_index, var1, ());
//             factor_graph.add_edge(fg_node_index, var2, ());
//         }

//         FactorGraph {
//             graph: factor_graph,
//         }
//     }
// }
