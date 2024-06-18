use std::{cell::RefCell, rc::Rc};


type VariableIndex = usize;  // used to index variables
type LabelIndex = usize;  // used to index labels
type AssignOne = (VariableIndex, LabelIndex);  // assignment of a label to a variable
type AssignTwo = [(VariableIndex, LabelIndex); 2];
type Assignment = Vec<AssignOne>;  // assignment of a set of labels to a set of variables

struct NullaryCost<T> {
    cost: T,  // value
    // has no parent assignment
}

struct UnaryCost<T> {
    cost: T,  // value
    parent: AssignOne,  // parent assignment
}

struct BinaryCost<T> {
    cost: T,  // value
    parent: AssignTwo,  // parent assignment
}

struct HigherOrderCost<T> {
    cost: T,  // value
    parent: Assignment,  // parent assignment
}

enum CostFunctionKind<T> {
    Nullary(NullaryCost<T>),
    Unary(UnaryCost<T>),
    Binary(BinaryCost<T>),
    HigherOrder(HigherOrderCost<T>),
}

struct Variable<T> {
    labels: Vec<Label<T>>,  // list of available labels
}

struct Label<T> {
    parent: VariableIndex,  // parent variable
    cost_unary: Rc<RefCell<UnaryCost<T>>>,  // unary cost associated with this label
    costs_binary: Vec<Rc<RefCell<BinaryCost<T>>>>,  // binary cost functions involving this label
    costs_higher_order: Vec<Rc<RefCell<HigherOrderCost<T>>>>,  // higher-order cost functions involving this label
}

pub struct CostFunctionNetwork<T> {
    cost_nullary: Rc<RefCell<NullaryCost<T>>>,  // constant term in total cost
    variables: Vec<Variable<T>>,  // list of variables
}

impl<T: Default> CostFunctionNetwork<T> {
    // Initializes an empty cost function network
    pub fn new() -> Self {
        CostFunctionNetwork {
            cost_nullary: Rc::new(RefCell::new(NullaryCost {cost: Default::default()})),
            variables: Vec::new(),
        }
    }

    // Initializes cost function network and sets given domain sizes (nullary and all unary costs are default)
    pub fn from_domain_sizes(&mut self, domain_sizes: &Vec<usize>) -> Self {
        let mut variables = Vec::new();
        for (var_idx, &domain_size) in domain_sizes.iter().enumerate() {
            let mut labels = Vec::new();
            for label_idx in 0..domain_size {
                labels.push(Label {
                    parent: var_idx,
                    cost_unary: Rc::new(RefCell::new(UnaryCost {cost: Default::default(), parent: (var_idx, label_idx)})),
                    costs_binary: Vec::new(),
                    costs_higher_order: Vec::new(),
                });
            }
            variables.push(Variable {labels: labels});
        }

        CostFunctionNetwork {
            cost_nullary: Rc::new(RefCell::new(NullaryCost {cost: Default::default()})),
            variables: variables,
        }
    }

    // Initializes cost function network and sets given nullary and unary costs
    pub fn from_costs(&mut self, nullary_cost_value: T, unary_costs: Vec<Vec<T>>) -> Self {
        let mut variables = Vec::new();
        for (var_idx, var_unary_costs) in unary_costs.into_iter().enumerate() {
            let mut labels = Vec::new();
            for (label_idx, unary_cost_value) in var_unary_costs.into_iter().enumerate() {
                labels.push(Label {
                    parent: var_idx,
                    cost_unary: Rc::new(RefCell::new(UnaryCost {cost: unary_cost_value, parent: (var_idx, label_idx)})),
                    costs_binary: Vec::new(),
                    costs_higher_order: Vec::new(),
                });
            }
            variables.push(Variable {labels: labels});
        }

        CostFunctionNetwork {
            cost_nullary: Rc::new(RefCell::new(NullaryCost {cost: nullary_cost_value})),
            variables: variables,
        }
    }

    // Sets nullary cost
    pub fn set_cost_nullary(mut self, nullary_cost_value: T) -> Self {
        self.cost_nullary.as_ref().borrow_mut().cost = nullary_cost_value;
        self
    }

    // Adds new variable with empty label list
    pub fn add_variable(mut self) -> Self {
        self.variables.push(Variable {labels: Vec::new()});
        self
    }

    // Adds new label to given variable (with default unary cost and empty binary and higher-order cost function lists)
    pub fn add_label(mut self, var_idx: VariableIndex) -> Self {
        let label_idx = self.variables[var_idx].labels.len();
        self.variables[var_idx].labels.push(Label {
            parent: var_idx,
            cost_unary: Rc::new(RefCell::new(UnaryCost {cost: Default::default(), parent: (var_idx, label_idx)})),
            costs_binary: Vec::new(),
            costs_higher_order: Vec::new(),
        });
        self
    }

    // Sets unary cost of given variable label
    pub fn set_cost_unary(mut self, var_idx: VariableIndex, label_idx: LabelIndex, unary_cost_value: T) -> Self {
        self.variables[var_idx].labels[label_idx].cost_unary.as_ref().borrow_mut().cost = unary_cost_value;
        self
    }

    // Adds binary cost function
    pub fn add_cost_binary(mut self, assignment: AssignTwo, cost_value: T) -> Self {
        let cost_function = Rc::new(RefCell::new(BinaryCost {cost: cost_value, parent: assignment}));
        for &(var_idx, label_idx) in &assignment {
            self.variables[var_idx].labels[label_idx].costs_binary.push(Rc::clone(&cost_function));
        }
        self
    }

    // Adds higher-order cost function
    pub fn add_cost_higher_order(mut self, assignment: Assignment, cost_value: T) -> Self {
        let cost_function = Rc::new(RefCell::new(HigherOrderCost {cost: cost_value, parent: assignment}));
        for &(var_idx, label_idx) in &cost_function.borrow().parent {
            self.variables[var_idx].labels[label_idx].costs_higher_order.push(Rc::clone(&cost_function));
        }
        self
    }
}

// todo: encoder and decoder for variables and labels


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        let cfn: CostFunctionNetwork<f64> = CostFunctionNetwork::new();
        assert_eq!(cfn.cost_nullary.borrow().cost, 0.0);
        assert_eq!(cfn.variables.len(), 0);
    }

    #[test]
    fn from_domain_sizes() {
        // todo: test
    }

    #[test]
    fn from_costs () {}

    #[test]
    fn set_cost_nullary () {}

    #[test]
    fn add_variable () {}

    #[test]
    fn add_label () {}

    #[test]
    fn set_cost_unary () {}

    #[test]
    fn add_cost_binary () {}

    #[test]
    fn add_cost_higher_order () {}

}
