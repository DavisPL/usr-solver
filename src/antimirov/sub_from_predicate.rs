//!
//! SFromP (Sub from Predicate) function
//! And associated helper functions
//!

use super::subs::{merge_binary, merge_sets, union_sets, SimpleSub};
use super::util::{char_minus_one, char_plus_one, CHAR_MAX, CHAR_MIN};

use crate::types::expr::{CharExpression, MaybeCharExpression, StringIndex, StringVar};
use crate::types::predicate::Predicate;

use std::collections::HashSet;
use std::rc::Rc;

pub fn sub_from_predicate(p: &Predicate) -> (HashSet<SimpleSub>, HashSet<SimpleSub>) {
    match p {
        Predicate::True => (SimpleSub::empty().into_set(), HashSet::new()),
        Predicate::False => (HashSet::new(), SimpleSub::empty().into_set()),
        Predicate::Not(p) => {
            let (left, right) = sub_from_predicate(p);
            (right, left)
        }
        Predicate::And(p1, p2) => {
            let (left1, right1) = sub_from_predicate(p1);
            let (left2, right2) = sub_from_predicate(p2);
            (merge_sets(&left1, &left2), union_sets(right1, right2))
        }
        Predicate::Or(p1, p2) => {
            let (left1, right1) = sub_from_predicate(p1);
            let (left2, right2) = sub_from_predicate(p2);
            (union_sets(left1, left2), merge_sets(&right1, &right2))
        }
        Predicate::Equals(expr1, expr2) => sub_from_eq(expr1, expr2),
        Predicate::EqualLength(var, len) => sub_from_eq_len(var, len),
        Predicate::LessThan(expr, c) => {
            // Extract the char expression from MaybeCharExpression
            let (char_expr, succeed_sub, fail_subs) = extract_char_expr(expr);
            let (true_sub, false_sub) = sub_from_range_le(&char_expr, *c);

            // True case: extraction succeeds + char is in-bounds
            let true_subs = merge_sets(&true_sub, &succeed_sub.into_set());

            // False case: extraction fails OR char is out-of-bounds
            let false_subs = union_sets(false_sub, fail_subs);

            (true_subs, false_subs)
        }
        Predicate::GreaterThan(expr, c) => {
            // Extract the char expression from MaybeCharExpression
            let (char_expr, succeed_sub, fail_subs) = extract_char_expr(expr);
            let (true_sub, false_sub) = sub_from_range_ge(&char_expr, *c);

            // True case: extraction succeeds + char is in-bounds
            let true_subs = merge_sets(&true_sub, &succeed_sub.into_set());

            // False case: extraction fails OR char is out-of-bounds
            let false_subs = union_sets(false_sub, fail_subs);

            (true_subs, false_subs)
        }
    }
}

fn sub_from_eq(
    expr1: &Rc<MaybeCharExpression>,
    expr2: &Rc<MaybeCharExpression>,
) -> (HashSet<SimpleSub>, HashSet<SimpleSub>) {
    let (char_expr1, sub1, fails1) = extract_char_expr(expr1);
    let (char_expr2, sub2, fails2) = extract_char_expr(expr2);

    let mut true_cases = HashSet::new();
    let mut false_cases = fails1;
    false_cases.extend(fails2);

    if let Some(base_sub) = merge_binary(&sub1, &sub2) {
        match (char_expr1, char_expr2) {
            (CharExpression::CharVar(var1), exp2) => {
                let mut sub = SimpleSub::empty();
                sub.set_char_var(var1, exp2);
                if let Some(merged) = merge_binary(&sub, &base_sub) {
                    true_cases.insert(merged);
                }
            }
            (char_expr1, CharExpression::CharVar(var2)) => {
                let mut sub = SimpleSub::empty();
                sub.set_char_var(var2, char_expr1);
                if let Some(merged) = merge_binary(&sub, &base_sub) {
                    true_cases.insert(merged);
                }
            }
            (CharExpression::Literal(lit1), CharExpression::Literal(lit2)) => {
                if lit1 == lit2 {
                    true_cases.insert(base_sub);
                }
            }
        }
    }

    (true_cases, false_cases)
}

pub fn sub_from_eq_len(_var: &StringVar, _len: &i32) -> (HashSet<SimpleSub>, HashSet<SimpleSub>) {
    // TODO
    unimplemented!()
}

// Return substitutions for (<=, >)
fn sub_from_range_le(
    expr: &CharExpression,
    pivot: char,
) -> (HashSet<SimpleSub>, HashSet<SimpleSub>) {
    let (below, equal, above) = sub_from_char_compare(expr, pivot);
    let mut below_subs = HashSet::new();
    let mut above_subs = HashSet::new();
    if let Some(below) = below {
        below_subs.insert(below);
    }
    if let Some(equal) = equal {
        below_subs.insert(equal);
    }
    if let Some(above) = above {
        above_subs.insert(above);
    }
    (below_subs, above_subs)
}

// Return substitutions for >=, <
fn sub_from_range_ge(
    expr: &CharExpression,
    pivot: char,
) -> (HashSet<SimpleSub>, HashSet<SimpleSub>) {
    let (below, equal, above) = sub_from_char_compare(expr, pivot);
    let mut below_subs = HashSet::new();
    let mut above_subs = HashSet::new();
    if let Some(below) = below {
        below_subs.insert(below);
    }
    if let Some(equal) = equal {
        above_subs.insert(equal);
    }
    if let Some(above) = above {
        above_subs.insert(above);
    }
    (above_subs, below_subs)
}

// Return subsitutions for:
// 1. When expr < pivot
// 2. When expr = pivot
// 3. When expr > pivot
fn sub_from_char_compare(
    expr: &CharExpression,
    pivot: char,
) -> (Option<SimpleSub>, Option<SimpleSub>, Option<SimpleSub>) {
    // Local imports for char comparison
    use std::cmp::Ordering;

    // Handle using range constraints
    match expr {
        CharExpression::CharVar(var) => {
            let below = char_minus_one(pivot).map(|c| {
                let mut below = SimpleSub::empty();
                below.add_range(var.clone(), CHAR_MIN, c);
                below
            });
            let mut equal = SimpleSub::empty();
            equal.add_range(var.clone(), pivot, pivot);
            let above = char_plus_one(pivot).map(|c| {
                let mut above = SimpleSub::empty();
                above.add_range(var.clone(), c, CHAR_MAX);
                above
            });
            (below, Some(equal), above)
        }
        CharExpression::Literal(x) => {
            let result = SimpleSub::empty();
            match x.cmp(&pivot) {
                Ordering::Less => (Some(result), None, None),
                Ordering::Equal => (None, Some(result), None),
                Ordering::Greater => (None, None, Some(result)),
            }
        }
    }
}

// The following function turns out to be key: it allows us to convert a string
// index into a character expression by constructing a substitution on the fly.
// This requires a constructor for fresh variables, so for now, we include a placeholder.
//
// Returns:
// - a (char_expr, substitution) pair, for when the index succeeds
// - a set of (substitution)s for when the index is out of bounds.
fn string_index_to_char_expr(
    _string_index: &StringIndex,
) -> (CharExpression, SimpleSub, HashSet<SimpleSub>) {
    // TODO
    unimplemented!()
}

// Using string_index_to_char_expr to unwrap MaybeCharExpressions.
// Same return values as string_index_to_char_expr.
fn extract_char_expr(
    maybe_char_expr: &MaybeCharExpression,
) -> (CharExpression, SimpleSub, HashSet<SimpleSub>) {
    match maybe_char_expr {
        MaybeCharExpression::CharExpression(c_expr) => {
            (c_expr.clone(), SimpleSub::empty(), HashSet::new())
        }
        MaybeCharExpression::StringIndex(string_index) => string_index_to_char_expr(string_index),
    }
}
