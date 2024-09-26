mod data_structures {
    pub mod jagged_arrays;
}

mod factor_types {
    pub mod factor_trait;
    pub mod factor_type;
    pub mod function_table;
    pub mod potts;
    pub mod uniform_constant;
}

mod message {
    pub mod message_nd;
    pub mod message_trait;
    pub mod messages;
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

use std::time::Instant;

use cfn::{
    cost_function_network::*,
    relaxation::{ConstructRelaxation, Relaxation},
    solver::{Solver, SolverOptions},
    srmp::SRMP,
    uai::UAI,
};
use log::info;

fn main() {
    std::env::set_var("RUST_LOG", "info"); // change "info" to "debug" for debug-level logging, etc.
    env_logger::init();

    let test_instance_files = std::fs::read_dir("test_instances/").unwrap();

    for path in test_instance_files {
        let input_file = path.unwrap().path();
        let filename = input_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        info!("Processing instance {}.", filename);

        let time_start = Instant::now();
        let cfn = CostFunctionNetwork::read_uai(input_file, false);
        info!(
            "UAI import complete. Elapsed time {:?}.",
            time_start.elapsed()
        );

        let time_start = Instant::now();
        let relaxation = Relaxation::new(&cfn);
        info!(
            "Relaxation constructed. Elapsed time {:?}.",
            time_start.elapsed()
        );

        let srmp = SRMP::init(&relaxation);
        let options = SolverOptions::default();
        srmp.run(&options);

        info!("Finished processing instance {}.\n\n\n", filename);
    }
}
