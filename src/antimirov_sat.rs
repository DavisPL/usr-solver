use crate::antimirov::{derivative, nullable};
use crate::classes::{CharExpression, CharVar, GenRegex};
use std::collections::HashSet;
use std::rc::Rc;

pub struct SatChecker {}

// Stores a regex and at what depth of derivative it was found.
struct DerivativeDepth {
    gre: Rc<GenRegex>,
    depth: i32,
}

impl SatChecker {
    pub fn new() -> Self {
        SatChecker {}
    }
    pub fn satisfiable(&mut self, expr: &Rc<GenRegex>) -> bool {
        let mut sat_stack = vec![DerivativeDepth {
            gre: expr.clone(),
            depth: 0,
        }];
        let mut visited: HashSet<Rc<GenRegex>> = HashSet::new();
        while let Some(layer) = sat_stack.pop() {
            if !nullable(&layer.gre).is_empty() {
                return true;
            } else {
                let deriv = derivative(&layer.gre, &self.get_fresh_var(layer.depth));
                for ele in deriv {
                    if !visited.contains(ele.get_expr()) {
                        sat_stack.push(DerivativeDepth {
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
    fn get_fresh_var(&mut self, id: i32) -> Rc<CharExpression> {
        let var_name = format!("f.{}", id);
        Rc::new(CharExpression::CharVar(CharVar { name: var_name }))
    }
}
