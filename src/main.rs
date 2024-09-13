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
    pub mod unary_factor;
}

mod message {
    pub mod message_general;
    pub mod message_trait;
    pub mod messages;
    // pub mod message_type;
    // pub mod message_unary;
}

mod cfn {
    pub mod cost_function_network;
    pub mod relaxation;
    pub mod solution;
    pub mod solver;
    pub mod srmp;
    pub mod uai;
}

mod csp {
    pub mod ac3;
    pub mod binary_csp;
}

use cfn::{
    cost_function_network::*,
    relaxation::{ConstructRelaxation, Relaxation},
    solver::{Solver, SolverOptions},
    srmp::SRMP,
    uai::UAI,
};
use log::debug;
use std::fs::OpenOptions;

fn main() {
    // Enable debug-level logging
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    debug!("In main");

    let test_instance_files = std::fs::read_dir("test_instances/").unwrap();

    for path in test_instance_files {
        let input_filename = path.unwrap().path();

        debug!("Importing test instance from {}", input_filename.display());
        let input_file = OpenOptions::new().read(true).open(input_filename).unwrap();
        let cfn = CostFunctionNetwork::read_uai(input_file, false);

        debug!("Flipping signs");
        // cfn.map_factors_inplace(|value| *value *= -1.0); // flip sign (todo: is this needed?)

        debug!("Constructing relaxation");
        let relaxation = Relaxation::new(&cfn);

        debug!("Initializing SRMP");
        let srmp = SRMP::init(&relaxation);

        debug!("Running SRMP");
        let options = SolverOptions::default();
        srmp.run(&options);

        debug!("Finished\n\n\n");

        // let output_file = OpenOptions::new()
        //     .create(true)
        //     .write(true)
        //     .open("problem_instances/output.uai")
        //     .unwrap();
        // cfn.write_to_uai(output_file, false).unwrap();
    }
}
