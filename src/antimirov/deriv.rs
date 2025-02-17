//!
//! Implementation of the Antimirov Derivative
//!

// TODO: fix and remove
#![allow(unused_variables)]

use super::determinized::derivative_determinized;
use super::sub_from_predicate::sub_from_predicate;
use super::subs::{
    merge_binary, merge_sets, sub_difference_from_merge, sub_in, union_sets, AntimirovElement,
    SimpleSub, SubExpr,
};

use crate::brzozowski;
use crate::types::expr::CharExpression;
use crate::types::regex::GenRegex;

use std::collections::{BTreeMap, HashSet};
use std::rc::Rc;

/*
    The main derivative operation
*/

pub fn derivative(
    gre: &Rc<GenRegex>,
    deriv_char: &Rc<CharExpression>,
) -> HashSet<AntimirovElement> {
    // println!("taking d({}, {})", gre, deriv_char);

    match gre.as_ref() {
        GenRegex::EmptySet => HashSet::new(),
        GenRegex::Epsilon => HashSet::new(),
        GenRegex::Sigma => AntimirovElement::new_epsilon().into_set(),
        GenRegex::SigmaStar => AntimirovElement::new_emptysub(gre.clone()).into_set(),
        GenRegex::Range(char1, char2) => match deriv_char.as_ref() {
            CharExpression::Literal(literal) => {
                if literal < char1 || literal > char2 {
                    HashSet::new()
                } else {
                    AntimirovElement::new_epsilon().into_set()
                }
            }
            CharExpression::CharVar(deriv_var) => {
                AntimirovElement::new_epsilon_range(deriv_var.clone(), *char1, *char2).into_set()
            }
        },
        GenRegex::CharExpression(c_expr) => match (deriv_char.as_ref(), c_expr) {
            (CharExpression::Literal(deriv_lit), CharExpression::Literal(literal_value)) => {
                if deriv_lit == literal_value {
                    AntimirovElement::new_epsilon().into_set()
                } else {
                    HashSet::new()
                }
            }
            (CharExpression::CharVar(d_var), CharExpression::Literal(lit_val)) => {
                let mut char_to = BTreeMap::new();
                char_to.insert(d_var.clone(), c_expr.clone());
                let subs = SimpleSub::new(BTreeMap::new(), char_to, BTreeMap::new());
                AntimirovElement::new(GenRegex::epsilon(), subs).into_set()
            }
            (_, CharExpression::CharVar(c_var)) => {
                let mut char_to = BTreeMap::new();
                char_to.insert(c_var.clone(), deriv_char.as_ref().clone());
                let subs = SimpleSub::new(BTreeMap::new(), char_to, BTreeMap::new());
                AntimirovElement::new(GenRegex::epsilon(), subs).into_set()
            }
        },
        GenRegex::StringVar(string_var) => {
            let head = vec![deriv_char.as_ref().clone()];

            let subexpr = SubExpr::new(head, true);

            let mut string_to = BTreeMap::new();
            string_to.insert(string_var.clone(), subexpr);

            let substitution = SimpleSub::new(string_to, BTreeMap::new(), BTreeMap::new());

            AntimirovElement::new(gre.clone(), substitution).into_set()
        }
        GenRegex::Union(side1, side2) => {
            let side1_deriv = derivative(side1, deriv_char);
            let side2_deriv = derivative(side2, deriv_char);
            side1_deriv.union(&side2_deriv).cloned().collect()
        }
        GenRegex::Intersect(left, right) => {
            let p_deriv = derivative(left, deriv_char);
            let q_deriv = derivative(right, deriv_char);
            let mut ret_set = HashSet::new();
            for p_sub in &p_deriv {
                for q_sub in &q_deriv {
                    if let Some(ret) = merge_derivs_intersect(p_sub, q_sub) {
                        ret_set.insert(ret);
                    }
                }
            }
            ret_set
        }
        GenRegex::Concatenation(left, right) => {
            let left_deriv = derivative(left, deriv_char);
            let mut ret_set = HashSet::new();
            for sub in &left_deriv {
                if let Some(ret) = apply_deriv_concat(sub, right) {
                    ret_set.insert(ret);
                }
            }

            let p_nullable = nullable(left);
            if !p_nullable.is_empty() {
                let right_deriv = derivative(right, deriv_char);
                for n_sub in &p_nullable {
                    for q_sub in &right_deriv {
                        if let Some(ret) = merge_derivs_concat(n_sub, q_sub) {
                            ret_set.insert(ret);
                        }
                    }
                }
            }

            ret_set
        }
        GenRegex::Kleene(expr) => {
            let p_derivs = derivative(expr, deriv_char);
            let mut ret_set = HashSet::new();
            for p_deriv in &p_derivs {
                let ret = apply_deriv_kleene(p_deriv, gre);
                ret_set.insert(ret);
            }
            ret_set
        }
        GenRegex::Complement(_) => {
            // Use Determinized derivative
            // Filter out empty sets
            derivative_determinized(gre, deriv_char)
                .into_iter()
                .filter(|x| !matches!(x.get_expr().as_ref(), GenRegex::EmptySet))
                .collect()
        }
        GenRegex::IfThenElse(_, _, _) => {
            // Use Brzozowski derivative
            let deriv = brzozowski::deriv::derivative(gre, deriv_char);
            AntimirovElement::new(deriv, SimpleSub::empty()).into_set()
        }
        GenRegex::StringSlice(_, _) => {
            unimplemented!();
        }
        GenRegex::StringIndex(_) => {
            unimplemented!();
        }
    }
}

/*
    Derivative helpers
*/

fn merge_derivs_intersect(
    left: &AntimirovElement,
    right: &AntimirovElement,
) -> Option<AntimirovElement> {
    // Merge subs
    // This also merges range constraints
    let l_sub = left.get_subs();
    let r_sub = right.get_subs();
    let merged_sub = merge_binary(l_sub, r_sub)?;

    // Compute the difference and apply needed remaining subs in left and right
    let l_minus_r = sub_difference_from_merge(&merged_sub, r_sub);
    let r_minus_l = sub_difference_from_merge(&merged_sub, l_sub);
    // let l_minus_r = sub_difference(Rc::new(l_sub.clone()), Rc::new(r_sub.clone()))?;
    // let r_minus_l = sub_difference(Rc::new(r_sub.clone()), Rc::new(l_sub.clone()))?;
    let l_expr = left.get_expr();
    let r_expr = right.get_expr();
    let p_prime_sub = sub_in(l_expr, &r_minus_l);
    let q_prime_sub = sub_in(r_expr, &l_minus_r);
    let final_expr = GenRegex::make_intersection(p_prime_sub, q_prime_sub);
    Some(AntimirovElement::new(final_expr, merged_sub))
}

fn merge_derivs_concat(n_sub: &SimpleSub, right: &AntimirovElement) -> Option<AntimirovElement> {
    // Merge subs and range constraints
    let r_sub = right.get_subs();
    let merged_sub = merge_binary(n_sub, r_sub)?;

    let r_minus_l = sub_difference_from_merge(&merged_sub, r_sub);
    // let r_r_minus_l = sub_difference(Rc::new(n_sub.clone()), Rc::new(right_elem.clone()))?;
    let result = sub_in(right.get_expr(), &r_minus_l);
    Some(AntimirovElement::new(result, merged_sub))
}

fn apply_deriv_concat(
    left_deriv: &AntimirovElement,
    right: &Rc<GenRegex>,
) -> Option<AntimirovElement> {
    let l_expr = left_deriv.get_expr();
    let l_sub = left_deriv.get_subs();
    Some(AntimirovElement::new(
        GenRegex::make_concatenation(l_expr.clone(), sub_in(right, l_sub)),
        l_sub.clone(),
    ))
}

fn apply_deriv_kleene(left_deriv: &AntimirovElement, right: &Rc<GenRegex>) -> AntimirovElement {
    let l_expr = left_deriv.get_expr();
    let l_sub = left_deriv.get_subs();
    AntimirovElement::new(
        GenRegex::make_concatenation(l_expr.clone(), sub_in(right, l_sub)),
        l_sub.clone(),
    )
}

/*
    Nullable
*/

pub fn nullable(gre: &Rc<GenRegex>) -> HashSet<SimpleSub> {
    match gre.as_ref() {
        GenRegex::EmptySet => HashSet::new(),
        GenRegex::Epsilon => SimpleSub::empty().into_set(),
        GenRegex::Sigma => HashSet::new(),
        GenRegex::SigmaStar => SimpleSub::empty().into_set(),
        GenRegex::Range(_, _) => HashSet::new(),
        GenRegex::CharExpression(c_expr) => HashSet::new(),
        GenRegex::StringVar(s_var) => {
            let mut string_to = BTreeMap::new();
            string_to.insert(s_var.clone(), SubExpr::empty());
            let string_sub = SimpleSub::new(string_to, BTreeMap::new(), BTreeMap::new());
            string_sub.into_set()
        }
        GenRegex::Union(side1, side2) => {
            let left_null = nullable(side1);
            let right_null = nullable(side2);
            union_sets(left_null, right_null)
        }
        GenRegex::Intersect(side1, side2) | GenRegex::Concatenation(side1, side2) => {
            let left_null = nullable(side1);
            let right_null = nullable(side2);
            merge_sets(&left_null, &right_null)
        }
        GenRegex::Kleene(_) => SimpleSub::empty().into_set(),
        GenRegex::Complement(gre1) => {
            // Use complement of the nullable function
            nullable_complement(gre1)
        }
        GenRegex::IfThenElse(p, g1, g2) => {
            let (left, right) = sub_from_predicate(p);
            let left_nullable = nullable(g1);
            let right_nullable = nullable(g2);
            let result1 = merge_sets(&left, &left_nullable);
            let result2 = merge_sets(&right, &right_nullable);
            union_sets(result1, result2)
        }
        GenRegex::StringSlice(_, _) => {
            unimplemented!();
        }
        GenRegex::StringIndex(_) => {
            unimplemented!();
        }
    }
}

/// Optimization to determine subs for whether the *complement* of a regex is nullable
/// Returns all subs under which the regex is *not* nullable.
pub fn nullable_complement(gre: &Rc<GenRegex>) -> HashSet<SimpleSub> {
    match gre.as_ref() {
        GenRegex::EmptySet => SimpleSub::empty().into_set(),
        GenRegex::Epsilon => HashSet::new(),
        GenRegex::Sigma => SimpleSub::empty().into_set(),
        GenRegex::SigmaStar => HashSet::new(),
        GenRegex::Range(_, _) => SimpleSub::empty().into_set(),
        GenRegex::CharExpression(c_expr) => SimpleSub::empty().into_set(),
        GenRegex::StringVar(s_var) => {
            // The hard case
            // Here we can just enumerate if we come across the case.
            // TODO
            unimplemented!()
            // let mut subs = HashSet::new();
            // let mut string_to = BTreeMap::new();
            // string_to.insert(s_var.clone(), SubExpr::empty());
            // let string_sub = SimpleSub::new(string_to, BTreeMap::new());
            // subs.insert(string_sub);
            // subs
        }
        GenRegex::Union(side1, side2) => {
            // Matches logic for GenRegex::Intersection in the regular nullable case
            let left_null = nullable_complement(side1);
            let right_null = nullable_complement(side2);
            merge_sets(&left_null, &right_null)
        }
        GenRegex::Intersect(side1, side2) | GenRegex::Concatenation(side1, side2) => {
            // Matches logic for GenRegex::Union in the regular nullable case
            let left_null = nullable_complement(side1);
            let right_null = nullable_complement(side2);
            union_sets(left_null, right_null)
        }
        GenRegex::Kleene(_) => HashSet::new(),
        GenRegex::Complement(gre1) => {
            // Flip the negation context
            nullable(gre1)
        }
        GenRegex::IfThenElse(p, g1, g2) => {
            let (left, right) = sub_from_predicate(p);
            let left_nullable = nullable_complement(g1);
            let right_nullable = nullable_complement(g2);
            let result1 = merge_sets(&left, &left_nullable);
            let result2 = merge_sets(&right, &right_nullable);
            union_sets(result1, result2)
        }
        GenRegex::StringSlice(_, _) => {
            // TODO
            unimplemented!();
        }
        GenRegex::StringIndex(_) => {
            // TODO
            unimplemented!()
        }
    }
}
