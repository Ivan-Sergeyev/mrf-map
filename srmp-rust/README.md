# Rust Implementation of Cost Function Networks and Constraint Satisfaction Problems

- inspiration: SRMP implementation by Vladimir Kolmogorov

## Implemented features

[v] CFNs
    - support for unary, binary, and higher-order terms
    - implementation stores all terms and an undirected hypergraph
    - can save and load in UAI format
[v] Relaxations
    - "Minimal edges" (aka factor graph)
[v] Binary CSPs
    - constraints are stored in a custom compressed bool tables (replaces `Vec<Vec<bool>>`, stores data linearly, stores bools in bits instead of bytes)
    - binary constraints are stored in a custom jagged table (replaces `Vec<Vec<...>>`, stores data linearly, represents non-existent constraints as `Option::None`)
    - arc consistency propagation via AC3
[ ] Algorithms
    - options struct, interface for algorithms via a trait

## Todo list

[ ] implement remaining features from SRMP:
    - save/load in UAI.LG format
    - algorithms (already have outline for SRMP!)
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
[ ] add logging
[ ] add tests
[ ] add docs

## Optional features

[ ] Flow algorithm for solving CFNs with only binary labels and pairwise terms
