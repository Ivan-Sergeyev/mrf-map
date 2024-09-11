use std::error::Error;
use std::collections::HashMap;

use good_lp::{constraint, default_solver, variable, variables, Expression, Solution, SolverModel};

fn edge_prev(var: usize, num_vars: usize) -> Vec<usize> {
    return vec![(num_vars + var - 1) % num_vars, var];
}

fn edge_next(var: usize, num_vars: usize) -> Vec<usize> {
    return vec![var, (var + 1) % num_vars];
}

fn main() -> Result<(), Box<dyn Error>> {
    // Set parameters of instance
    let num_vars = 3;
    let dom_size = 2;

    // Create frustrated cycle
    let vars: Vec<usize> = Vec::from_iter(0..num_vars);
    let doms: Vec<Vec<usize>> = vec![Vec::from_iter(0..dom_size); num_vars];
    let edges: Vec<Vec<usize>> = vars.clone().into_iter().map(|var| edge_next(var, num_vars)).collect();

    // let cost_0 = 0;
    let cost_1 = vec![vec![0; dom_size]; num_vars];
    let mut cost_2 = HashMap::new();
    for edge in &edges {
        cost_2.insert(edge, HashMap::new());
        for label_0 in &doms[edge[0]] {
            for label_1 in &doms[edge[1]] {
                cost_2.get_mut(edge).unwrap()
                    .insert((label_0, label_1), ((label_0 != label_1) ^ (edge[1] == vars[0])) as i32);
            }
        }
    }

    // Create empty LP
    variables! {osac_lp:}

    // Add variables
    let mut lp_vars_u = HashMap::new();
    for var in &vars {
        // println!("u[{}]", var);
        lp_vars_u.insert(var, osac_lp.add(variable().name(format!("u[{}]", var))));
    }
    // // One-line implementation (with vector instead of hashmap):
    // let lp_vars_u_2 : Vec<good_lp::Variable> = vars.into_iter().map(|var| osac_lp.add(variable().name(format!("u[{}]", var)))).collect();

    let mut lp_vars_p = HashMap::new();
    for edge in &edges {
        lp_vars_p.insert(edge, HashMap::new());
        for var in edge {
            lp_vars_p.get_mut(edge).unwrap().insert(var, HashMap::new());
            for label in &doms[*var] {
                // println!("p[{:?}][{}][{}]", edge, var, label);
                lp_vars_p.get_mut(edge).unwrap()
                    .get_mut(var).unwrap()
                    .insert(label, osac_lp.add(
                        variable().name(format!("p[{:?}][{}][{}]", edge, var, label))
                    ));
            }
        }
    }

    // Set objective
    let objective : Expression = lp_vars_u.iter().map(|(_key, value)| value).sum();
    let mut model = osac_lp.maximise(objective).using(default_solver);

    // Add constraints
    for var in &vars {
        for label in &doms[*var] {
            let e1 = edge_prev(*var, num_vars);
            let e2 = edge_next(*var, num_vars);
            model = model.with(constraint!(
                cost_1[*var][*label] - lp_vars_u[var] + lp_vars_p[&e1][var][label] + lp_vars_p[&e2][var][label] >= 0
            ));
        }
    }

    for edge in &edges {
        for label_0 in &doms[edge[0]] {
            for label_1 in &doms[edge[1]] {
                let labels = (label_0, label_1);
                model = model.with(constraint!(
                    cost_2[edge][&labels] - lp_vars_p[edge][&edge[0]][label_0] - lp_vars_p[edge][&edge[1]][label_1] >= 0
                ));
            }
        }
    }

    // Solve LP
    let solution = model.solve()?;

    // Print optimal solution
    for var in &vars {
        println!("u[{}] = {}", var, solution.value(lp_vars_u[var]));
    }
    for edge in &edges {
        for var in edge {
            for label in &doms[*var] {
                println!("p[{:?}][{}][{}] = {}", edge, var, label, solution.value(lp_vars_p[edge][var][label]));
            }
        }
    }

    Ok(())
}
