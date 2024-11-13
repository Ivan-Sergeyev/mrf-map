mod data_structures {
    pub mod jagged_arrays;
}

mod factors {
    pub mod factor_trait;
    pub mod factor_type;
    pub mod function_table;
    pub mod potts;
    pub mod uniform_constant;
}

mod messages {
    pub mod message_nd;
    pub mod message_trait;
}

mod alg {
    pub mod solver;
    pub mod srmp;
    pub mod srmp_new;
}

mod cfn {
    pub mod cost_function_network;
    pub mod factor_sequence;
    pub mod relaxation;
    pub mod solution;
    pub mod uai;
}

mod csp {
    pub mod ac3;
    pub mod binary_csp;
}

use std::time::Instant;

use alg::{
    solver::{Solver, SolverOptions},
    srmp::SRMP,
};
use cfn::{
    cost_function_network::*,
    relaxation::{ConstructRelaxation, Relaxation},
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

        let srmp = SRMP::init(&cfn, &relaxation);
        let options = SolverOptions::default();
        srmp.run(&options);

        info!("Finished processing instance {}.\n\n\n", filename);
    }
}
