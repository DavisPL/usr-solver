//!
//! Matcher for Antimirov derivatives.
//! This matcher is currently unused.
//!

use super::deriv::{derivative, nullable};

use crate::types::expr::CharExpression;
use crate::types::regex::GenRegex;

use std::rc::Rc;

pub fn match_antimirov(expr: &Rc<GenRegex>, proposed: &str) -> bool {
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
        if match_antimirov(elem.get_expr(), tail) {
            return true;
        }
    }
    false
}
