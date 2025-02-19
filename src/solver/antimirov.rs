/*
    Satisfiability checker using Antimirov derivatives

    This solver internally uses Brzozowski in the complement case,
    so it is a hybrid approach.
*/

use super::Solver;

use crate::antimirov::deriv::{derivative, nullable};
use crate::types::expr::{CharExpression, CharVar};
use crate::types::regex::GenRegex;

use std::collections::{HashSet, BinaryHeap};
use std::cmp::{Ord, Ordering, PartialOrd};
use std::rc::Rc;
use std::cmp::Reverse;


#[derive(Debug, Default)]
pub struct AntimirovSolver {}

// Stores a regex and at what depth of derivative it was found.
struct DerivativeDepth {
    gre: Rc<GenRegex>,
    depth: i32,
}
impl Ord for DerivativeDepth {
    fn cmp(&self, other: &Self) -> Ordering {
        self.gre.cmp(&other.gre)
    }
}

impl PartialOrd for DerivativeDepth {

    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl PartialEq for DerivativeDepth {
    fn eq(&self, other: &Self) -> bool {
        self.gre == other.gre
    }
}

impl Eq for DerivativeDepth {}

impl AntimirovSolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl Solver for AntimirovSolver {
    fn satisfiable(&mut self, expr: &Rc<GenRegex>) -> bool {
        let mut sat_stack = BinaryHeap::new();
        sat_stack.push(Reverse(DerivativeDepth {
            gre: expr.clone(),
            depth: 0,
        }));
        let mut visited: HashSet<Rc<GenRegex>> = HashSet::new();
        while let Some(Reverse(layer)) = sat_stack.pop() {
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
                    sat_stack.push(Reverse(DerivativeDepth {
                        gre: ele.get_expr().clone(),
                        depth: layer.depth + 1,
                    }));
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
