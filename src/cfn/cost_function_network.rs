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
    factor_types::{factor_trait::Factor, factor_type::FactorType},
    message::message_general::GeneralMessage,
};

use crate::cfn::uai::UAIState;

use super::{solution::Solution, uai::UAI};

type VariableIndex = usize;
type NonUnaryFactorIndex = usize;

pub enum FactorOrigin {
    Variable(VariableIndex),
    NonUnaryFactor(NonUnaryFactorIndex),
}

// Stores information about a variable in the cost function network
#[derive(Debug)]
pub struct Variable {
    variable: Vec<usize>, // single-element vec containing the index of this variable
    domain_size: usize,   // the domain size of this variable
    factor_index: Option<usize>, // the index of the corresponding unary factor in `factors` (if it exits)
    in_non_unary_factors: Vec<NonUnaryFactorIndex>, // indices of non-unary factors that include this variable
}

// Stores information about a non-unary factor
#[derive(Debug)]
pub struct NonUnaryFactor {
    full_function_table_size: usize, // the product of domain sizes of associated variables
    factor_index: Option<usize>, // the index of the corresponding non-unary factor in `factors` (if it exists)
    variables: Vec<VariableIndex>, // indices of variables included in this factor
}

// Stores the cost function network
pub struct CostFunctionNetwork {
    variables: Vec<Variable>, // stores structural information about variables in the network
    non_unary_factors: Vec<NonUnaryFactor>, // stores structural information about non-unary factors in the network
    factors: Vec<FactorType>,               // contains numerical representations of all factors
    origins: Vec<FactorOrigin>, // indicates where structural information for each factor is stored
}

impl CostFunctionNetwork {
    // Creates an empty cost function network
    pub fn new() -> Self {
        CostFunctionNetwork {
            variables: Vec::new(),
            non_unary_factors: Vec::new(),
            factors: Vec::new(),
            origins: Vec::new(),
        }
    }

    // Creates an empty cost function network with reserved capacity for a given number of unary and non-unary factors
    pub fn with_capacity(capacity_unary: usize, capacity_non_unary: usize) -> Self {
        let reserve_capacity = capacity_unary + capacity_non_unary;
        CostFunctionNetwork {
            variables: Vec::with_capacity(capacity_unary),
            non_unary_factors: Vec::with_capacity(capacity_non_unary),
            factors: Vec::with_capacity(reserve_capacity),
            origins: Vec::with_capacity(reserve_capacity),
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
        let variables: Vec<Variable> = domain_sizes // todo: remove type annotations?
            .iter()
            .enumerate()
            .map(|(index, domain_size)| Variable {
                variable: vec![index],
                domain_size: *domain_size,
                factor_index: None,
                in_non_unary_factors: Vec::new(),
            })
            .collect();
        let reserve_capacity = (reserve_unary as usize) * variables.len() + capacity_non_unary;

        CostFunctionNetwork {
            variables: variables,
            non_unary_factors: Vec::with_capacity(capacity_non_unary),
            factors: Vec::with_capacity(reserve_capacity),
            origins: Vec::with_capacity(reserve_capacity),
        }
    }

    // Creates a cost function network with provided unary function tables,
    // and additionally reserves capacity for a given number of non-unary factors
    pub fn from_unary_function_tables(
        unary_function_tables: Vec<Vec<f64>>,
        capacity_non_unary: usize,
    ) -> Self {
        let variables = unary_function_tables
            .iter()
            .enumerate()
            .map(|(index, unary_function_table)| Variable {
                variable: vec![index],
                domain_size: unary_function_table.len(),
                factor_index: Some(index),
                in_non_unary_factors: Vec::new(),
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
            variables: variables,
            non_unary_factors: Vec::with_capacity(capacity_non_unary),
            factors,
            origins: factor_origins,
        }
    }

    // Reserves capacity for at least `additional` more non-unary factors
    pub fn reserve(&mut self, additional: usize) -> &mut Self {
        self.non_unary_factors.reserve(additional);
        self.factors.reserve(additional);
        self.origins.reserve(additional);
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

    // Adds a unary factor (assuming it does not already exist)
    pub fn add_unary_factor(&mut self, variable: usize, factor: FactorType) -> &mut Self {
        // Assumptions:
        // - `variable` does not have a unary factor associated with it
        // - `factor` has arity 1
        assert!(
            self.variables[variable].factor_index.is_none(),
            "Variable already has an associated unary factor."
        );
        self.variables[variable].factor_index = Some(self.factors.len());
        self.factors.push(factor);
        self.origins.push(FactorOrigin::Variable(variable));
        self
    }

    // Sets a unary factor of a given variable (overwrites it if it already exists, or adds a new one if it does not)
    pub fn set_unary_factor(&mut self, variable: usize, factor: FactorType) -> &mut Self {
        if let Some(unary_factor_index) = self.variables[variable].factor_index {
            self.factors[unary_factor_index] = factor;
            self
        } else {
            self.add_unary_factor(variable, factor)
        }
    }

    // Adds a non-unary factor (assuming it does not already exist)
    pub fn add_non_unary_factor(&mut self, variables: Vec<usize>, factor: FactorType) -> &mut Self {
        // Assumptions:
        // - `variables` is sorted in increasing order
        // - `variables` has at least 2 elements
        // - `factor` has arity equal to len of `variables`
        assert!(
            variables.len() >= 2,
            "A non-unary factor must have at least 2 variables."
        );
        assert!(
            variables.windows(2).all(|w| w[0] < w[1]),
            "Variables in a non-unary factor must be distinct and sorted in increasing order."
        );

        let new_non_unary_factor_index = self.factors.len();
        for variable in &variables {
            self.variables[*variable]
                .in_non_unary_factors
                .push(new_non_unary_factor_index);
        }
        self.non_unary_factors.push(NonUnaryFactor {
            full_function_table_size: self.product_domain_sizes(&variables),
            factor_index: Some(self.factors.len()),
            variables,
        });
        self.factors.push(factor);
        self.origins
            .push(FactorOrigin::NonUnaryFactor(new_non_unary_factor_index));
        self
    }

    // Sets a non-unary factor (overwrites it if it already exsits, or adds a new one if it does not)
    pub fn set_non_unary_factor(&mut self, variables: Vec<usize>, factor: FactorType) -> &mut Self {
        // Assumptions:
        // - `variables` is sorted in increasing order
        // - `variables` has at least 2 elements
        // - `factor` has arity equal to len of `variables`
        if true {
            self.add_non_unary_factor(variables, factor)
        } else {
            // todo feature: if this factor already exists, overwrite it instead
            unimplemented!("Overwriting non-unary factors is not currently implemented");
        }
    }

    // Sets a factor of arbitrary type
    pub fn set_factor(&mut self, variables: Vec<usize>, factor: FactorType) -> &mut Self {
        // Assumption:
        // - `variables` is sorted in increasing order
        // - `factor` has arity equal to len of `variables`
        match variables.len() {
            1 => self.set_unary_factor(variables[0], factor),
            _ => self.set_non_unary_factor(variables, factor),
        }
    }

    // Returns a factor indicated by its origin (unary or non-unary)
    pub fn get_factor(&self, factor_origin: &FactorOrigin) -> Option<&FactorType> {
        match factor_origin {
            FactorOrigin::Variable(variable_index) => self.variables[*variable_index].factor_index,
            FactorOrigin::NonUnaryFactor(non_unary_factor_index) => {
                self.non_unary_factors[*non_unary_factor_index].factor_index
            }
        }
        .and_then(|factor_index| Some(&self.factors[factor_index]))
    }

    // Returns arity of a given factor (unary or non-unary)
    pub fn arity(&self, factor_origin: &FactorOrigin) -> usize {
        match factor_origin {
            FactorOrigin::Variable(_) => 1,
            FactorOrigin::NonUnaryFactor(non_unary_factor_index) => self.non_unary_factors
                [*non_unary_factor_index]
                .variables
                .len(),
        }
    }

    // Returns variables associated with a given factor (unary or non-unary)
    pub fn factor_variables(&self, factor_origin: &FactorOrigin) -> &Vec<usize> {
        match factor_origin {
            FactorOrigin::Variable(variable_index) => &self.variables[*variable_index].variable,
            FactorOrigin::NonUnaryFactor(non_unary_factor_index) => {
                &self.non_unary_factors[*non_unary_factor_index].variables
            }
        }
    }

    // Returns product of domain sizes of variables associated with a given factor (unary or non-unary)
    pub fn full_function_table_size(&self, factor_origin: &FactorOrigin) -> usize {
        match factor_origin {
            FactorOrigin::Variable(variable_index) => self.variables[*variable_index].domain_size,
            FactorOrigin::NonUnaryFactor(non_unary_factor_index) => {
                self.non_unary_factors[*non_unary_factor_index].full_function_table_size
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
            self.full_function_table_size(factor_origin),
        )
    }

    // Creates new message initialized with contents of a given factor (unary or non-unary)
    pub fn new_message_clone(&self, factor_origin: &FactorOrigin) -> GeneralMessage {
        GeneralMessage::clone_factor(
            self.get_factor(factor_origin),
            self.full_function_table_size(factor_origin),
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

    // Returns the number of non-unary factors in the cost function network
    pub fn num_non_unary_factors(&self) -> usize {
        self.non_unary_factors.len()
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
        for variable in &self.variables {
            if let Some(index) = variable.factor_index {
                cost += self.factors[index].cost(&self, solution, &variable.variable);
            }
        }

        // Add costs of all non-unary factors (that exist)
        for non_unary_factor in &self.non_unary_factors {
            if let Some(index) = non_unary_factor.factor_index {
                cost += self.factors[index].cost(&self, solution, &non_unary_factor.variables);
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
        for factor in &self.origins {
            // Number of variables, list of variables
            match factor {
                FactorOrigin::Variable(variable_index) => {
                    write!(file, "1 {}\n", variable_index)?;
                }
                FactorOrigin::NonUnaryFactor(non_unary_factor_index) => {
                    let variables = &self.non_unary_factors[*non_unary_factor_index].variables;
                    let num_variables = variables.len();
                    write!(file, "{} {}\n", num_variables, vec_to_string(variables))?;
                }
            }
        }

        debug!("Writing function tables");
        for factor in self.factors.iter() {
            factor.write_uai(&mut file, mapping)?;
        }

        debug!("UAI export complete");
        Ok(())
    }
}
