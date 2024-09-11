# Refactoring and Documentation

│   main.rs
│
├───cfn
│       cost_function_network.rs [v] (looks correct based on close inspection during revision, low testing priority)
│       relaxation.rs
│       solution.rs
│       solver.rs
│       srmp.rs
│       uai.rs [v] (looks correct based on close inspection during revision, low testing priority)
│
├───csp
│       ac3.rs
│       binary_csp.rs
│
├───data_structures
│       hypergraph.rs (very straightforward, looks correct, low testing priority)
│       jagged_arrays.rs (only used in binary csp at the moment)
│
├───factor_types
│       factor_trait.rs
│       factor_type.rs
│       general_factor.rs
│       unary_factor.rs
│
└───message
        messages.rs [v] (started writing tests)
        message_general.rs
        message_trait.rs
        message_type.rs

# Testing

- Unit tests: start with messages.rs.
    This is a good "middle point" for binary search for bugs.
    If some tests fail, then look for bugs in the underlying data structures.
    If all tests are OK, move on to testing SRMP.
- SRMP: unit tests don't seem appropriate here (unless very basic functionality).
    Instead, run integration tests on toy instances (like frustrated cycle, m.b. others from ac3, m.b. one used for tests in messages.rs).
    Follow this plan:
    - Generate a bunch of such instances
    - Save them as UAI test files
    - Answers should be easy to find or verify, save those as files as well
    - Run Rust and C++ programs on both, check for discrepancies

## Notes on C++ Code

- Reading from LG format is incorrect. The values should be exponentiated, not taken log of.
- When reading from UAI/LG, the sign of all cost functions is flipped. The file format does not specify if the original optimization problem is max or min.
