//!
//! Implementation of the Antimirov Derivative
//!

// TODO: fix and remove
#![allow(unused_variables)]
#![allow(clippy::single_match)]

use crate::brzozowski;
use crate::classes::{
    AntimirovElement, AnySub, CharExpression, CharVar, GenRegex, MaybeCharExpression, Predicate,
    RangeConstr, SimpleSub, StringIndex, StringVar, SubExpr,
};

use disjoint_sets::UnionFind;
use std::collections::BTreeMap;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/*
    Union-find functions
*/

fn union_over_set(
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

fn count_union_elems(substitutions: &AnySub) -> usize {
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
    for c_exprs in substitutions.get_char_map().values() {
        count += c_exprs.len() + 1;
    }
    count
}

/*
    Substitution operations: merge, difference, and sub_in
*/

fn merge(substitutions: AnySub) -> Option<SimpleSub> {
    let mut str_eq_class = substitutions.get_str_map().clone();
    let char_eq_class = substitutions.get_char_map().clone();

    //let mut union_set: HashSet<Rc<CharExpression>> = HashSet::new();
    let mut expr_to_id: HashMap<Rc<CharExpression>, usize> = HashMap::new();
    let mut id_to_expr: HashMap<usize, Rc<CharExpression>> = HashMap::new();
    let mut canonical_map: HashMap<Rc<CharExpression>, Rc<CharExpression>> = HashMap::new();
    let mut union_find: UnionFind<usize> = UnionFind::new(count_union_elems(&substitutions) + 1);

    for eq_exprs in str_eq_class.values_mut() {
        let mut ind = 0;
        while eq_exprs.len() > 1 {
            let mut length_flag = false;
            let mut union_set: HashSet<Rc<CharExpression>> = HashSet::new();
            let mut i = 0;
            while i < eq_exprs.len() {
                let curr_sub_expr = &eq_exprs[i];
                if ind < curr_sub_expr.head_length() {
                    let temp = &curr_sub_expr[ind];
                    union_set.insert(Rc::new(temp.clone()));
                    i += 1;
                } else if curr_sub_expr.get_tail() && eq_exprs.len() > 1 {
                    eq_exprs.remove(i);
                } else {
                    for (j, item_j) in eq_exprs.iter().enumerate() {
                        if i != j {
                            if ind < item_j.head_length() {
                                return None;
                            } else {
                                continue;
                            }
                        }
                    }
                    //str_eq_class.insert(var.clone(), vec![curr_sub_expr.clone()]);
                    let new_vec = vec![curr_sub_expr.clone()];

                    // Move the ownership of `new_vec` to `eq_exprs`
                    *eq_exprs = new_vec;
                    //eq_exprs = &mut vec![curr_sub_expr.clone()];
                    length_flag = true;
                    break;
                }
            }
            if length_flag {
                break;
            }
            ind += 1;
            if !union_over_set(
                &mut union_find,
                &union_set,
                &mut expr_to_id,
                &mut id_to_expr,
                &mut canonical_map,
            ) {
                return None;
            } //TODO: Union everything together here (add in union_find element)
        }
    }
    let mut combined_expr: SimpleSub = SimpleSub::empty();
    let mut char_set = HashSet::new();
    for (var, eq_exprs) in &char_eq_class {
        let mut temp_set: HashSet<Rc<CharVar>> = eq_exprs
            .iter()
            .filter_map(|expr| {
                if let CharExpression::CharVar(ref var) = *expr {
                    Some(Rc::new(var.clone()))
                } else {
                    None
                }
            })
            .collect();
        let mut u_set: HashSet<_> = eq_exprs
            .iter()
            .map(|expr| Rc::new((expr).clone())) // Dereference `expr` (&&CharExpression) and clone
            .collect();
        u_set.insert(Rc::new(CharExpression::CharVar(var.clone())));
        temp_set.insert(Rc::new(var.clone()));

        if !union_over_set(
            &mut union_find,
            &u_set,
            &mut expr_to_id,
            &mut id_to_expr,
            &mut canonical_map,
        ) {
            return None;
        }
        char_set = char_set.union(&temp_set).cloned().collect();
    }
    for var in char_set {
        let deref = var.as_ref();
        let char_expression = Rc::new(CharExpression::CharVar(var.as_ref().clone()));
        let id_var = expr_to_id[&char_expression];
        let found_expr = id_to_expr[&union_find.find(id_var)].clone();
        match canonical_map.get(&char_expression) {
            Some(value) => combined_expr.set_char_var(var.as_ref().clone(), value.as_ref().clone()),
            None => {
                if CharExpression::CharVar(deref.clone()) != *found_expr {
                    combined_expr.set_char_var(deref.clone(), found_expr.as_ref().clone());
                }
            }
        }
    }

    //let string_subs = sub_in(string_subs, char_subs); //TODO: implement sub_in
    //
    for (var, mut eq_exprs) in str_eq_class {
        let sub_expr_vector = eq_exprs[0].get_mut_head();
        for (i, item) in sub_expr_vector.iter_mut().enumerate() {
            match item {
                CharExpression::CharVar(c_var) => {
                    let substitution_value = combined_expr.get_char_var(c_var);
                    match substitution_value {
                        Some(v) => {
                            // The key was found, and `v` is the value, so update the vector element
                            *item = v.clone();
                            //println!("Updated value at index {}: {:?}", i, v);
                        }
                        None => {
                            // The key was not found, so do nothing
                            //println!("No value found for key at index {}", i);
                        }
                    }
                }
                _ => {}
            }
        }
        combined_expr.set_string_var(var.clone(), eq_exprs[0].clone());
    }
    Some(combined_expr)
}

fn merge_binary(sub1: &SimpleSub, sub2: &SimpleSub) -> Option<SimpleSub> {
    let union_lr: AnySub = sub1.clone().union(sub2.clone());
    merge(union_lr)
}

fn merge_sets(subs1: &HashSet<SimpleSub>, subs2: &HashSet<SimpleSub>) -> HashSet<SimpleSub> {
    let mut result = HashSet::new();
    for sub1 in subs1 {
        for sub2 in subs2 {
            if let Some(ret) = merge_binary(sub1, sub2) {
                result.insert(ret);
            }
        }
    }
    result
}

fn union_sets(subs1: HashSet<SimpleSub>, subs2: HashSet<SimpleSub>) -> HashSet<SimpleSub> {
    let mut result = subs1;
    result.extend(subs2);
    result
}

fn sub_difference_from_merge(merged: &SimpleSub, sub: &SimpleSub) -> Option<SimpleSub> {
    let mut retsub = merged.clone();
    for char_var in sub.get_char_map().keys() {
        retsub.remove_char_map(char_var);
    }
    for (string_var, sub_expr1) in merged.get_str_map() {
        if let Some(sub_expr2) = sub.get_string_var(string_var) {
            retsub.remove_str_map(string_var);
            if let Some(mut sub) = sub_expr_match(sub_expr1, sub_expr2, string_var) {
                retsub.get_char_map_mut().append(sub.get_char_map_mut());
                retsub.get_str_map_mut().append(sub.get_str_map_mut());
            } else {
                return None;
            }
        }
    }
    Some(retsub)
}

// Note: no longer used atm in favor of sub_difference_from_merge
fn sub_difference(sub1: &SimpleSub, sub2: &SimpleSub) -> Option<SimpleSub> {
    if let Some(result) = merge_binary(sub1, sub2) {
        sub_difference_from_merge(&result, sub2)
    } else {
        None
    }
}

fn sub_expr_match(
    sub_expr1: &SubExpr,
    sub_expr2: &SubExpr,
    str_var: &StringVar,
) -> Option<SimpleSub> {
    let mut retval = SimpleSub::empty();
    if sub_expr1.is_empty() && sub_expr2.is_empty() {
        return Some(retval);
    } else if sub_expr1.head_length() == 0 && sub_expr1.get_tail() {
        retval.set_string_var(str_var.clone(), sub_expr2.clone());
        return Some(retval);
    } else if sub_expr2.head_length() == 0 && sub_expr2.get_tail() {
        retval.set_string_var(str_var.clone(), sub_expr1.clone());
        return Some(retval);
    } else if sub_expr1.is_empty() || sub_expr2.is_empty() {
        return None;
    }
    let trunc_sub_expr1 = SubExpr::new(sub_expr1.get_head()[1..].to_vec(), sub_expr1.get_tail());
    let trunc_sub_expr2 = SubExpr::new(sub_expr2.get_head()[1..].to_vec(), sub_expr2.get_tail());
    match sub_expr_match(&trunc_sub_expr1, &trunc_sub_expr2, str_var) {
        Some(val) => retval = val,
        None => return None,
    }
    let head1 = &sub_expr1.get_head()[0];
    let head2 = &sub_expr2.get_head()[0];
    if let CharExpression::CharVar(key) = head1 {
        retval.set_char_var(key.clone(), head2.clone());
    } else if let CharExpression::CharVar(key) = head2 {
        retval.set_char_var(key.clone(), head1.clone());
    }
    Some(retval)
}

fn sub_in(expr: &Rc<GenRegex>, substitution: &SimpleSub) -> Rc<GenRegex> {
    if substitution.get_str_map().is_empty() && substitution.get_char_map().is_empty() {
        return expr.clone(); // Returns a clone of expr.
    }
    match expr.as_ref() {
        GenRegex::EmptySet => Rc::clone(expr),
        GenRegex::Epsilon => Rc::clone(expr),
        GenRegex::Sigma => Rc::clone(expr),
        GenRegex::Range(char1, char2) => Rc::clone(expr),
        GenRegex::CharExpression(char_expr) => match char_expr {
            CharExpression::CharVar(char_var) => match substitution.get_char_var(char_var) {
                Some(value) => Rc::new(GenRegex::CharExpression(value.clone())),
                None => expr.clone(),
            },
            CharExpression::Literal(_) => expr.clone(),
        },
        GenRegex::StringVar(string_var) => match substitution.get_string_var(string_var) {
            Some(value) => value.to_gen_regex(string_var),
            None => expr.clone(),
        },
        GenRegex::StringIndex(string_index) => {
            match substitution.get_string_var(&string_index.var) {
                Some(value) => {
                    let index = string_index.index as usize;
                    let length = value.get_head().len();
                    if index < length {
                        Rc::new(GenRegex::CharExpression(value.get_head()[index].clone()))
                    } else if value.get_tail() {
                        Rc::new(GenRegex::StringIndex(StringIndex {
                            var: string_index.var.clone(),
                            index: ((index - length + 1) as i32),
                        }))
                    } else {
                        Rc::new(GenRegex::EmptySet)
                    }
                }
                None => expr.clone(),
            }
        }
        GenRegex::Union(gen_regex1, gen_regex2) => Rc::new(GenRegex::Union(
            sub_in(gen_regex1, substitution),
            sub_in(gen_regex2, substitution),
        )),
        GenRegex::Intersect(gen_regex1, gen_regex2) => Rc::new(GenRegex::Intersect(
            sub_in(gen_regex1, substitution),
            sub_in(gen_regex2, substitution),
        )),
        GenRegex::Concatenation(gen_regex1, gen_regex2) => Rc::new(GenRegex::Concatenation(
            sub_in(gen_regex1, substitution),
            sub_in(gen_regex2, substitution),
        )),
        GenRegex::Kleene(gen_regex) => Rc::new(GenRegex::Kleene(sub_in(gen_regex, substitution))),
        GenRegex::Complement(gen_regex) => {
            Rc::new(GenRegex::Complement(sub_in(gen_regex, substitution)))
        }
        GenRegex::IfThenElse(predicate, gen_regex1, gen_regex2) => {
            // TODO: Placeholder
            // Implement this case
            expr.clone()
        }
        GenRegex::StringSlice(string_var, _) => {
            // TODO: Placeholder
            // Implement this case
            expr.clone()
        }
    }
}

fn sub_in_predicate(pred: &Rc<Predicate>, sub: &SimpleSub) -> Rc<Predicate> {
    match pred.as_ref() {
        Predicate::True => Rc::clone(pred),
        Predicate::False => Rc::clone(pred),
        Predicate::Not(p) => Rc::new(Predicate::Not(sub_in_predicate(p, sub))),
        Predicate::And(p1, p2) => Rc::new(Predicate::And(
            sub_in_predicate(p1, sub),
            sub_in_predicate(p2, sub),
        )),
        Predicate::Or(p1, p2) => Rc::new(Predicate::Or(
            sub_in_predicate(p1, sub),
            sub_in_predicate(p2, sub),
        )),
        Predicate::Equals(expr1, expr2) => {
            let new_expr1 = sub_in_maybe_char_expr(expr1, sub);
            let new_expr2 = sub_in_maybe_char_expr(expr2, sub);
            Rc::new(Predicate::Equals(new_expr1, new_expr2))
        }
        Predicate::LessThan(expr, c) => {
            let new_expr = sub_in_maybe_char_expr(expr, sub);
            Rc::new(Predicate::LessThan(new_expr, *c))
        }
        Predicate::GreaterThan(expr, c) => {
            let new_expr = sub_in_maybe_char_expr(expr, sub);
            Rc::new(Predicate::LessThan(new_expr, *c))
        }
        Predicate::EqualLength(var, len) => {
            // TODO
            unimplemented!()
            // let new_var = sub_in_string_var(var, sub);
            // Rc::new(Predicate::EqualLength(new_var, *len))
        }
    }
}

fn sub_in_maybe_char_expr(expr: &MaybeCharExpression, sub: &SimpleSub) -> Rc<MaybeCharExpression> {
    match expr {
        MaybeCharExpression::CharExpression(c_expr) => {
            let new_expr = sub_in_char_expr(c_expr, sub);
            Rc::new(MaybeCharExpression::CharExpression(new_expr))
        }
        MaybeCharExpression::StringIndex(string_index) => {
            // TODO
            unimplemented!()
            // let new_var = sub_in_string_var(&string_index.var, sub);
            // Rc::new(MaybeCharExpression::StringIndex(StringIndex {
            //     var: new_var,
            //     index: string_index.index,
            // }))
        }
    }
}

fn sub_in_char_expr(expr: &CharExpression, sub: &SimpleSub) -> CharExpression {
    match expr {
        CharExpression::CharVar(var) => sub.get_char_var(var).unwrap_or(expr).clone(),
        CharExpression::Literal(_) => expr.clone(),
    }
}

fn merge_range_constraints(
    constraints1: &BTreeMap<CharVar, RangeConstr>,
    constraints2: &BTreeMap<CharVar, RangeConstr>,
) -> Option<BTreeMap<CharVar, RangeConstr>> {
    let mut constraints = constraints1.clone();
    for (key, val) in constraints2 {
        if let Some(other_val) = constraints.get(key) {
            if let Some(merged_range) = val.intersect(other_val) {
                constraints.insert(key.clone(), merged_range);
            } else {
                return None;
            }
        } else {
            constraints.insert(key.clone(), val.clone());
        }
    }
    Some(constraints)
}

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
        GenRegex::Sigma => {
            AntimirovElement::new(GenRegex::epsilon(), SimpleSub::empty(), BTreeMap::new())
                .into_set()
        }
        GenRegex::Range(char1, char2) => {
            let mut result =
                AntimirovElement::new(GenRegex::epsilon(), SimpleSub::empty(), BTreeMap::new());
            match deriv_char.as_ref() {
                CharExpression::Literal(literal) => {
                    if literal < char1 || literal > char2 {
                        return HashSet::new();
                    }
                }
                CharExpression::CharVar(deriv_var) => {
                    result.add_range(deriv_var.clone(), *char1, *char2);
                }
            }

            result.into_set()
        }
        GenRegex::CharExpression(c_expr) => match (deriv_char.as_ref(), c_expr) {
            (CharExpression::Literal(deriv_lit), CharExpression::Literal(literal_value)) => {
                if deriv_lit == literal_value {
                    AntimirovElement::new(GenRegex::epsilon(), SimpleSub::empty(), BTreeMap::new())
                        .into_set()
                } else {
                    HashSet::new()
                }
            }
            (CharExpression::CharVar(d_var), CharExpression::Literal(lit_val)) => {
                let mut char_to = BTreeMap::new();
                char_to.insert(d_var.clone(), c_expr.clone());
                let subs = SimpleSub::new(BTreeMap::new(), char_to);
                AntimirovElement::new(GenRegex::epsilon(), subs, BTreeMap::new()).into_set()
            }
            (_, CharExpression::CharVar(c_var)) => {
                let mut char_to = BTreeMap::new();
                char_to.insert(c_var.clone(), deriv_char.as_ref().clone());
                let subs = SimpleSub::new(BTreeMap::new(), char_to);
                AntimirovElement::new(GenRegex::epsilon(), subs, BTreeMap::new()).into_set()
            }
        },
        GenRegex::StringVar(string_var) => {
            let head = vec![deriv_char.as_ref().clone()];

            let subexpr = SubExpr::new(head, true);

            let mut string_to = BTreeMap::new();
            string_to.insert(string_var.clone(), subexpr);

            let substitution = SimpleSub::new(string_to, BTreeMap::new());

            AntimirovElement::new(gre.clone(), substitution, BTreeMap::new()).into_set()
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
            let deriv = brzozowski::derivative(gre, deriv_char);
            AntimirovElement::new(deriv, SimpleSub::empty(), BTreeMap::new()).into_set()
        }
        GenRegex::IfThenElse(_, _, _) => {
            let deriv = brzozowski::derivative(gre, deriv_char);
            AntimirovElement::new(deriv, SimpleSub::empty(), BTreeMap::new()).into_set()
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
    let l_sub = left.get_subs();
    let r_sub = right.get_subs();
    let merged_sub = merge_binary(l_sub, r_sub)?;

    // Merge range constraints
    let l_range = left.get_ranges();
    let r_range = right.get_ranges();
    let constraints = merge_range_constraints(l_range, r_range)?;

    // Compute the difference and apply needed remaining subs in left and right
    let l_minus_r = sub_difference_from_merge(&merged_sub, r_sub)?;
    let r_minus_l = sub_difference_from_merge(&merged_sub, l_sub)?;
    // let l_minus_r = sub_difference(Rc::new(l_sub.clone()), Rc::new(r_sub.clone()))?;
    // let r_minus_l = sub_difference(Rc::new(r_sub.clone()), Rc::new(l_sub.clone()))?;
    let l_expr = left.get_expr();
    let r_expr = right.get_expr();
    let p_prime_sub = sub_in(l_expr, &r_minus_l);
    let q_prime_sub = sub_in(r_expr, &l_minus_r);
    let final_expr = Rc::new(GenRegex::Intersect(p_prime_sub, q_prime_sub));
    Some(AntimirovElement::new(final_expr, merged_sub, constraints))
}

fn merge_derivs_concat(n_sub: &SimpleSub, right: &AntimirovElement) -> Option<AntimirovElement> {
    let r_sub = right.get_subs();
    let merged_sub = merge_binary(n_sub, r_sub)?;
    let r_ranges = right.get_ranges();

    let r_minus_l = sub_difference_from_merge(&merged_sub, r_sub)?;
    // let r_r_minus_l = sub_difference(Rc::new(n_sub.clone()), Rc::new(right_elem.clone()))?;
    let result = sub_in(right.get_expr(), &r_minus_l);
    Some(AntimirovElement::new(result, merged_sub, r_ranges.clone()))
}

fn apply_deriv_concat(
    left_deriv: &AntimirovElement,
    right: &Rc<GenRegex>,
) -> Option<AntimirovElement> {
    let l_expr = left_deriv.get_expr();
    let l_sub = left_deriv.get_subs();
    let l_ranges = left_deriv.get_ranges();
    Some(AntimirovElement::new(
        Rc::new(GenRegex::Concatenation(
            l_expr.clone(),
            sub_in(right, l_sub),
        )),
        l_sub.clone(),
        l_ranges.clone(),
    ))
}

fn apply_deriv_kleene(left_deriv: &AntimirovElement, right: &Rc<GenRegex>) -> AntimirovElement {
    let l_expr = left_deriv.get_expr();
    let l_sub = left_deriv.get_subs();
    let l_ranges = left_deriv.get_ranges();
    AntimirovElement::new(
        Rc::new(GenRegex::Concatenation(
            l_expr.clone(),
            sub_in(right, l_sub),
        )),
        l_sub.clone(),
        l_ranges.clone(),
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
        GenRegex::Range(_, _) => HashSet::new(),
        GenRegex::CharExpression(c_expr) => HashSet::new(),
        GenRegex::StringVar(s_var) => {
            let mut subs = HashSet::new();
            let mut string_to = BTreeMap::new();
            string_to.insert(s_var.clone(), SubExpr::empty());
            let string_sub = SimpleSub::new(string_to, BTreeMap::new());
            subs.insert(string_sub);
            subs
        }
        GenRegex::Union(side1, side2) => {
            let left_null = nullable(&side1);
            let right_null = nullable(&side2);
            union_sets(left_null, right_null)
        }
        GenRegex::Intersect(side1, side2) | GenRegex::Concatenation(side1, side2) => {
            let left_null = nullable(&side1);
            let right_null = nullable(&side2);
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
            let left_null = nullable_complement(&side1);
            let right_null = nullable_complement(&side2);
            merge_sets(&left_null, &right_null)
        }
        GenRegex::Intersect(side1, side2) | GenRegex::Concatenation(side1, side2) => {
            // Matches logic for GenRegex::Union in the regular nullable case
            let left_null = nullable_complement(&side1);
            let right_null = nullable_complement(&side2);
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
            unimplemented!();
        }
        GenRegex::StringIndex(_) => {
            unimplemented!();
        }
    }
}

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
            // TODO: Best to handle this as range constraints
            unimplemented!()
        }
        Predicate::GreaterThan(expr, c) => {
            // TODO: Best to handle this as range constraints
            unimplemented!()
        }
    }
}

fn sub_from_eq(
    expr1: &Rc<MaybeCharExpression>,
    expr2: &Rc<MaybeCharExpression>,
) -> (HashSet<SimpleSub>, HashSet<SimpleSub>) {
    match expr1.as_ref() {
        MaybeCharExpression::CharExpression(c1) => {
            match c1 {
                CharExpression::CharVar(var1) => {
                    // let mut sub = SimpleSub::empty();
                    // TODO
                    unimplemented!()
                    // sub.set_char_var(var1.clone(), expr2.as_ref().clone());
                    // (sub.into_set(), HashSet::new())
                }
                CharExpression::Literal(_) => {
                    // TODO
                    unimplemented!()
                    // (SimpleSub::empty().into_set(), HashSet::new())
                }
            }
        }
        MaybeCharExpression::StringIndex(_) => {
            // TODO
            unimplemented!()
        }
    }
}

pub fn sub_from_eq_len(var: &StringVar, len: &i32) -> (HashSet<SimpleSub>, HashSet<SimpleSub>) {
    // TODO
    unimplemented!()
}

/*
    Top-level functions: satisfiable and matching
*/

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

/*fn assign_unique_ids(
    substitutions: GenRegexPairSet,
    id_map: &mut HashMap<GenRegex, i32>,
    next_id: &mut i32,
) {
    for sub in &substitutions {
        match sub.0.as_ref() {
            GenRegex::StringVar(_) => {
                let index_str = &sub.0;
                id_map.entry(index_str.as_ref().clone()).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
            GenRegex::CharExpression(c_expr) => {
                if let CharExpression::CharVar(_) = c_expr.as_ref() {
                    let index_str = &sub.0;
                    id_map.entry(index_str.as_ref().clone()).or_insert_with(|| {
                        let id = *next_id;
                        *next_id += 1;
                        id
                    });
                }
            }
            _ => {}
        }
        match sub.1.as_ref() {
            GenRegex::StringVar(_) => {
                let index_str = &sub.1;
                id_map.entry(index_str.as_ref().clone()).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
            GenRegex::CharExpression(_) => {
                let index_str = &sub.1;
                id_map.entry(index_str.as_ref().clone()).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
            _ => {}
        }
    }
}*/
