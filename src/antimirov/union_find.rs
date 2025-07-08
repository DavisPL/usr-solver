//!
//! Union-find functions
//!

use super::subs::AnySub;
use crate::types::expr::CharExpression;

pub use disjoint_sets::UnionFind;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub fn union_over_set(
    union_find: &mut UnionFind<usize>,
    union_set: &HashSet<Rc<CharExpression>>,
    expr_to_id: &mut HashMap<Rc<CharExpression>, usize>,
    id_to_expr: &mut HashMap<usize, Rc<CharExpression>>,
    canonical_map: &mut HashMap<Rc<CharExpression>, Rc<CharExpression>>,
) -> bool {
    let mut prev: std::option::Option<Rc<CharExpression>> = None;

    for element in union_set {
        if matches!(element.as_ref(), CharExpression::Literal(_)) {
            canonical_map.insert(element.clone(), element.clone());
        }

        if let Some(prev_exists) = prev {
            let prev_id: usize;
            let curr_id: usize;
            if expr_to_id.contains_key(&prev_exists) {
                prev_id = expr_to_id[&prev_exists];
            } else {
                prev_id = expr_to_id.len() + 1;
                expr_to_id.insert(prev_exists.clone(), prev_id);
                id_to_expr.insert(prev_id, prev_exists.clone());
            }
            if expr_to_id.contains_key(element.as_ref()) {
                curr_id = expr_to_id[element.as_ref()];
            } else {
                curr_id = expr_to_id.len() + 1;
                expr_to_id.insert(element.clone(), curr_id);
                //expr_to_id[element.as_ref()] = curr_id;
                id_to_expr.insert(curr_id, element.clone());
            } // By this point in the code we should have the ID for the 2 elements we are unioning
            if canonical_map.contains_key(&prev_exists)
                && canonical_map.contains_key(element.as_ref())
                && canonical_map[&prev_exists] != canonical_map[element.as_ref()]
            {
                return false;
            }
            union_find.union(prev_id, curr_id); //ERROR HERE
            if canonical_map.contains_key(element.as_ref()) {
                canonical_map.insert(prev_exists, canonical_map[element.as_ref()].clone());
            } else if canonical_map.contains_key(&prev_exists) {
                canonical_map.insert(element.clone(), canonical_map[&prev_exists].clone());
            }
        }
        prev = Some(element.clone())
    }
    true
}

pub fn count_union_elems(substitutions: &AnySub) -> usize {
    /*let mut char_vars: HashSet<CharExpression> = HashSet::new();
    for sub in substitutions.get_str_map().values(){
        for sub_expr in sub{
            for c_expr in sub_expr.get_head(){
                char_vars.insert(c_expr);
            }
        }

    }
    for c_exprs in substitutions.get_char_map(){
        for c in c_exprs{
            char_vars.insert(c_expr);
        }
    }
    return char_vars.len();*/
    let mut count: usize = 0;
    for sub in substitutions.get_str_map().values() {
        for sub_expr in sub {
            count += sub_expr.get_head().len();
        }
    }
    for sub in substitutions.get_not_constraints() {
        count += 1
    }
    for c_exprs in substitutions.get_char_map().values() {
        count += c_exprs.len() + 1;
    }
    let ranges = substitutions.get_ranges();
    let Some(ranges) = ranges else {
        return count;
    };
    for r in ranges {
        count += 1;
    }
    count
}
