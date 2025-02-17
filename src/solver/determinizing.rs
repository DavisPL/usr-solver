/*
    Satisfiability checker using Antimirov derivatives - determinizing version

    (This is just a lazy copy of the code in brzozowski.rs)
*/

use super::Solver;

use crate::antimirov::determinized::{derivative_determinized, nullable_determinized};
use crate::types::expr::{CharExpression, CharVar};
use crate::types::regex::GenRegex;

use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct DeterminizingSolver {}

struct DerivativeDepth {
    gre: Rc<GenRegex>,
    depth: i32,
}

impl DeterminizingSolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl Solver for DeterminizingSolver {
    fn satisfiable(&mut self, expr: &Rc<GenRegex>) -> bool {
        let mut sat_stack = VecDeque::new();
        sat_stack.push_back(DerivativeDepth {
            gre: expr.clone(),
            depth: 0,
        });
        let mut visited: HashSet<Rc<GenRegex>> = HashSet::new();
        while let Some(layer) = sat_stack.pop_front() {
            println!("{}", layer.gre);
            if !nullable_determinized(&layer.gre).0.is_empty() {
                return true;
            } else {
                let deriv = derivative_determinized(&layer.gre, &self.get_fresh_var(layer.depth));
                for ele in deriv {
                    // Check range
                    let range = ele.get_ranges();
                    for (var, range) in range {
                        // TODO: Placeholder
                        eprintln!("TODO: handle range constraint {} on {}", range, var);
                        // For now, ignore and continue
                    }
                    if !visited.contains(ele.get_expr()) {
                        sat_stack.push_back(DerivativeDepth {
                            gre: ele.get_expr().clone(),
                            depth: layer.depth + 1,
                        });
                    }
                }
                visited.insert(layer.gre);
            }
        }
        false
    }
}

impl DeterminizingSolver {
    fn get_fresh_var(&mut self, id: i32) -> Rc<CharExpression> {
        let var_name = format!("f.{}", id);
        Rc::new(CharExpression::CharVar(CharVar { name: var_name }))
    }
}
