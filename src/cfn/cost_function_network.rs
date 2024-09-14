#![allow(dead_code)]

use std::{
    fmt::{Debug, Display},
    fs::File,
    io::{self, BufRead, BufReader, Write},
    mem::swap,
    ops::{Index, IndexMut},
    slice::Iter,
};

use log::{debug, warn};

use crate::{
    cfn::uai::{string_to_vec, vec_to_string},
    factor_types::{factor_trait::Factor, factor_type::FactorType, function_table::FunctionTable},
    message::message_general::GeneralMessage,
};

use crate::cfn::uai::UAIState;

use super::uai::{option_to_string, UAI};

type VariableIndex = usize;
type FactorIndex = usize;

pub enum FactorOrigin {
    Variable(VariableIndex),
    NonUnaryFactor(FactorIndex),
}

// Stores information about a variable in the cost function network
#[derive(Debug)]
pub struct Variable {
    singleton: Vec<usize>, // a one-element vector containing the variable's index
    domain_size: usize,    // the size of the domain of this variable
    factor_index: Option<usize>, // the index of the corresponding unary factor in `factors` (if it exits)
    in_non_unary_factors: Vec<FactorIndex>, // indices of non-unary factors that include this variable
}

// Stores the cost function network
pub struct CostFunctionNetwork {
    variables: Vec<Variable>, // stores structural information about variables in the network
    factors: Vec<FactorType>, // contains numerical representations of all factors (unary and non-unary)
}

impl CostFunctionNetwork {
    // Creates an empty cost function network
    pub fn new() -> Self {
        CostFunctionNetwork {
            variables: Vec::new(),
            factors: Vec::new(),
        }
    }

    // Creates an empty cost function network with reserved capacity for a given number of unary and non-unary factors
    pub fn with_capacity(capacity_unary: usize, capacity_non_unary: usize) -> Self {
        let reserve_capacity = capacity_unary + capacity_non_unary;
        CostFunctionNetwork {
            variables: Vec::with_capacity(capacity_unary),
            factors: Vec::with_capacity(reserve_capacity),
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
        let variables = domain_sizes
            .iter()
            .enumerate()
            .map(|(variable_index, domain_size)| Variable {
                singleton: vec![variable_index],
                domain_size: *domain_size,
                factor_index: None,
                in_non_unary_factors: Vec::new(),
            })
            .collect::<Vec<_>>();
        let reserve_capacity = (reserve_unary as usize) * variables.len() + capacity_non_unary;

        CostFunctionNetwork {
            variables,
            factors: Vec::with_capacity(reserve_capacity),
        }
    }

    // Reserves capacity for at least `additional` more non-unary factors
    pub fn reserve(&mut self, additional: usize) -> &mut Self {
        self.factors.reserve(additional);
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
    }

    // Sets a factor of arbitrary type
    pub fn add_factor(&mut self, factor: FactorType) -> &mut Self {
        assert_eq!(
            factor.arity(),
            factor.variables().len(),
            "Factor's arity doesn't match the number of variables in it."
        );
        assert!(
            factor.variables().windows(2).all(|w| w[0] < w[1]),
            "Variables in a non-unary factor must be distinct and sorted in increasing order."
        );

        match factor.arity() {
            1 => {
                let variable = factor.variables()[0];
                if let Some(unary_factor_index) = self.variables[variable].factor_index {
                    self.factors[unary_factor_index] = factor;
                } else {
                    self.variables[variable].factor_index = Some(self.factors.len());
                    self.factors.push(factor);
                }
            }
            _ => {
                if false {
                    // todo feature: if this factor already exists, overwrite it instead
                    unimplemented!("Overwriting non-unary factors is not currently implemented");
                } else {
                    for variable in factor.variables() {
                        self.variables[*variable]
                            .in_non_unary_factors
                            .push(self.factors.len());
                    }
                    self.factors.push(factor);
                }
            }
        }
        self
    }

    // Returns a factor indicated by its origin (unary or non-unary)
    pub fn get_factor(&self, factor_origin: &FactorOrigin) -> Option<&FactorType> {
        match factor_origin {
            FactorOrigin::Variable(variable_index) => self.variables[*variable_index].factor_index,
            FactorOrigin::NonUnaryFactor(factor_index) => Some(*factor_index),
        }
        .and_then(|factor_index| Some(&self.factors[factor_index]))
    }

    // Returns arity of a given factor (unary or non-unary)
    pub fn arity(&self, factor_origin: &FactorOrigin) -> usize {
        match factor_origin {
            FactorOrigin::Variable(_) => 1,
            FactorOrigin::NonUnaryFactor(factor_index) => self.factors[*factor_index].arity(),
        }
    }

    pub fn factor_variables(&self, factor_origin: &FactorOrigin) -> &Vec<usize> {
        match factor_origin {
            FactorOrigin::Variable(variable_index) => &self.variables[*variable_index].singleton,
            FactorOrigin::NonUnaryFactor(factor_index) => self.factors[*factor_index].variables(),
        }
    }

    pub fn function_table_len(&self, factor_origin: &FactorOrigin) -> usize {
        match factor_origin {
            FactorOrigin::Variable(variable_index) => self.variables[*variable_index].domain_size,
            FactorOrigin::NonUnaryFactor(factor_index) => {
                self.factors[*factor_index].function_table_len()
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
        GeneralMessage::zero_from_len(
            self.get_factor(factor_origin),
            self.function_table_len(factor_origin),
        )
    }

    // Creates new message initialized with contents of a given factor (unary or non-unary)
    pub fn new_message_clone(&self, factor_origin: &FactorOrigin) -> GeneralMessage {
        GeneralMessage::clone_factor(
            self.get_factor(factor_origin),
            self.function_table_len(factor_origin),
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
        self.variables.len()
    }

    // Returns the domain size of a variable
    pub fn domain_size(&self, variable: usize) -> usize {
        self.variables[variable].domain_size
    }

    pub fn factors_iter(&self) -> Iter<FactorType> {
        self.factors.iter()
    }

    // Returns the number of factors in the cost function network
    pub fn factors_len(&self) -> usize {
        self.factors.len()
    }

    // Returns the cost of a solution
    pub fn cost(&self, solution: &Solution) -> f64 {
        self.factors
            .iter()
            .map(|factor| factor.cost(&self, solution))
            .sum()
    }
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

                    // Add factor to cost function network
                    let factor = FactorType::FunctionTable(FunctionTable::new(
                        &cfn,
                        function_scopes[function_idx].to_vec(),
                        function_table,
                    ));
                    cfn.add_factor(factor);

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

        let mapping = [|value: &f64| *value, |value: &f64| value.ln()][lg as usize];

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
        for factor in &self.factors {
            write!(
                file,
                "{} {}\n",
                factor.arity(),
                vec_to_string(factor.variables())
            )?;
        }

        debug!("Writing function tables");
        for factor in self.factors.iter() {
            factor.write_uai(&mut file, mapping)?;
        }

        debug!("UAI export complete");
        Ok(())
    }
}

pub struct Solution {
    labels: Vec<Option<usize>>, // indexed by variables, None = variable is unlabeled, usize = label of variable
}

impl Solution {
    // Creates a new solution with each variable unassigned
    pub fn new(cfn: &CostFunctionNetwork) -> Self {
        Solution {
            labels: vec![None; cfn.num_variables()],
        }
    }

    // Checks if every variable in vec is labeled
    pub fn is_fully_labeled(&self, variables: &Vec<usize>) -> bool {
        variables
            .iter()
            .all(|variable| self.labels[*variable].is_some())
    }

    // Returns number of labeled variables in vec
    pub fn num_labeled(&self, variables: &Vec<usize>) -> usize {
        variables.iter().fold(0, |num_labeled, variable| {
            num_labeled + self.labels[*variable].is_some() as usize
        })
    }

    fn labels_to_vec_string(&self) -> Vec<String> {
        self.labels
            .iter()
            .map(|label| option_to_string(*label))
            .collect::<Vec<_>>()
    }
}

impl Index<usize> for Solution {
    type Output = Option<usize>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.labels[index]
    }
}

impl IndexMut<usize> for Solution {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.labels[index]
    }
}

impl std::fmt::Debug for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.labels_to_vec_string())
    }
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.labels_to_vec_string())
    }
}