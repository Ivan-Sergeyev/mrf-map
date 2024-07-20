mod data_structures {
    pub mod hypergraph;
    pub mod jagged_arrays;
}

mod csp {
    pub mod ac3;
    pub mod binary_csp;
}

mod cfn {
    pub mod cost_function_network;
    pub mod plan;
    pub mod solver;
}

use cfn::cost_function_network::*;
use std::fs::OpenOptions;

fn main() {
    env_logger::init();

    let input_file = OpenOptions::new()
        .read(true)
        .open("problem_instances/grid4x4.UAI.LG")
        .unwrap();
    let cfn = GeneralCFN::read_from_uai(input_file);

    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("problem_instances/output.uai")
        .unwrap();
    cfn.write_to_uai(output_file).unwrap();
}
