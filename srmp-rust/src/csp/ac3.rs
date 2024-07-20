#![allow(dead_code)]

use crate::csp::binary_csp::BinaryCSP;
use std::collections::VecDeque;

/// Supporting data structures and implementation of the AC-3 algorithm.
/// todo: upgrade to ensure uniqueness of elements in queue
/// todo: upgrade to handle CSPs
/// todo: implement more efficient arc consistency algorithm
pub struct AC3 {
    active_domains: Vec<Vec<usize>>,
    queue: VecDeque<(usize, usize)>,
}

impl AC3 {
    pub fn new() -> Self {
        AC3 {
            active_domains: Vec::new(),
            queue: VecDeque::new(),
        }
    }

    fn init(&mut self, csp: &BinaryCSP) -> Option<usize> {
        self.active_domains = Vec::with_capacity(csp.num_variables());
        for var in csp.var_range() {
            let active_domain: Vec<usize> = csp
                .domain_range(var)
                .filter(|&label| *csp.is_unary_satisfied(var, label))
                .collect();
            if active_domain.is_empty() {
                return Some(var); // preemptive domain wipe out at var
            }
            self.active_domains.push(active_domain);
        }

        for var_x in csp.var_range() {
            for var_y in csp.var_range_from(var_x) {
                if csp.exists_binary_constraint(var_x, var_y) {
                    self.queue.push_back((var_x, var_y));
                }
            }
        }

        None // initialization successful
    }

    fn revise(&mut self, csp: &BinaryCSP, var_x: usize, var_y: usize) -> bool {
        let mut revised = false;
        let mut upd_domain_x = Vec::new();

        for &label_x in &self.active_domains[var_x] {
            let mut found_satisfying_assignment = false;

            for &label_y in &self.active_domains[var_y] {
                if csp.is_binary_satisfied(var_x, var_y, label_x, label_y) {
                    found_satisfying_assignment = true;
                    upd_domain_x.push(label_x);
                    break;
                }
            }

            if !found_satisfying_assignment {
                revised = true;
                // killer[var_x, label_x] = var_y (generally S)  // additions in Instrumented-AC from Cooper et al 2010
                // Q.push(var_x, label_x)                        // both exploited later in phase 2 (computing \lambda)
            }
        }

        self.active_domains[var_x] = upd_domain_x;
        revised
    }

    pub fn run_algorithm(&mut self, csp: &BinaryCSP) -> Option<usize> {
        if let Some(var) = self.init(csp) {
            return Some(var); // preemptive domain wipe out at var
        }

        loop {
            let Some((var_x, var_y)) = self.queue.pop_front() else {
                break;
            };

            if !self.revise(csp, var_x, var_y) {
                continue;
            }

            if self.active_domains[var_x].is_empty() {
                return Some(var_x); // domain wipe out at var_x
            }

            for var_z in csp.var_range() {
                if var_z != var_x && var_z != var_y && csp.exists_binary_constraint(var_x, var_z) {
                    self.queue.push_back((var_x, var_z));
                }
            }
        }

        None // CSP is arc consistent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frustrated_cycle() {
        let mut fc_csp = BinaryCSP::from_unary_constraints(vec![vec![true, true]; 3]);
        fc_csp.add_binary_constraint(0, 1, vec![vec![false, true], vec![false, true]]);
        fc_csp.add_binary_constraint(1, 2, vec![vec![false, true], vec![false, true]]);
        fc_csp.add_binary_constraint(2, 0, vec![vec![false, true], vec![false, true]]);
        let ac3_result = AC3::new().run_algorithm(&fc_csp);
        assert_eq!(ac3_result, None);
    }

    #[test]
    fn inconsistent_one_variable() {
        let csp = BinaryCSP::from_unary_constraints(vec![vec![false; 5]]);
        let ac3_result = AC3::new().run_algorithm(&csp);
        assert_eq!(ac3_result, Some(0));
    }

    #[test]
    fn inconsistent_two_variables() {
        let mut csp = BinaryCSP::from_unary_constraints(vec![vec![true, false], vec![false, true]]);
        csp.add_binary_constraint(0, 1, vec![vec![true, false], vec![false, true]]);
        let ac3_result = AC3::new().run_algorithm(&csp);
        assert_eq!(ac3_result, Some(0));
    }

    #[test]
    fn consistent_two_variables() {
        let mut csp = BinaryCSP::from_unary_constraints(vec![
            vec![true, true],
            vec![true, false],
            vec![false, true],
        ]);
        csp.add_binary_constraint(0, 1, vec![vec![true, false], vec![false, true]]);
        csp.add_binary_constraint(1, 2, vec![vec![true, true], vec![false, true]]);
        csp.add_binary_constraint(0, 2, vec![vec![true, true], vec![false, true]]);
        let ac3_result = AC3::new().run_algorithm(&csp);
        assert_eq!(ac3_result, None);
    }

    // todo: test with different unary domain sizes
    // todo: test where ac3 needs to revisit edges several times
}
