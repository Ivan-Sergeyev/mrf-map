#![allow(dead_code)]

use std::{fs::File, io};

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
