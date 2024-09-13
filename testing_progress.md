# Refactoring and Documentation

│   main.rs
│
├───cfn
│       cost_function_network.rs [v] (looks correct based on close inspection during revision, low testing priority)
│       relaxation.rs
│       solution.rs (very straightforward, looks correct, low testing priority)
│       solver.rs (very straightforward, looks correct, low testing priority)
│       srmp.rs
│       uai.rs [v] (looks correct based on close inspection during revision, low testing priority)
│
├───csp (no, focus on SRMP for now)
│       ac3.rs (no, focus on SRMP for now)
│       binary_csp.rs (no, focus on SRMP for now)
│
├───data_structures
│       hypergraph.rs (very straightforward, looks correct, low testing priority)
│       jagged_arrays.rs (no, only used in binary csp at the moment)
│
├───factor_types
│       factor_trait.rs (very straightforward, looks correct, low testing priority) -- careful with Index and IndexMut, this is an implicit assumption about how factors are currently implemented
│       factor_type.rs
│       general_factor.rs (pretty straightforward, looks correct, low testing priority) -- careful with Index and IndexMut, this is an implicit assumption about how factors are currently implemented
│       unary_factor.rs (pretty straightforward, looks correct, low testing priority) -- careful with Index and IndexMut, this is an implicit assumption about how factors are currently implemented
│
└───message
        messages.rs [v] (started writing tests)
        message_general.rs
        message_trait.rs
        message_type.rs (currently unused)

- README.rs

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
