//!
//! Determinized version of the Antimirov substitution-derivative.
//!
//! This is an experiment - just to see if it works.
//!

// TODO: fix and remove
#![allow(unused_variables)]
#![allow(unused_imports)]

use super::deriv;
use super::subs::{AntimirovElement, SimpleSub, SubExpr};
use super::util::{char_minus_one, char_plus_one, CHAR_MAX, CHAR_MIN};

use crate::types::expr::{CharExpression, CharVar};
use crate::types::regex::GenRegex;

use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

/*
    Determinized derivative

    Returns a set of AntimirovElements (R, f, pi)
    where R is a derivative, f is a substitution, and pi is a range constraint
    such that all (f, pi) pairs form a partition of the entire space of valuations.

    The idea is that we can then complement these easily by just negating each individual R.
*/
pub fn derivative_determinized(
    gre: &Rc<GenRegex>,
    deriv_char: &Rc<CharExpression>,
) -> HashSet<AntimirovElement> {
    // println!("taking d({}, {})", gre, deriv_char);

    match gre.as_ref() {
        GenRegex::EmptySet => AntimirovElement::new_empty().into_set(),
        GenRegex::Epsilon => AntimirovElement::new_empty().into_set(),
        GenRegex::Sigma => AntimirovElement::new_epsilon().into_set(),
        GenRegex::Range(char1, char2) => match deriv_char.as_ref() {
            CharExpression::Literal(literal) => {
                if literal < char1 || literal > char2 {
                    AntimirovElement::new_empty().into_set()
                } else {
                    AntimirovElement::new_epsilon().into_set()
                }
            }
            CharExpression::CharVar(deriv_var) => determinize_range(deriv_var, *char1, *char2),
        },
        GenRegex::CharExpression(c_expr) => match (deriv_char.as_ref(), c_expr) {
            (CharExpression::Literal(deriv_lit), CharExpression::Literal(literal_value)) => {
                if deriv_lit == literal_value {
                    AntimirovElement::new_epsilon().into_set()
                } else {
                    AntimirovElement::new_empty().into_set()
                }
            }
            (CharExpression::CharVar(d_var), CharExpression::Literal(lit_val)) => {
                determinize_range(d_var, *lit_val, *lit_val)
            }
            (CharExpression::Literal(lit_val), CharExpression::CharVar(c_var)) => {
                determinize_range(c_var, *lit_val, *lit_val)
            }
            (CharExpression::CharVar(d_var), CharExpression::CharVar(c_var)) => {
                // TODO: Hard case, requires encoding x != y
                unimplemented!();
            }
        },
        GenRegex::StringVar(string_var) => {
            // TODO: Hard case, requires handling w |-> xw and negation of this
            unimplemented!();
        }
        GenRegex::Union(side1, side2) => {
            let side1_deriv = derivative_determinized(side1, deriv_char);
            let side2_deriv = derivative_determinized(side2, deriv_char);
            merge_helper(&side1_deriv, &side2_deriv, &|left, right| {
                Some(GenRegex::union(left, right))
            })
        }
        GenRegex::Intersect(side1, side2) => {
            let side1_deriv = derivative_determinized(side1, deriv_char);
            let side2_deriv = derivative_determinized(side2, deriv_char);
            merge_helper(&side1_deriv, &side2_deriv, &|left, right| {
                Some(GenRegex::intersect(left, right))
            })
        }
        GenRegex::Concatenation(left, right) => {
            unimplemented!();
            // let left_deriv = derivative(left, deriv_char);
            // let mut ret_set = HashSet::new();
            // for sub in &left_deriv {
            //     if let Some(ret) = apply_deriv_concat(sub, right) {
            //         ret_set.insert(ret);
            //     }
            // }

            // let p_nullable = nullable(left);
            // if !p_nullable.is_empty() {
            //     let right_deriv = derivative(right, deriv_char);
            //     for n_sub in &p_nullable {
            //         for q_sub in &right_deriv {
            //             if let Some(ret) = merge_derivs_concat(n_sub, q_sub) {
            //                 ret_set.insert(ret);
            //             }
            //         }
            //     }
            // }

            // ret_set
        }
        GenRegex::Kleene(expr) => {
            let p_derivs = derivative_determinized(expr, deriv_char);
            let right_copy = AntimirovElement::new_emptysub(gre.clone()).into_set();
            merge_helper(&right_copy, &p_derivs, &|left, right| {
                Some(GenRegex::concat(left, right))
            })
        }
        GenRegex::Complement(expr) => {
            // This is where we get the benefit of determinization!
            let p_derivs = derivative_determinized(expr, deriv_char);
            p_derivs
                .into_iter()
                .map(|elem| elem.map_expr(|gre| GenRegex::complement(&gre)))
                .collect()
        }
        GenRegex::IfThenElse(_, _, _) => {
            // Unimplemented for now
            unimplemented!();
        }
        GenRegex::StringSlice(_, _) => {
            // Unimplemented for now
            unimplemented!();
        }
        GenRegex::StringIndex(_) => {
            // Unimplemented for now
            unimplemented!();
        }
    }
}

// Return a determinized set of derivatives for a range
// TODO: handle the case char1 = char2 by creating a substitution instead of a range
fn determinize_range(deriv_var: &CharVar, char1: char, char2: char) -> HashSet<AntimirovElement> {
    let mut result = HashSet::new();
    if let Some(char0) = char_minus_one(char1) {
        result.insert(AntimirovElement::new_empty_range(
            deriv_var.clone(),
            CHAR_MIN,
            char0,
        ));
    }
    result.insert(AntimirovElement::new_epsilon_range(
        deriv_var.clone(),
        char1,
        char2,
    ));
    if let Some(char2) = char_plus_one(char1) {
        result.insert(AntimirovElement::new_empty_range(
            deriv_var.clone(),
            char2,
            CHAR_MAX,
        ));
    }
    result
}

// Merge two determinized derivatives using a custom GenRegex combination operation
// This is done exhaustively (effectively a product construction)
fn merge_helper<F>(
    left_set: &HashSet<AntimirovElement>,
    right_set: &HashSet<AntimirovElement>,
    merge_op: &F,
) -> HashSet<AntimirovElement>
where
    F: Fn(&Rc<GenRegex>, &Rc<GenRegex>) -> Option<Rc<GenRegex>>,
{
    let mut result = HashSet::new();
    for left in left_set {
        for right in right_set {
            if let Some(merged) = AntimirovElement::merge_using(left, right, merge_op) {
                result.insert(merged);
            }
        }
    }
    result
}
