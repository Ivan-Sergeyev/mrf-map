// note: logging can be implemented via [log](https://docs.rs/log/latest/log/) and [env_logger](https://docs.rs/env_logger/latest/env_logger/)
// [example 1](https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/log.html#log-a-debug-message-to-the-console)
// [example 2](https://samkeen.dev/logging-in-rust-a-beginners-guide)

mod data_structures {
    pub mod hypergraph;
    pub mod jagged_arrays;
}

mod factor_types {
    pub mod factor_trait;
    pub mod factor_type;
    pub mod general_factor;
    pub mod nullary_factor;
    pub mod unary_factor;
}

mod message_passing {
    pub mod mp_factor_type;
    pub mod mp_general_factor;
    pub mod mp_trait;
    pub mod mp_unary_factor;
}

mod cfn {
    pub mod cost_function_network;
    pub mod relaxation;
    pub mod uai;
}

mod algo {
    pub mod solver;
    pub mod srmp;
}

mod csp {
    pub mod ac3;
    pub mod binary_csp;
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
