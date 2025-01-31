//!
//! Development for Antimirov derivatives
//!

// TODO: fix and remove
#![allow(unused_variables)]

pub mod deriv;
pub mod sat_checker;
pub mod sub_from_predicate;
pub mod subs;
pub mod union_find;

/*
    Top-level functions: satisfiable and matching
*/

pub use deriv::{derivative, nullable};

use crate::types::expr::{CharExpression, CharVar};
use crate::types::regex::GenRegex;

use std::collections::HashSet;
use std::rc::Rc;

pub fn satisfiable(expr: &Rc<GenRegex>) -> bool {
    let mut ind = 0;
    satisfiable_helper(expr, &mut ind, HashSet::new())
}

fn satisfiable_helper(
    expr: &Rc<GenRegex>,
    index: &mut i32,
    mut visited: HashSet<GenRegex>,
) -> bool {
    println!("Checking sat: {} (index {})", expr, index);
    visited.insert(expr.as_ref().clone());
    if nullable(expr).is_empty() {
        let new_name = "f".to_owned() + &index.to_string();
        let c_var = Rc::new(CharExpression::CharVar(CharVar { name: new_name }));
        let deriv = derivative(expr, &c_var);
        if deriv.is_empty() {
            return false;
        }
        *index += 1;
        for elem in deriv {
            // TODO: handle range constraint here
            if !visited.contains(elem.get_expr())
                && satisfiable_helper(elem.get_expr(), index, visited.clone())
            {
                return true;
            }
        }
        return false;
    }
    true
}

pub fn matching(expr: &Rc<GenRegex>, proposed: &str) -> bool {
    if proposed.is_empty() {
        return !nullable(expr).is_empty();
    }
    let first_char = proposed.chars().next().unwrap();
    let tail = &proposed[1..];
    let literal = Rc::new(CharExpression::Literal(first_char));
    let deriv = derivative(expr, &literal);
    if deriv.is_empty() {
        return false;
    }
    for elem in deriv {
        if matching(elem.get_expr(), tail) {
            return true;
        }
    }
    false
}
