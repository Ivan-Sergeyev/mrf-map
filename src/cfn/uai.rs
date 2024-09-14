#![allow(dead_code)]

use std::{fmt::Debug, fs::File, io, str::FromStr};

/// Interface for reading from and writing to file in UAI format.
/// The format specification can be found
/// - [here](https://uaicompetition.github.io/uci-2022/file-formats/model-format/)
/// - [here](https://toulbar2.github.io/toulbar2/formats/uailgformat.html)
/// - [here](https://www.cs.huji.ac.il/project/PASCAL/fileFormat.php)
/// If `lg` is set to true, use the LG format, where all probabilities are replaced by their logarithm.
pub trait UAI {
    fn read_uai(file: File, lg: bool) -> Self;
    fn write_uai(&self, file: File, lg: bool) -> io::Result<()>;
}

// States for reading UAI files
pub enum UAIState {
    ModelType,
    NumberOfVariables,
    DomainSizes,
    NumberOfFunctions,
    FunctionScopes(usize),            // stores variable index
    NumberOfTableValues(usize),       // stores function index
    TableValues(usize, usize, usize), // stores function index, how many entries were read, and function table size
    EndOfFile,
}

pub fn string_to_vec<T>(string: &str) -> Vec<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    string
        .split_whitespace()
        .map(|s| s.parse::<T>().unwrap())
        .collect()
}

pub fn repeat_float_to_string(repeat: usize, value: f64) -> String {
    (0..repeat)
        .map(|_| value.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn vec_to_string<T: ToString>(vec: &Vec<T>) -> String {
    vec.iter()
        .map(|elem| elem.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn vec_mapping_to_string<T: ToString>(vec: &Vec<T>, mapping: fn(&T) -> T) -> String {
    vec.iter()
        .map(|elem| mapping(elem).to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn option_to_string<T: ToString>(option: Option<T>) -> String {
    match option {
        Some(value) => value.to_string(),
        None => "None".to_string(),
    }
}
