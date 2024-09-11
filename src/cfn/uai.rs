#![allow(dead_code)]

use std::{fs::File, io};

/// Interface for reading from and writing to file in UAI format.
/// The format specification can be found [here](https://uaicompetition.github.io/uci-2022/file-formats/model-format/).
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
    FunctionScopes(usize),            // variable index
    NumberOfTableValues(usize),       // function index
    TableValues(usize, usize, usize), // function index, how many entries were read, function table size
    EndOfFile,
}
