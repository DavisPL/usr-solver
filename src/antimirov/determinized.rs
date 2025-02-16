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

use crate::types::expr::CharExpression;
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
            CharExpression::CharVar(deriv_var) => {
                let mut result = HashSet::new();
                if let Some(char0) = char_minus_one(*char1) {
                    let mut result_low = AntimirovElement::new_empty();
                    result_low.add_range(deriv_var.clone(), CHAR_MIN, char0);
                    result.insert(result_low);
                }
                let mut result_mid = AntimirovElement::new_epsilon();
                result_mid.add_range(deriv_var.clone(), *char1, *char2);
                result.insert(result_mid);
                if let Some(char2) = char_plus_one(*char1) {
                    let mut result_high = AntimirovElement::new_empty();
                    result_high.add_range(deriv_var.clone(), char2, CHAR_MAX);
                    result.insert(result_high);
                }
                result
            }
        },
        GenRegex::CharExpression(c_expr) => match (deriv_char.as_ref(), c_expr) {
            (CharExpression::Literal(deriv_lit), CharExpression::Literal(literal_value)) => {
                unimplemented!();
                // if deriv_lit == literal_value {
                //     AntimirovElement::new_epsilon().into_set()
                // } else {
                //     HashSet::new()
                // }
            }
            (CharExpression::CharVar(d_var), CharExpression::Literal(lit_val)) => {
                unimplemented!();
                // let mut char_to = BTreeMap::new();
                // char_to.insert(d_var.clone(), c_expr.clone());
                // let subs = SimpleSub::new(BTreeMap::new(), char_to, BTreeMap::new());
                // AntimirovElement::new(GenRegex::epsilon(), subs).into_set()
            }
            (_, CharExpression::CharVar(c_var)) => {
                unimplemented!();
                // let mut char_to = BTreeMap::new();
                // char_to.insert(c_var.clone(), deriv_char.as_ref().clone());
                // let subs = SimpleSub::new(BTreeMap::new(), char_to, BTreeMap::new());
                // AntimirovElement::new(GenRegex::epsilon(), subs).into_set()
            }
        },
        GenRegex::StringVar(string_var) => {
            unimplemented!();
            // let head = vec![deriv_char.as_ref().clone()];

            // let subexpr = SubExpr::new(head, true);

            // let mut string_to = BTreeMap::new();
            // string_to.insert(string_var.clone(), subexpr);

            // let substitution = SimpleSub::new(string_to, BTreeMap::new(), BTreeMap::new());

            // AntimirovElement::new(gre.clone(), substitution).into_set()
        }
        GenRegex::Union(side1, side2) => {
            unimplemented!();
            // let side1_deriv = derivative(side1, deriv_char);
            // let side2_deriv = derivative(side2, deriv_char);
            // side1_deriv.union(&side2_deriv).cloned().collect()
        }
        GenRegex::Intersect(left, right) => {
            unimplemented!();
            // let p_deriv = derivative(left, deriv_char);
            // let q_deriv = derivative(right, deriv_char);
            // let mut ret_set = HashSet::new();
            // for p_sub in &p_deriv {
            //     for q_sub in &q_deriv {
            //         if let Some(ret) = merge_derivs_intersect(p_sub, q_sub) {
            //             ret_set.insert(ret);
            //         }
            //     }
            // }
            // ret_set
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
            unimplemented!();

            // let p_derivs = derivative(expr, deriv_char);
            // let mut ret_set = HashSet::new();
            // for p_deriv in &p_derivs {
            //     let ret = apply_deriv_kleene(p_deriv, gre);
            //     ret_set.insert(ret);
            // }
            // ret_set
        }
        GenRegex::Complement(_) => {
            unimplemented!();

            // let deriv = brzozowski::deriv::derivative(gre, deriv_char);
            // AntimirovElement::new(deriv, SimpleSub::empty()).into_set()
        }
        GenRegex::IfThenElse(_, _, _) => {
            unimplemented!();

            // let deriv = brzozowski::deriv::derivative(gre, deriv_char);
            // AntimirovElement::new(deriv, SimpleSub::empty()).into_set()
        }
        GenRegex::StringSlice(_, _) => {
            unimplemented!();
        }
        GenRegex::StringIndex(_) => {
            unimplemented!();
        }
    }
}
