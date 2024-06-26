# Rust Implementation of Cost Function Networks and Constraint Satisfaction Problems

- inspiration: SRMP implementation by Vladimir Kolmogorov

## Implemented features

[v] Binary CSPs
    - constraints are stored in a custom compressed bool tables (replaces `Vec<Vec<bool>>`, stores data linearly, stores bools in bits instead of bytes)
    - binary constraints are stored in a custom jagged table (replaces `Vec<Vec<...>>`, stores data linearly, represents non-existent constraints as `Option::None`)
    - arc consistency propagation via AC3
[v] CFNs
    - currently only unary and pairwise terms are supported
    - implementation stores `Vec`'s of unary and pairwise terms and an undirected graph (`petgraph` module)
    - can generate
    - can save and load in UAI format
[ ] Algorithms
    - options struct, interface for algorithms via a trait

## Todo list

[ ] implement remaining features from SRMP:
    - save/load in UAI.LG format
    - algorithms (already have outline for SRMP!)
    - general factors (arbitrary arity)
    - optimized factor types (e.g., Potts)
    - etc.
[ ] variable splitting for CFNs and/or CPSs
[ ] extend CSP functionality
    - generate CSP based on CFN
    - alternative and more efficient arc consistency algorithms (e.g., AC2001, AC-6, Bessiere et al. 2005)
[ ] optimize implementations
[ ] add problem instances from data sets used in publications
    - authors of toulbar
    - Kappes et al. 2015
    - etc.
[ ] add logging, docs, and tests

## Optional features

[ ] Flow algorithm for solving CFNs with only binary labels and pairwise terms
