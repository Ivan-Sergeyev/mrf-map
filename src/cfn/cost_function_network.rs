#![allow(dead_code)]

use std::{
    borrow::Cow,
    fmt::Debug,
    fs::OpenOptions,
    io::{self, BufRead, BufReader, Write},
    mem,
    path::PathBuf,
    slice::Iter,
    time::Instant,
};

use log::{debug, warn};

use crate::{
    cfn::uai::{string_to_vec, vec_to_string},
    factors::{factor_trait::Factor, factor_type::FactorType, function_table::FunctionTable},
};

use crate::cfn::uai::UAIState;

use super::uai::UAI;

type VariableIndex = usize;
type FactorIndex = usize;

// Shows if a factor is unary or non-unary factors and stores the corresponding index
pub enum FactorOrigin {
    Variable(VariableIndex),
    NonUnaryFactor(FactorIndex),
}

// Stores information about a variable in the cost function network
#[derive(Debug)]
pub struct Variable {
    domain_size: usize,          // the size of the domain of this variable
    factor_index: Option<usize>, // the index of the corresponding unary factor in `factors` (if it exits)
}

// Stores a cost function network
pub struct CostFunctionNetwork {
    variables: Vec<Variable>, // stores information about variables in the network
    factors: Vec<FactorType>, // stores representations of all factors (unary and non-unary)
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
            .map(|domain_size| Variable {
                domain_size: *domain_size,
                factor_index: None,
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
                    // store the following field in Variable struct and use it to implement this feature:
                    // in_non_unary_factors: Vec<FactorIndex>, // indices of non-unary factors that include this variable
                    unimplemented!("Overwriting non-unary factors is not currently implemented");
                } else {
                    self.factors.push(factor);
                }
            }
        }
        self
    }

    // Returns the factor indicated by its origin (unary or non-unary)
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

    // Returns a reference to the Vec of variables associated with a given factor
    pub fn factor_variables(&self, factor_origin: &FactorOrigin) -> Cow<Vec<usize>> {
        match factor_origin {
            FactorOrigin::Variable(variable_index) => Cow::Owned(vec![*variable_index]),
            FactorOrigin::NonUnaryFactor(factor_index) => {
                Cow::Borrowed(self.factors[*factor_index].variables())
            }
        }
    }

    // Returns the length
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
        for &var_a in alpha_variables.iter() {
            if var_b_iter.peek().is_some_and(|var_b| **var_b == var_a) {
                var_b_iter.next();
            } else {
                difference.push(var_a);
            }
        }
        difference
    }

    // Returns the number of variables in the cost function network
    pub fn num_variables(&self) -> usize {
        self.variables.len()
    }

    // Returns the domain size of a variable
    pub fn domain_size(&self, variable: usize) -> usize {
        self.variables[variable].domain_size
    }

    // Returns an iterator over all factors
    pub fn factors_iter(&self) -> Iter<FactorType> {
        self.factors.iter()
    }

    // Returns the number of factors in the cost function network
    pub fn factors_len(&self) -> usize {
        self.factors.len()
    }
}

impl UAI for CostFunctionNetwork {
    fn read_uai(path: PathBuf, lg: bool) -> Self {
        debug!("In read_uai() for file {:?} with lg option {}", path, lg);

        let file = OpenOptions::new().read(true).open(path).unwrap();

        let mut state = UAIState::ModelType;

        let lines = BufReader::new(file).lines();
        let mut trimmed_line;

        // Flip signs for UAI, exponentiate and flip signs for LG
        let mapping = [
            |value: &mut f64| *value *= -1.,
            |value: &mut f64| *value = -(value.exp()),
        ][lg as usize];

        let mut cfn = CostFunctionNetwork::new();

        let mut num_variables = 0;
        let mut domain_sizes = Vec::new();
        let mut function_scopes = Vec::new();
        let mut function_entries = Vec::new();

        for line in lines {
            let line = line.unwrap();
            trimmed_line = line.trim();

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
                    assert!(
                        function_idx < function_scopes.len(),
                        "Function index out of bounds."
                    );
                    let mut new_entries = string_to_vec(trimmed_line);
                    let new_cur_entries = cur_entries + new_entries.len();
                    function_entries.append(&mut new_entries);

                    assert!(new_cur_entries <= num_entries, "Too many entries.");
                    if new_cur_entries < num_entries {
                        debug!(
                            "Reading function {}. Collected {} out of {} entries.",
                            function_idx, cur_entries, num_entries
                        );
                        state = UAIState::TableValues(function_idx, new_cur_entries, num_entries);
                        continue;
                    }
                    debug!(
                        "Reading function {}. Collected all {} entries.",
                        function_idx, num_entries
                    );

                    // Move values into a separate vector
                    let mut function_table = Vec::new();
                    mem::swap(&mut function_entries, &mut function_table);

                    // Apply mapping (flip signs for UAI, exponentiate and flip signs for LG)
                    function_table.iter_mut().for_each(|value| mapping(value));

                    // Create factor from function table and add it to the cost function network
                    let factor = FactorType::FunctionTable(FunctionTable::new(
                        &cfn,
                        function_scopes[function_idx].to_vec(),
                        function_table,
                    ));
                    cfn.add_factor(factor);

                    // Proceed to the next function
                    state = if function_idx + 1 < function_scopes.len() {
                        UAIState::NumberOfTableValues(function_idx + 1)
                    } else {
                        UAIState::EndOfFile
                    };
                }
                UAIState::EndOfFile => {
                    warn!("Ignored trailing line at the end of file: {}", line);
                }
            }
        }

        debug!("UAI import complete.");

        cfn
    }

    fn write_uai(&self, path: PathBuf, lg: bool) -> io::Result<()> {
        debug!("In write_uai() for file {:?} with lg option {}", path, lg);

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open("problem_instances/output.uai")
            .unwrap();

        let time_start = Instant::now();
        let mapping = [|value: &f64| -*value, |value: &f64| (-value).ln()][lg as usize];

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

        let elapsed_time = time_start.elapsed();
        debug!("UAI export complete. Elapsed time {:?}.", elapsed_time);
        Ok(())
    }
}
