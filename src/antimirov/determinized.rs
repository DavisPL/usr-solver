//!
//! Determinized version of the Antimirov substitution-derivative.
//!
//! This is an experiment - just to see if it works.
//!

// TODO: fix and remove
#![allow(unused_variables)]

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
    // println!("d({}, {})", gre, deriv_char);

    // let result =
    match gre.as_ref() {
        GenRegex::EmptySet => AntimirovElement::new_empty().into_set(),
        GenRegex::Epsilon => AntimirovElement::new_empty().into_set(),
        GenRegex::Sigma => AntimirovElement::new_epsilon().into_set(),
        GenRegex::SigmaStar => AntimirovElement::new_emptysub(gre.clone()).into_set(),
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
                // TODO 1: Tiching
                determinize_range(d_var, *lit_val, *lit_val)
            }
            (CharExpression::Literal(lit_val), CharExpression::CharVar(c_var)) => {
                // TODO 2: Tiching
                determinize_range(c_var, *lit_val, *lit_val)
            }
            (CharExpression::CharVar(d_var), CharExpression::CharVar(c_var)) => {
                // TODO 3: Tiching
                // TODO: Hard case, requires encoding x != y
                unimplemented!();
            }
        },
        GenRegex::StringVar(string_var) => {
            // TODO 4: Tiching
            // Hard case, requires handling w |-> xw and negation of this
            eprintln!("Warning: determinized derivative case for StringVar may be unsound");
            unimplemented!();
            // TEMP - uncomment to run determinizing solver on examples
            // AntimirovElement::new_emptysub(gre.clone()).into_set()
        }
        GenRegex::Union(side1, side2) => {
            let side1_deriv = derivative_determinized(side1, deriv_char);
            let side2_deriv = derivative_determinized(side2, deriv_char);
            merge_helper(&side1_deriv, &side2_deriv, &|left, right| {
                GenRegex::make_union(left.clone(), right.clone())
            })
        }
        GenRegex::Intersect(side1, side2) => {
            let side1_deriv = derivative_determinized(side1, deriv_char);
            let side2_deriv = derivative_determinized(side2, deriv_char);
            merge_helper(&side1_deriv, &side2_deriv, &|left, right| {
                GenRegex::make_intersection(left.clone(), right.clone())
            })
        }
        GenRegex::Concatenation(left, right) => {
            // Derivative-of-left case
            let left_deriv = derivative_determinized(left, deriv_char);
            let right_copy = AntimirovElement::new_emptysub(right.clone()).into_set();
            let left_result = merge_helper(&left_deriv, &right_copy, &|l, r| {
                GenRegex::make_concatenation(l.clone(), r.clone())
            });

            // Derivative-of-right case
            let (left_nullable_yes, left_nullable_no) = nullable_determinized(left);
            if left_nullable_yes.is_empty() {
                left_result
            } else {
                let right_deriv = derivative_determinized(right, deriv_char);

                // Refine nullable case
                let not_nullable_right =
                    merge_helper(&left_nullable_no, &right_deriv, &|_null, _r| {
                        GenRegex::empty_set()
                    });
                let nullable_right =
                    merge_helper(&left_nullable_yes, &right_deriv, &|_null, r| r.clone());
                let right_result = not_nullable_right
                    .into_iter()
                    .chain(nullable_right)
                    .collect();

                // Merge both cases
                merge_helper(&left_result, &right_result, &|left, right| {
                    GenRegex::make_union(left.clone(), right.clone())
                })
            }
        }
        GenRegex::Kleene(expr) => {
            let p_derivs = derivative_determinized(expr, deriv_char);
            let right_copy = AntimirovElement::new_emptysub(gre.clone()).into_set();
            merge_helper(&right_copy, &p_derivs, &|left, right| {
                GenRegex::make_concatenation(left.clone(), right.clone())
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
            // TODO Caleb
            unimplemented!();
        }
        GenRegex::StringSlice(_, _) => {
            unimplemented!();
        }
        GenRegex::StringIndex(_) => {
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
    F: Fn(&Rc<GenRegex>, &Rc<GenRegex>) -> Rc<GenRegex>,
{
    // println!("    merge_helper left: {:?}", left_set);
    // println!("    merge_helper right: {:?}", right_set);
    let mut result = HashSet::new();
    for left in left_set {
        for right in right_set {
            if let Some(merged) = AntimirovElement::merge_using_safe(left, right, merge_op) {
                result.insert(merged);
            }
        }
    }
    // println!("    merge_helper result: {:?}", result);
    result
}

/*
    Determinized nullable

    Returns a set of AntimirovElements (varnothing, f, pi)
    and a set of AntimirovElements (varnothing, f, pi)
    where f is a substitution and pi is a range constraint
    such that all (f, pi) pairs form a partition of the entire space of valuations.

    The regex coordinate `varnothing` is just a placeholder. It isn't used by
    the functions in this file - technically, it would be more correct to return epsilon
    for the true case and empty for the false case.

    This is required to be a partition for the Concat case to work correctly in the main derivative.
*/

pub fn nullable_negation_helper(
    (true_case, false_case): (HashSet<AntimirovElement>, HashSet<AntimirovElement>),
) -> (HashSet<AntimirovElement>, HashSet<AntimirovElement>) {
    (false_case, true_case)
}

pub fn nullable_and_helper(
    (left_true, left_false): (HashSet<AntimirovElement>, HashSet<AntimirovElement>),
    (right_true, right_false): (HashSet<AntimirovElement>, HashSet<AntimirovElement>),
) -> (HashSet<AntimirovElement>, HashSet<AntimirovElement>) {
    let true_case = merge_helper(&left_true, &right_true, &|_left, _right| {
        GenRegex::empty_set()
    });
    let left_only = merge_helper(&left_true, &left_false, &|_left, _right| {
        GenRegex::empty_set()
    });
    let false_case = left_false.into_iter().chain(left_only).collect();
    (true_case, false_case)
}

pub fn nullable_or_helper(
    left: (HashSet<AntimirovElement>, HashSet<AntimirovElement>),
    right: (HashSet<AntimirovElement>, HashSet<AntimirovElement>),
) -> (HashSet<AntimirovElement>, HashSet<AntimirovElement>) {
    nullable_negation_helper(nullable_and_helper(
        nullable_negation_helper(left),
        nullable_negation_helper(right),
    ))
}

//TODO: Add not constraint tracking
pub fn nullable_determinized(
    gre: &Rc<GenRegex>,
) -> (HashSet<AntimirovElement>, HashSet<AntimirovElement>) {
    fn true_helper() -> (HashSet<AntimirovElement>, HashSet<AntimirovElement>) {
        (AntimirovElement::new_epsilon().into_set(), HashSet::new())
    }
    fn false_helper() -> (HashSet<AntimirovElement>, HashSet<AntimirovElement>) {
        (HashSet::new(), AntimirovElement::new_epsilon().into_set())
    }
    match gre.as_ref() {
        GenRegex::EmptySet => false_helper(),
        GenRegex::Epsilon => true_helper(),
        GenRegex::Sigma => false_helper(),
        GenRegex::SigmaStar => true_helper(),
        GenRegex::Range(char1, char2) => false_helper(),
        GenRegex::CharExpression(c_expr) => false_helper(),
        GenRegex::StringVar(s_var) => {
            let mut string_to = BTreeMap::new();
            string_to.insert(s_var.clone(), SubExpr::empty());
            //TODO: set correct not constraints for string_sub
            let string_sub =
                SimpleSub::new(string_to, BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
            let true_case = AntimirovElement::new(GenRegex::empty_set(), string_sub).into_set();

            // TODO: Ensure var is fresh
            let fresh = CharVar {
                name: "fresh".to_string(),
            };
            let fresh_expr = CharExpression::CharVar(fresh.clone());
            let subexpr = SubExpr::new(vec![fresh_expr], true);

            let mut string_to = BTreeMap::new();
            string_to.insert(s_var.clone(), subexpr);

            //TODO: set correct not constraints for substitutions
            let substitution =
                SimpleSub::new(string_to, BTreeMap::new(), BTreeMap::new(), BTreeMap::new());

            let false_case = AntimirovElement::new(GenRegex::empty_set(), substitution).into_set();

            (true_case, false_case)
        }
        GenRegex::Union(side1, side2) => {
            let left = nullable_determinized(side1);
            let right = nullable_determinized(side2);
            nullable_or_helper(left, right)
        }
        GenRegex::Intersect(side1, side2) | GenRegex::Concatenation(side1, side2) => {
            let left = nullable_determinized(side1);
            let right = nullable_determinized(side2);
            nullable_and_helper(left, right)
        }
        GenRegex::Kleene(_) => true_helper(),
        GenRegex::Complement(gre1) => nullable_negation_helper(nullable_determinized(gre1)),
        GenRegex::IfThenElse(p, g1, g2) => {
            // TODO Caleb
            unimplemented!();
        }
        GenRegex::StringSlice(_, _) => {
            unimplemented!();
        }
        GenRegex::StringIndex(_) => {
            unimplemented!();
        }
    }
}
