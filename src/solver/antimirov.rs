/*
    Satisfiability checker using Antimirov derivatives

    This solver internally uses Brzozowski in the complement case,
    so it is a hybrid approach.
*/

use super::Solver;

use crate::antimirov::deriv::{derivative, nullable};
use crate::types::expr::{CharExpression, CharVar};
use crate::types::regex::GenRegex;

use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct AntimirovSolver {}

// Stores a regex and at what depth of derivative it was found.
struct DerivativeDepth {
    gre: Rc<GenRegex>,
    depth: i32,
}

impl AntimirovSolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl Solver for AntimirovSolver {
    fn satisfiable(&mut self, expr: &Rc<GenRegex>) -> bool {
        let mut sat_stack = VecDeque::new();
        sat_stack.push_back(DerivativeDepth {
            gre: expr.clone(),
            depth: 0,
        });
        let mut visited: HashSet<Rc<GenRegex>> = HashSet::new();
        while let Some(layer) = sat_stack.pop_front() {
            if !nullable(&layer.gre).is_empty() {
                return true;
            } else if visited.contains(&layer.gre) {
                continue;
            } else {
                visited.insert(layer.gre.clone());
                println!("{}", layer.gre);
                println!("Visited count: {}", visited.len());
                println!("Stack size: {}", sat_stack.len());

                let deriv = derivative(&layer.gre, &self.get_fresh_var(layer.depth));
                for ele in deriv {
                    // Check range
                    let range = ele.get_ranges();
                    for (var, range) in range {
                        // TODO: Placeholder
                        eprintln!("TODO: handle range constraint {} on {}", range, var);
                        // For now, ignore and continue
                    }
                    sat_stack.push_back(DerivativeDepth {
                        gre: ele.get_expr().clone(),
                        depth: layer.depth + 1,
                    });
                }
            }
        }
        false
    }
}

impl AntimirovSolver {
    fn get_fresh_var(&mut self, id: i32) -> Rc<CharExpression> {
        let var_name = format!("f.{}", id);
        Rc::new(CharExpression::CharVar(CharVar { name: var_name }))
    }
}
