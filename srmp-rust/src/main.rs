mod tables {
    pub mod compressed_bit_table;
    pub mod jagged_table;
    pub mod justified_table;
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

use std::fs::OpenOptions;

use cfn::cost_function_network::*;
use csp::ac3::AC3;
use csp::binary_csp::BinaryCSP;

fn example_1() {
    // Example: frustrated cycle
    let mut fc_csp = BinaryCSP::from_unary_constraints(vec![vec![1, 1]; 3]);
    fc_csp.add_binary_constraint(0, 1, vec![vec![0, 1], vec![0, 1]]);
    fc_csp.add_binary_constraint(1, 2, vec![vec![0, 1], vec![0, 1]]);
    fc_csp.add_binary_constraint(2, 0, vec![vec![0, 1], vec![0, 1]]);

    let ac3_result = AC3::new().run_algorithm(&fc_csp);
    if let Some(var) = ac3_result {
        println!("AC-3 results in domain wipe out at variable {var}");
    } else {
        println!("AC-3 results in arc-consistent CSP");
    }
}

fn example_2() {
    // Example: inconsistent CSP on one variable
    let csp = BinaryCSP::from_unary_constraints(vec![vec![0; 5]]);

    let ac3_result = AC3::new().run_algorithm(&csp);
    if let Some(var) = ac3_result {
        println!("AC-3 results in domain wipe out at variable {var}");
    } else {
        println!("AC-3 results in arc-consistent CSP");
    }
}

fn example_3() {
    // Example: inconsistent CSP on two variables
    let mut csp = BinaryCSP::from_unary_constraints(vec![vec![1, 0], vec![0, 1]]);
    csp.add_binary_constraint(0, 1, vec![vec![1, 0], vec![0, 1]]);

    let ac3_result = AC3::new().run_algorithm(&csp);
    if let Some(var) = ac3_result {
        println!("AC-3 results in domain wipe out at variable {var}");
    } else {
        println!("AC-3 results in arc-consistent CSP");
    }
}

fn example_4() {
    // Example: consistent CSP on three variables
    let mut csp = BinaryCSP::from_unary_constraints(vec![vec![1, 1], vec![1, 0], vec![0, 1]]);
    csp.add_binary_constraint(0, 1, vec![vec![1, 0], vec![0, 1]]);
    csp.add_binary_constraint(1, 2, vec![vec![1, 1], vec![0, 1]]);
    csp.add_binary_constraint(0, 2, vec![vec![1, 1], vec![0, 1]]);

    let ac3_result = AC3::new().run_algorithm(&csp);
    if let Some(var) = ac3_result {
        println!("AC-3 results in domain wipe out at variable {var}");
    } else {
        println!("AC-3 results in arc-consistent CSP");
    }
}

// todo: test preventing pushes to queue because of uniqueness

fn main() {
    // // todo: convert into tests
    // example_1();
    // example_2();
    // example_3();
    // example_4();

    let input_file = OpenOptions::new()
        .read(true)
        .open("problem_instances/grid4x4.UAI.LG")
        .unwrap();
    let cfn = CFNPetGraph::read_from_uai(input_file);

    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open("problem_instances/output.uai")
        .unwrap();
    cfn.write_to_uai(output_file).unwrap();
}
