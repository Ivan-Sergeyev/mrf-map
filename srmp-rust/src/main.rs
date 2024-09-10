// note: logging can be implemented via [log](https://docs.rs/log/latest/log/) and [env_logger](https://docs.rs/env_logger/latest/env_logger/)
// [example 1](https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/log.html#log-a-debug-message-to-the-console)
// [example 2](https://samkeen.dev/logging-in-rust-a-beginners-guide)

mod algo {
    pub mod factor_sequence_ordering;
    pub mod message_passing;
    pub mod solver;
    pub mod srmp;
}

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
    pub mod factor_types;
    pub mod relaxation;
    pub mod uai;
}

use cfn::{cost_function_network::*, uai::UAI};
use std::fs::OpenOptions;

fn main() {
    env_logger::init();

    let input_file = OpenOptions::new()
        .read(true)
        .open("problem_instances/grid4x4.UAI.LG")
        .unwrap();
    let cfn = GeneralCFN::read_from_uai(input_file, false);

    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("problem_instances/output.uai")
        .unwrap();
    cfn.write_to_uai(output_file, false).unwrap();
}
