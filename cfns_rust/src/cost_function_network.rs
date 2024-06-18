// pub mod cost_function_network;

// feature todo: add inheritance to CostFunction structs (e.g. w.r.t. operations on values)?
// feature todo: binary case only -- via traits
// feature todo: swap graph structure (adjacency lists, PCSR (http://supertech.csail.mit.edu/papers/WheatmanXu18.pdf)) -- via traits!

struct NullaryCostFunction<T> {
    value: T,
}

struct UnaryCostFunction<'a, T> {
    value: T,
    parent_label: &'a Label<'a, T>,
}

struct BinaryCostFunction<'a, T> {
    value: T,
    parent_labels: [&'a Label<'a, T>; 2],
}

struct HigherOrderCostFunction<'a, T> {
    value: T,
    parent_labels: Vec<&'a Label<'a, T>>,
}

enum CostFunctionKind<'a, T> {
    Nullary(NullaryCostFunction<T>),
    Unary(UnaryCostFunction<'a, T>),
    Binary(BinaryCostFunction<'a, T>),
    HigherOrder(HigherOrderCostFunction<'a, T>),
}

use std::ops::Add;
impl<T> Add<T> for NullaryCostFunction<T> {
    type Output = NullaryCostFunction<T>;

    fn add(self, other: T) -> NullaryCostFunction<T> {

    }
}
add_impl!{NullaryCostFunction<T>}


struct Variable<'a, T> {
    name: String,  // variable name
    labels: Vec<Label<'a, T>>,  // available labels
}

struct Label<'a, T> {
    name: String,  // label name
    parent_variable: &'a Variable<'a, T>,  // which variable this label belongs to
    cost_unary: UnaryCostFunction<'a, T>,  // cost of this label
    costs: Vec<HigherOrderCostFunction<'a, T>>,  // costs associated to this label
}

pub struct CostFunctionNetwork<'a, T> {
    name: String,  // cost function network name
    cost_nullary: NullaryCostFunction<T>,  // constant term in total cost
    variables: Vec<Variable<'a, T>>,  // variables
}

impl<'a, T> CostFunctionNetwork<'a, T> {
    // pub fn new() -> Self {

    // }
}
