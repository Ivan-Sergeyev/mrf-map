#![allow(dead_code)]

use std::{
    fmt::Debug,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    mem::swap,
    str::FromStr,
};

use log::{debug, warn};

use crate::{
    data_structures::hypergraph::Hypergraph,
    factor_types::{factor_trait::Factor, factor_type::FactorType},
    message::message_general::GeneralMessage,
};

use std::slice::Iter;

use crate::{
    cfn::uai::UAIState, data_structures::hypergraph::UndirectedHypergraph,
    factor_types::unary_factor::UnaryFactor,
};

use super::{solution::Solution, uai::UAI};

// Stores information about a variable in the cost function network
#[derive(Debug)]
pub struct CFNVariable {
    variable: Vec<usize>, // single-element vec containing the index of this variable
    domain_size: usize,   // the domain size of this variable
    factor_index: Option<usize>, // the index of the corresponding unary factor in the `factors` vec (if it exits)
}

// Stores information about a non-unary factor
#[derive(Debug)]
pub struct CFNNonUnaryFactor {
    max_function_table_size: usize, // the product of domain sizes of associated variables
    factor_index: Option<usize>, // the index of the corresponding non-unary factor in the `factors` vec (if it exists)
}

type HNodeIndex = usize;
type HHyperedgeIndex = usize;

pub enum FactorOrigin {
    Variable(HNodeIndex),
    NonUnary(HHyperedgeIndex),
}

// Stores the cost function network
pub struct CostFunctionNetwork {
    hypergraph: UndirectedHypergraph<CFNVariable, CFNNonUnaryFactor>, // stores the structure of the network,
    // namely relations between factors with variables, together with the information in the corresponding data structs
    factors: Vec<FactorType>, // contains numerical representations of all factors
    factor_origins: Vec<FactorOrigin>, // indicates what each factor corresponds to in `hypergraph` (node or hyperedge)
}

impl CostFunctionNetwork {
    // Creates an empty cost function network
    pub fn new() -> Self {
        CostFunctionNetwork {
            hypergraph: UndirectedHypergraph::with_capacity(0, 0),
            factors: Vec::new(),
            factor_origins: Vec::new(),
        }
    }

    // Creates an empty cost function network with reserved capacity for a given number of unary and non-unary factors
    pub fn with_capacity(capacity_unary: usize, capacity_non_unary: usize) -> Self {
        let reserve_capacity = capacity_unary + capacity_non_unary;
        CostFunctionNetwork {
            hypergraph: UndirectedHypergraph::with_capacity(capacity_unary, capacity_non_unary),
            factors: Vec::with_capacity(reserve_capacity),
            factor_origins: Vec::with_capacity(reserve_capacity),
        }
    }

    // Creates an empty cost function network with provided domain sizes,
    // optionally reserves capacity for unary factors,
    // and additionally reserves capacity for a given number of non-unary factors
    pub fn from_domain_sizes(
        domain_sizes: &Vec<usize>,
        reserve_unary: bool,
        capacity_non_unary: usize,
    ) -> Self {
        let node_data: Vec<CFNVariable> = domain_sizes // todo: remove type annotations?
            .iter()
            .enumerate()
            .map(|(index, domain_size)| CFNVariable {
                variable: vec![index],
                domain_size: *domain_size,
                factor_index: None,
            })
            .collect();
        let reserve_capacity = (reserve_unary as usize) * node_data.len() + capacity_non_unary;

        CostFunctionNetwork {
            hypergraph: UndirectedHypergraph::from_node_data(node_data, 0),
            factors: Vec::with_capacity(reserve_capacity),
            factor_origins: Vec::with_capacity(reserve_capacity),
        }
    }

    // Creates a cost function network with provided unary function tables,
    // and additionally reserves capacity for a given number of non-unary factors
    pub fn from_unary_function_tables(
        unary_function_tables: Vec<Vec<f64>>,
        capacity_non_unary: usize,
    ) -> Self {
        let node_data = unary_function_tables
            .iter()
            .enumerate()
            .map(|(index, unary_function_table)| CFNVariable {
                variable: vec![index],
                domain_size: unary_function_table.len(),
                factor_index: Some(index),
            })
            .collect();

        let mut factor_origins: Vec<FactorOrigin> = (0..unary_function_tables.len())
            .map(|index| FactorOrigin::Variable(index))
            .collect();
        factor_origins.reserve(capacity_non_unary);

        let mut factors: Vec<FactorType> = unary_function_tables
            .into_iter()
            .map(|unary_function_table| FactorType::Unary(unary_function_table.into()))
            .collect();
        factors.reserve(capacity_non_unary);

        CostFunctionNetwork {
            hypergraph: UndirectedHypergraph::from_node_data(node_data, capacity_non_unary),
            factors,
            factor_origins,
        }
    }

    // Reserves capacity for at least `additional` more non-unary factors
    pub fn reserve(&mut self, additional: usize) -> &mut Self {
        self.factors.reserve(additional);
        self.factor_origins.reserve(additional);
        self
    }

    // Computes the product of domain sizes of given variables
    fn product_domain_sizes(&self, variables: &Vec<usize>) -> usize {
        variables
            .iter()
            .map(|variable| self.domain_size(*variable))
            .product()
    }

    // Computes the product of domain sizes of given variables, alternative implementation
    // todo: bench against product_domain_sizes()
    fn product_domain_sizes_alt(&self, variables: &Vec<usize>) -> usize {
        variables
            .iter()
            .fold(1, |product, variable| product * self.domain_size(*variable))
        // alternative:
        // todo: bench both functions to find which implementation is faster
    }

    // Adds a unary factor (assuming it does not already exist)
    pub fn add_unary_factor(&mut self, variable: usize, factor: UnaryFactor) -> &mut Self {
        // Assumption: `variable` does not have a unary factor associated with it
        assert!(
            self.hypergraph.node_data(variable).factor_index.is_none(),
            "Variable has no associated unary factor"
        );
        self.hypergraph.node_data_mut(variable).factor_index = Some(self.factors.len());
        self.factors.push(FactorType::Unary(factor));
        self.factor_origins.push(FactorOrigin::Variable(variable));
        self
    }

    // Sets a unary factor of a given variable (overwrites it if it already exists, or adds a new one if it does not)
    pub fn set_unary_factor(&mut self, variable: usize, factor: UnaryFactor) -> &mut Self {
        if let Some(unary_factor_index) = self.hypergraph.node_data(variable).factor_index {
            self.factors[unary_factor_index] = FactorType::Unary(factor);
            self
        } else {
            self.add_unary_factor(variable, factor)
        }
    }

    // Adds a non-unary factor (assuming it does not already exist)
    pub fn add_non_unary_factor(&mut self, variables: Vec<usize>, factor: FactorType) -> &mut Self {
        // Assumptions:
        // - `variables` is sorted in increasing order
        // - `non_unary_factor` is not Unary
        let hyperedge_data = CFNNonUnaryFactor {
            max_function_table_size: self.product_domain_sizes(&variables),
            factor_index: Some(self.factors.len()),
        };
        let hyperedge_index = self.hypergraph.add_hyperedge(variables, hyperedge_data);
        self.factors.push(factor);
        self.factor_origins
            .push(FactorOrigin::NonUnary(hyperedge_index));
        self
    }

    // Sets a non-unary factor (overwrites it if it already exsits, or adds a new one if it does not)
    pub fn set_non_unary_factor(
        &mut self,
        variables: Vec<usize>,
        non_unary_factor: FactorType,
    ) -> &mut Self {
        // Assumptions:
        // - `variables` is sorted in increasing order
        // - `non_unary_factor` is not Unary
        if true {
            self.add_non_unary_factor(variables, non_unary_factor)
        } else {
            // todo feature: if this factor already exists, overwrite it instead
            unimplemented!("Overwriting non-unary factors is not currently implemented");
        }
    }

    // Sets a factor of arbitrary type
    pub fn set_factor(&mut self, variables: Vec<usize>, factor: FactorType) -> &mut Self {
        // Assumption:
        // - `variables` is sorted in increasing order
        // - arity of `factor` is the same as number of elements in `variables`
        match factor {
            FactorType::Unary(unary_factor) => {
                assert_eq!(
                    variables.len(),
                    1,
                    "More than one variable provided for setting unary factor"
                );
                self.set_unary_factor(variables[0], unary_factor)
            }
            _ => self.set_non_unary_factor(variables, factor),
        }
    }

    // Returns an iterator over factors
    pub fn factors_iter(&self) -> Iter<FactorType> {
        self.factors.iter()
    }

    // Returns an iterator over factor origins
    pub fn factor_origins_iter(&self) -> Iter<FactorOrigin> {
        self.factor_origins.iter()
    }

    // Returns a factor indicated by its origin (unary or non-unary)
    pub fn get_factor(&self, factor_origin: &FactorOrigin) -> Option<&FactorType> {
        match factor_origin {
            FactorOrigin::Variable(node_index) => {
                self.hypergraph.node_data(*node_index).factor_index
            }
            FactorOrigin::NonUnary(hyperedge_index) => {
                self.hypergraph
                    .hyperedge_data(*hyperedge_index)
                    .factor_index
            }
        }
        .and_then(|factor_index| Some(&self.factors[factor_index]))
    }

    // Returns arity of a given factor (unary or non-unary)
    pub fn arity(&self, factor_origin: &FactorOrigin) -> usize {
        match factor_origin {
            FactorOrigin::Variable(_) => 1,
            FactorOrigin::NonUnary(hyperedge_index) => {
                self.hyperedge_variables(*hyperedge_index).len()
            }
        }
    }

    // Returns variables associated with a given factor (unary or non-unary)
    pub fn factor_variables(&self, factor_origin: &FactorOrigin) -> &Vec<usize> {
        match factor_origin {
            FactorOrigin::Variable(node_index) => &self.hypergraph.node_data(*node_index).variable,
            FactorOrigin::NonUnary(hyperedge_index) => self.hyperedge_variables(*hyperedge_index),
        }
    }

    // Returns variables associated with a given non-unary factor (indicated by its hyperedge index)
    pub fn hyperedge_variables(&self, hyperedge_index: usize) -> &Vec<usize> {
        self.hypergraph.hyperedge_endpoints(hyperedge_index)
    }

    // Returns product of domain sizes of variables associated with a given factor (unary or non-unary)
    pub fn max_function_table_size(&self, factor_origin: &FactorOrigin) -> usize {
        match factor_origin {
            FactorOrigin::Variable(node_index) => {
                self.hypergraph.node_data(*node_index).domain_size
            }
            FactorOrigin::NonUnary(hyperedge_index) => {
                self.hypergraph
                    .hyperedge_data(*hyperedge_index)
                    .max_function_table_size
            }
        }
    }

    // Returns a list of variables contained in the first factor and not the second,
    // assuming the first fully contains the second
    pub fn get_variables_difference(
        &self,
        alpha: &FactorOrigin,
        beta: &FactorOrigin,
    ) -> Vec<usize> {
        // Assumption: `alpha` contains `beta`
        let alpha_variables = self.factor_variables(alpha);
        let beta_variables = self.factor_variables(beta);
        let mut difference = Vec::with_capacity(alpha_variables.len() - beta_variables.len());
        let mut var_b_iter = beta_variables.iter().peekable();
        for &var_a in alpha_variables {
            if var_b_iter.peek().is_some_and(|var_b| **var_b == var_a) {
                var_b_iter.next();
            } else {
                difference.push(var_a);
            }
        }
        difference
    }

    // Creates new zero message corresponding to a given factor (unary or non-unary)
    pub fn new_zero_message(&self, factor_origin: &FactorOrigin) -> GeneralMessage {
        GeneralMessage::zero_from_size(
            self.get_factor(factor_origin),
            self.max_function_table_size(factor_origin),
        )
    }

    // Creates new message initialized with contents of a given factor (unary or non-unary)
    pub fn new_message_clone(&self, factor_origin: &FactorOrigin) -> GeneralMessage {
        GeneralMessage::clone_factor(
            self.get_factor(factor_origin),
            self.max_function_table_size(factor_origin),
        )
    }

    // Applies a mapping to all factors
    pub fn map_factors_inplace(&mut self, mapping: fn(&mut f64)) -> &mut Self {
        self.factors
            .iter_mut()
            .for_each(|factor| factor.map_inplace(mapping));
        self
    }

    // Returns the number of variables in the cost function network
    pub fn num_variables(&self) -> usize {
        self.hypergraph.num_nodes()
    }

    // Returns the domain size of a variable
    pub fn domain_size(&self, variable: usize) -> usize {
        self.hypergraph.node_data(variable).domain_size
    }

    // Returns the number of non-unary factors in the cost function network
    pub fn num_hyperedges(&self) -> usize {
        self.hypergraph.num_hyperedges()
    }

    // Returns the number of factors in the cost function network
    pub fn factors_len(&self) -> usize {
        self.factors.len()
    }

    // Returns the cost of a solution
    pub fn cost(&self, solution: &Solution) -> f64 {
        // Start with zero cost
        let mut cost = 0.;

        // Add costs of all unary factors (that exist)
        for node_index in self.hypergraph.nodes_iter() {
            let variable_data = self.hypergraph.node_data(node_index);
            if let Some(index) = variable_data.factor_index {
                cost += self.factors[index].cost(&self, solution, &variable_data.variable);
            }
        }

        // Add costs of all non-unary factors (that exist)
        for hyperedge_index in self.hypergraph.hyperedges_iter() {
            let factor_data = self.hypergraph.hyperedge_data(hyperedge_index);
            if let Some(index) = factor_data.factor_index {
                cost += self.factors[index].cost(
                    &self,
                    solution,
                    self.hypergraph.hyperedge_endpoints(hyperedge_index),
                );
            }
        }

        cost
    }
}

fn string_to_vec<T>(string: &str) -> Vec<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    string
        .split_whitespace()
        .map(|s| s.parse::<T>().unwrap())
        .collect()
}

fn vec_to_string<T: ToString>(v: &Vec<T>) -> String {
    v.iter()
        .map(|elem| elem.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

impl UAI for CostFunctionNetwork {
    fn read_uai(file: File, lg: bool) -> Self {
        debug!("In read_uai() for file {:?} with lg option {}", file, lg);

        let mut state = UAIState::ModelType;

        let lines = BufReader::new(file).lines();
        let mut trimmed_line;

        let mut cfn = CostFunctionNetwork::new();

        let mut num_variables = 0;
        let mut domain_sizes = Vec::new();
        let mut function_scopes = Vec::new();
        let mut function_entries = Vec::new();

        for line in lines {
            let line = line.unwrap();
            trimmed_line = line.trim();

            // Skip empty lines
            if trimmed_line.is_empty() {
                continue;
            }

            match state {
                UAIState::ModelType => {
                    debug!("Reading model type");
                    if trimmed_line != "MARKOV" {
                        unimplemented!("Only MARKOV graph type is supported.");
                    }
                    state = UAIState::NumberOfVariables;
                }
                UAIState::NumberOfVariables => {
                    debug!("Reading number of variables");
                    num_variables = trimmed_line.parse::<usize>().unwrap();
                    state = UAIState::DomainSizes;
                }
                UAIState::DomainSizes => {
                    debug!("Reading domain sizes");
                    domain_sizes = string_to_vec(trimmed_line);
                    assert_eq!(num_variables, domain_sizes.len());
                    state = UAIState::NumberOfFunctions;
                }
                UAIState::NumberOfFunctions => {
                    debug!("Reading number of functions");
                    let num_functions = trimmed_line.parse::<usize>().unwrap();
                    let capacity_non_unary = if num_functions > num_variables {
                        num_functions - num_variables
                    } else {
                        0
                    };
                    cfn = CostFunctionNetwork::from_domain_sizes(
                        &domain_sizes,
                        true,
                        capacity_non_unary,
                    );
                    function_scopes = Vec::with_capacity(num_functions);
                    state = UAIState::FunctionScopes(0);
                }
                UAIState::FunctionScopes(function_idx) => {
                    debug!("Reading scope of function {}", function_idx);
                    let function_desc = string_to_vec(trimmed_line);
                    let (scope_len, function_scope) = function_desc.split_at(1);
                    assert_eq!(scope_len[0], function_scope.len());
                    function_scopes.push(function_scope.to_vec());
                    state = if function_idx + 1 < function_scopes.capacity() {
                        UAIState::FunctionScopes(function_idx + 1)
                    } else {
                        UAIState::NumberOfTableValues(0)
                    };
                }
                UAIState::NumberOfTableValues(function_idx) => {
                    debug!("Reading function table size of function {}", function_idx);
                    assert!(function_idx < function_scopes.len());
                    let num_entries = trimmed_line.parse::<usize>().unwrap();
                    function_entries = Vec::with_capacity(num_entries);
                    state = UAIState::TableValues(function_idx, 0, num_entries);
                }
                UAIState::TableValues(function_idx, cur_entries, num_entries) => {
                    debug!(
                        "Reading function {}, collected {} out of {} entries",
                        function_idx, cur_entries, num_entries
                    );
                    assert!(function_idx < function_scopes.len());
                    let mut new_entries = string_to_vec(trimmed_line);
                    let new_cur_entries = cur_entries + new_entries.len();
                    function_entries.append(&mut new_entries);

                    assert!(new_cur_entries <= num_entries, "Too many entries");
                    if new_cur_entries < num_entries {
                        // Continue collecting function table entries
                        state = UAIState::TableValues(function_idx, new_cur_entries, num_entries);
                        continue;
                    }

                    debug!(
                        "Reading function {}, collected all {} entries, saving",
                        function_idx, num_entries
                    );
                    // Move values into a separate vectors
                    let mut function_table = Vec::new();
                    swap(&mut function_entries, &mut function_table);

                    // Prepare factor
                    let factor = match function_scopes[function_idx].len() {
                        1 => FactorType::Unary(function_table.into()),
                        _ => FactorType::General(function_table.into()),
                    };

                    // Add factor to cost function network
                    cfn.set_factor(function_scopes[function_idx].to_vec(), factor);

                    state = if function_idx + 1 < function_scopes.len() {
                        UAIState::NumberOfTableValues(function_idx + 1)
                    } else {
                        UAIState::EndOfFile
                    };
                }
                UAIState::EndOfFile => {
                    warn!("Ignoring trailing line at the end of file: {}", line);
                }
            }
        }

        if lg {
            debug!("LG flag is {}, exponentiating all function tables", lg);
            cfn.map_factors_inplace(|value: &mut f64| *value = value.exp());
        }

        debug!("UAI import complete");
        cfn
    }

    fn write_uai(&self, mut file: File, lg: bool) -> io::Result<()> {
        debug!("In write_uai() for file {:?} with lg option {}", file, lg);

        let mapping = [|value| value, |value: f64| value.ln()][lg as usize];

        debug!("Writing preamble: graph type, variables, and domain sizes");
        let graph_type = "MARKOV";
        let num_variables = self.num_variables();
        let domain_sizes: Vec<usize> = (0..num_variables)
            .map(|var| self.domain_size(var))
            .collect();
        write!(
            file,
            "{}\n{}\n{}\n",
            graph_type,
            num_variables,
            vec_to_string(&domain_sizes)
        )?;

        debug!("Writing number of functions");
        write!(file, "{}\n", self.factors_len())?;

        debug!("Writing function scopes");
        for factor in &self.factor_origins {
            // Number of variables, list of variables
            match factor {
                FactorOrigin::Variable(node_index) => {
                    write!(file, "1 {}\n", node_index)?;
                }
                FactorOrigin::NonUnary(hyperedge_index) => {
                    let variables = self.hyperedge_variables(*hyperedge_index);
                    let num_variables = variables.len();
                    write!(file, "{} {}\n", num_variables, vec_to_string(variables))?;
                }
            }
        }

        debug!("Writing function tables");
        for factor in self.factors_iter() {
            factor.write_uai(&mut file, mapping)?;
        }

        debug!("UAI export complete");
        Ok(())
    }
}
