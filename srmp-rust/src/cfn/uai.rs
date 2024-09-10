#![allow(dead_code)]

use std::{
    fmt::Debug,
    fs::File,
    io::{self, BufRead, BufReader, Write},
    str::FromStr,
};

use ndarray::Array;

use crate::{
    data_structures::hypergraph::Hypergraph,
    factor_types::{factor_trait::Factor, factor_type::FactorType},
    CostFunctionNetwork, FactorOrigin, GeneralCFN,
};

/// model format: https://uaicompetition.github.io/uci-2022/file-formats/model-format/
pub trait UAI
where
    Self: CostFunctionNetwork,
{
    fn read_from_uai(file: File, lg: bool) -> Self;
    fn write_to_uai(&self, file: File, lg: bool) -> io::Result<()>;
}

pub enum UAIState {
    ModelType,
    NumberOfVariables,
    DomainSizes,
    NumberOfFunctions,
    FunctionScopes(usize),
    NumberOfTableValues(usize),
    TableValues(usize, usize),
    EndOfFile,
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

impl UAI for GeneralCFN {
    fn read_from_uai(file: File, lg: bool) -> Self {
        let lines = BufReader::new(file).lines();

        let mut state = UAIState::ModelType;
        let mut trimmed_line;

        let mut num_variables = 0;
        let mut cfn = GeneralCFN::new();
        let mut function_scopes = Vec::new();
        let mut function_entries = Vec::new();

        for line in lines {
            let line = line.unwrap();
            trimmed_line = line.trim();

            // skip empty lines
            if trimmed_line.is_empty() {
                continue;
            }

            match state {
                UAIState::ModelType => {
                    if trimmed_line != "MARKOV" {
                        unimplemented!("Only MARKOV graph type is supported.");
                    }
                    state = UAIState::NumberOfVariables;
                }
                UAIState::NumberOfVariables => {
                    num_variables = trimmed_line.parse::<usize>().unwrap();
                    state = UAIState::DomainSizes;
                }
                UAIState::DomainSizes => {
                    let domain_sizes = string_to_vec(trimmed_line);
                    assert_eq!(num_variables, domain_sizes.len());
                    cfn = GeneralCFN::from_domain_sizes(domain_sizes);
                    state = UAIState::NumberOfFunctions;
                }
                UAIState::NumberOfFunctions => {
                    let num_functions = trimmed_line.parse::<usize>().unwrap();
                    function_scopes = Vec::with_capacity(num_functions);
                    state = UAIState::FunctionScopes(0);
                }
                UAIState::FunctionScopes(function_idx) => {
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
                    assert!(function_idx < function_scopes.len());
                    let num_entries = trimmed_line.parse::<usize>().unwrap();
                    function_entries = Vec::with_capacity(num_entries);
                    state = UAIState::TableValues(function_idx, num_entries);
                }
                UAIState::TableValues(function_idx, max_num_entries) => {
                    assert!(function_idx < function_scopes.len());
                    let mut new_entries = string_to_vec(trimmed_line);
                    function_entries.append(&mut new_entries);

                    let cur_num_entries = function_entries.len();
                    assert!(cur_num_entries <= max_num_entries);
                    if cur_num_entries < max_num_entries {
                        // need to collect more table entries
                        state = UAIState::TableValues(function_idx, max_num_entries);
                        continue;
                    }

                    // collected all table entries, ready to add factor to cost function network
                    let function_table = Array::from_shape_vec(
                        function_scopes[function_idx]
                            .iter()
                            .map(|&var| cfn.domain_size(var))
                            .collect::<Vec<usize>>(),
                        function_entries.drain(..).collect(),
                    )
                    .unwrap();
                    let factor = match function_scopes[function_idx].len() {
                        1 => FactorType::Unary(function_table.into()),
                        _ => FactorType::General(function_table.into()),
                    };
                    cfn = cfn.set_factor(function_scopes[function_idx].to_vec(), factor);

                    state = if function_idx + 1 < function_scopes.len() {
                        UAIState::NumberOfTableValues(function_idx + 1)
                    } else {
                        UAIState::EndOfFile
                    };
                }
                UAIState::EndOfFile => {
                    break;
                }
            }
        }

        let mapping = [|_: &mut f64| {}, |value: &mut f64| *value = value.exp()][lg as usize];
        cfn.map_factors_inplace(mapping)
    }

    fn write_to_uai(&self, mut file: File, lg: bool) -> io::Result<()> {
        let mapping = [|value| value, |value: f64| value.ln()][lg as usize];

        // preamble
        // - graph type, variables and domains
        let num_variables = self.num_variables();
        let domain_sizes: Vec<usize> = (0..self.num_variables())
            .map(|var| self.domain_size(var))
            .collect();
        write!(
            file,
            "MARKOV\n{}\n{}\n",
            num_variables,
            vec_to_string(&domain_sizes)
        )?;

        // - function scopes
        // -- number of functions
        write!(file, "{}\n", self.num_factors())?;
        // -- function scopes
        for factor in &self.factor_origins {
            // ---- number of variables, list of variables
            match factor {
                FactorOrigin::Variable(node_index) => {
                    write!(file, "1 {}\n", node_index)?;
                }
                FactorOrigin::NonUnary(hyperedge_index) => {
                    let variables = self.hypergraph.hyperedge_endpoints(*hyperedge_index);
                    let num_variables = variables.len();
                    write!(file, "{} {}\n", num_variables, vec_to_string(variables))?;
                }
            }
        }

        // function tables
        for factor in self.factors_iter() {
            // -- blank line, number of table values, table values
            match factor {
                FactorType::Unary(factor) => write!(
                    file,
                    "\n{}\n{}\n",
                    factor.function_table.len(),
                    factor.map(mapping).to_string()
                )?,
                FactorType::General(factor) => write!(
                    file,
                    "\n{}\n{}\n",
                    factor.function_table.len(),
                    factor.map(mapping).to_string()
                )?,
            }
        }

        Ok(())
    }
}
