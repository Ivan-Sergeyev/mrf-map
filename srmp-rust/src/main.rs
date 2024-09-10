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
    pub mod solution;
    pub mod relaxation;
    pub mod solver;
    pub mod srmp;
    pub mod uai;
}

mod csp {
    pub mod ac3;
    pub mod binary_csp;
}

use cfn::{cost_function_network::*, relaxation::{ConstructRelaxation, Relaxation}, solver::{Solver, SolverOptions}, srmp::SRMP, uai::UAI};
use log::debug;
use std::fs::OpenOptions;

fn main() {


    // // todo: move everything below to UAI (create a test)
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    debug!("In main");
    let input_file = OpenOptions::new()
        .read(true)
        .open("problem_instances/grid4x4.UAI.LG")
        .unwrap();
    let cfn = GeneralCFN::read_from_uai(input_file, false);
    let relaxation = Relaxation::new(&cfn);
    let srmp = SRMP::init(&relaxation);
    let options = SolverOptions::default();
    srmp.run(&options);

    // let output_file = OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .open("problem_instances/output.uai")
    //     .unwrap();
    // cfn.write_to_uai(output_file, false).unwrap();
}
