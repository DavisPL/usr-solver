//!
//! Implementation of the Antimirov Derivative
//!

// TODO: fix and remove
#![allow(unused_variables)]
#![allow(clippy::single_match)]

use crate::classes::StringIndex;
use crate::classes::{
    AntimirovDerivativeElement, AnySub, CharExpression, CharVar, GenRegex, MergeResult, SimpleSub,
    StringVar, SubExpr,
};
//use crate::classes::Pair;
//use crate::classes::Subs::Sub;
//use crate::brzozowski;
use disjoint_sets::UnionFind;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/*
    Some placeholder functions and types

    NB: There's a bit of shady design going on with converting GenRegex to Strings and back,
    and using things like &key[4..key.len() - 1] to extract the name of a StringVar.
    These are not super extensible, we should instead use custom types for things like charvars and
    stringvars that will support these operations more directly.

    The below will help transition in that direction by wrapping the GenRegex (when used as a string)
    in a type GenRegexId which supports the necessary operations.
*/

// Use String for IDs
//#[derive(Clone, Debug, Eq, Hash, PartialEq)]
//struct GenRegexId(String);

// Function used to inset GenRegexes into a HashMap.
// TBD: This should probably be replaced by using Hash directly on
// GenRegex, which should be more efficient.
//fn gre_id(gre: &Rc<GenRegex>) -> GenRegexId {
// Use ToString for now
//GenRegexId(gre)
//}

//impl GenRegexId {
fn is_char_var(gre: &Rc<GenRegex>) -> bool {
    if let GenRegex::CharExpression(c_expr) = gre.as_ref() {
        match c_expr {
            CharExpression::Literal(_) => false,
            CharExpression::CharVar(_) => true,
        }
    } else {
        false
    }
    //self.0.starts_with("char(") && self.0.ends_with(")")
}

fn get_char_var(gre: &Rc<GenRegex>) -> Option<CharExpression> {
    if is_char_var(gre) {
        if let GenRegex::CharExpression(c_expr) = gre.as_ref() {
            Some(c_expr.clone())
        } else {
            None
        }
        //let name = &self.0[5..self.0.len() - 1];
        //Some(CharExpression::CharVar(name.to_string()))
    } else {
        None
    }
}

/*fn into_gre_char_expr(gre: &Rc<GenRegex>) -> GenRegex {
    GenRegex::CharExpression(Rc::new(CharExpression::Literal(gre.0)))
}*/

fn is_string_var(gre: &Rc<GenRegex>) -> bool {
    matches!(gre.as_ref(), GenRegex::StringVar(_))
}

fn get_string_var(gre: &Rc<GenRegex>) -> Option<StringVar> {
    if is_string_var(gre) {
        if let GenRegex::StringVar(s_var) = gre.as_ref() {
            Some(s_var.clone())
        } else {
            None
        }
    } else {
        None
    }
}
//}

type GenRegexPairSet = BTreeSet<(Rc<GenRegex>, Rc<GenRegex>)>;
type GenRegexHashSet = HashSet<(Rc<GenRegex>, Rc<GenRegex>)>;
type GenRegexHashMap = HashMap<Rc<GenRegex>, Rc<GenRegex>>;
type GenRegexBoolHashMap = HashMap<Rc<GenRegex>, bool>;
type OuterSet = HashSet<(Rc<GenRegex>, GenRegexPairSet)>;

/*
    Main functions
*/

//TODO: DEbug

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
fn count_union_elems(substitutions: &Rc<AnySub>) -> usize {
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

fn merge(substitutions: Rc<AnySub>) -> MergeResult {
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
                                return MergeResult::Bottom;
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
                return MergeResult::Bottom;
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
            return MergeResult::Bottom;
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
    MergeResult::SimpleSub(combined_expr)
}

fn sub_difference(sub1: Rc<SimpleSub>, sub2: Rc<SimpleSub>) -> MergeResult {
    if let MergeResult::SimpleSub(result) =
        merge(Rc::new(sub1.as_ref().clone().union(sub2.as_ref().clone())))
    {
        let mut retsub = result.clone();
        for char_var in sub2.get_char_map().keys() {
            retsub.remove_char_map(char_var);
        }
        for (string_var, sub_expr1) in result.get_str_map() {
            if let Some(sub_expr2) = sub2.get_string_var(string_var) {
                retsub.remove_str_map(string_var);
                if let Some(mut sub) = sub_expr_match(sub_expr1, sub_expr2, string_var) {
                    retsub.get_char_map_mut().append(sub.get_char_map_mut());
                    retsub.get_str_map_mut().append(sub.get_str_map_mut());
                } else {
                    return MergeResult::Bottom;
                }
            }
        }
        MergeResult::SimpleSub(retsub)
    } else {
        MergeResult::Bottom
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
pub fn derivative(
    gre: &Rc<GenRegex>,
    deriv_char: &Rc<CharExpression>,
) -> HashSet<AntimirovDerivativeElement> {
    // println!("taking d({}, {})", gre, deriv_char);

    match gre.as_ref() {
        GenRegex::EmptySet => HashSet::new(),
        GenRegex::Sigma => HashSet::from([AntimirovDerivativeElement::new(
            GenRegex::epsilon(),
            SimpleSub::empty(),
        )]),
        GenRegex::Range(char1, char2) => {
            let mut result =
                AntimirovDerivativeElement::new(GenRegex::epsilon(), SimpleSub::empty());
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

            HashSet::from([result])
        }
        GenRegex::CharExpression(c_expr) => match (deriv_char.as_ref(), c_expr) {
            (CharExpression::Literal(deriv_lit), CharExpression::Literal(literal_value)) => {
                if deriv_lit == literal_value {
                    HashSet::from([AntimirovDerivativeElement::new(
                        GenRegex::epsilon(),
                        SimpleSub::empty(),
                    )])
                } else {
                    HashSet::new()
                }
            }
            (CharExpression::CharVar(d_var), CharExpression::Literal(lit_val)) => {
                let mut char_to = BTreeMap::new();
                char_to.insert(d_var.clone(), c_expr.clone());
                let subs = MergeResult::SimpleSub(SimpleSub::new(BTreeMap::new(), char_to));
                AntimirovDerivativeElement::set_from_merge(GenRegex::epsilon(), subs)
            }
            (_, CharExpression::CharVar(c_var)) => {
                let mut char_to = BTreeMap::new();
                char_to.insert(c_var.clone(), deriv_char.as_ref().clone());
                let subs = MergeResult::SimpleSub(SimpleSub::new(BTreeMap::new(), char_to));
                AntimirovDerivativeElement::set_from_merge(GenRegex::epsilon(), subs)
            }
        },
        GenRegex::StringVar(string_var) => {
            let head = vec![deriv_char.as_ref().clone()];

            let subexpr = SubExpr::new(head, true);

            let mut string_to = BTreeMap::new();
            string_to.insert(string_var.clone(), subexpr);

            let substitution = SimpleSub::new(string_to, BTreeMap::new());

            HashSet::from([AntimirovDerivativeElement::new(gre.clone(), substitution)])
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
            for p_sub in p_deriv {
                for q_sub in &q_deriv {
                    if let Some(ret) = merge_derivs_intersect(&p_sub, q_sub) {
                        ret_set.insert(ret);
                    }
                }
            }
            ret_set
        }
        GenRegex::Concatenation(left, right) => {
            let left_deriv = derivative(left, deriv_char);
            let mut ret_set = HashSet::new();
            for sub in left_deriv {
                if let Some(ret) = apply_derivs_concat(&sub, right) {
                    ret_set.insert(ret);
                }
            }

            let p_nullable = nullable(left);
            if !p_nullable.is_empty() {
                let right_deriv = derivative(right, deriv_char);
                for n_sub in p_nullable {
                    for q_sub in &right_deriv {
                        if let Some(ret) = merge_derivs_concat(&n_sub, q_sub) {
                            ret_set.insert(ret);
                        }
                    }
                }
            }

            ret_set
        }
        GenRegex::Kleene(expr) => {
            let p_deriv = derivative(expr, deriv_char);
            let mut ret_set = HashSet::new();
            for sub in p_deriv {
                let s_sub = sub.get_subs();
                let ret = AntimirovDerivativeElement::new(
                    Rc::new(GenRegex::Concatenation(
                        sub.get_expr().clone(),
                        sub_in(gre, s_sub),
                    )),
                    s_sub.clone(),
                );
                ret_set.insert(ret);
            }
            ret_set
        }
        GenRegex::StringSlice(_, _) => {
            unimplemented!();
        }
        GenRegex::StringIndex(_) => {
            unimplemented!();
        }
        GenRegex::Complement(_) => {
            unimplemented!();
        }
        GenRegex::IfThenElse(_, _, _) => {
            unimplemented!();
        }
    }
}

fn merge_derivs_intersect(
    p: &AntimirovDerivativeElement,
    q: &AntimirovDerivativeElement,
) -> Option<AntimirovDerivativeElement> {
    let left_elem = p.get_subs();
    let right_elem = q.get_subs();
    let union_lr: AnySub = left_elem.clone().union(right_elem.clone());
    let ret = merge(Rc::new(union_lr));
    if let Some(sub) = ret.into_sub() {
        // a bit odd we don't use ret here - I think it's correct though?
        // TODO: fix sub_diff
        if let Some(l_minus_r) =
            sub_difference(Rc::new(left_elem.clone()), Rc::new(right_elem.clone())).into_sub()
        {
            if let Some(r_minus_l) =
                sub_difference(Rc::new(right_elem.clone()), Rc::new(left_elem.clone())).into_sub()
            {
                let p_prime_sub = sub_in(p.get_expr(), &r_minus_l);
                let q_prime_sub = sub_in(q.get_expr(), &l_minus_r);
                let final_expr = Rc::new(GenRegex::Intersect(p_prime_sub, q_prime_sub));
                return Some(AntimirovDerivativeElement::new(final_expr, sub));
            }
        }
    }
    None
}

fn apply_derivs_concat(
    left_deriv: &AntimirovDerivativeElement,
    right: &Rc<GenRegex>,
) -> Option<AntimirovDerivativeElement> {
    let simple_sub = left_deriv.get_subs();
    if let GenRegex::CharExpression(c_expr) = left_deriv.get_expr().as_ref() {
        if let CharExpression::Literal(lit) = c_expr {
            // if let GenRegex::CharExpression(CharExpression::Literal(lit)) = sub.0.as_ref() {
            Some(AntimirovDerivativeElement::new(
                Rc::new(GenRegex::Concatenation(
                    left_deriv.get_expr().clone(),
                    sub_in(right, simple_sub),
                )),
                left_deriv.get_subs().clone(),
            ))
        } else {
            None
        }
    } else {
        Some(AntimirovDerivativeElement::new(
            Rc::new(GenRegex::Concatenation(
                left_deriv.get_expr().clone(),
                sub_in(right, simple_sub),
            )),
            left_deriv.get_subs().clone(),
        ))
    }
}

fn merge_derivs_concat(
    n_sub: &SimpleSub,
    right: &AntimirovDerivativeElement,
) -> Option<AntimirovDerivativeElement> {
    let right_elem = right.get_subs();
    let union_lr: AnySub = n_sub.clone().union(right_elem.clone());
    let ret = merge(Rc::new(union_lr));
    if let Some(sub) = ret.into_sub() {
        if let Some(r_minus_l) =
            sub_difference(Rc::new(n_sub.clone()), Rc::new(right_elem.clone())).into_sub()
        {
            let q_prime_sub = sub_in(right.get_expr(), &r_minus_l);
            return Some(AntimirovDerivativeElement::new(q_prime_sub, sub));
        }
    }
    None
}

fn sub_in(expr: &Rc<GenRegex>, substitution: &SimpleSub) -> Rc<GenRegex> {
    if substitution.get_str_map().is_empty() && substitution.get_char_map().is_empty() {
        return expr.clone(); // Returns a clone of expr.
    }
    match expr.as_ref() {
        GenRegex::EmptySet => Rc::clone(expr),
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
        GenRegex::StringSlice(string_var, _) => todo!(),
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
        GenRegex::IfThenElse(predicate, gen_regex1, gen_regex2) => todo!(),
    }
}

pub fn satisfiable(expr: &Rc<GenRegex>) -> bool {
    let mut ind = 0;
    satisfiable_helper(expr, &mut ind, HashSet::new())
}
pub fn satisfiable_helper(
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

pub fn nullable(gre: &Rc<GenRegex>) -> HashSet<SimpleSub> {
    match gre.as_ref() {
        GenRegex::EmptySet => HashSet::new(),
        GenRegex::Sigma => HashSet::new(),
        GenRegex::Range(_, _) => HashSet::new(),
        GenRegex::CharExpression(c_expr) => match c_expr {
            CharExpression::CharVar(_) => HashSet::new(),
            CharExpression::Literal(_) => HashSet::new(),
        },
        GenRegex::StringVar(s_var) => {
            let mut subs = HashSet::new();
            let mut string_to = BTreeMap::new();
            string_to.insert(s_var.clone(), SubExpr::empty());
            let string_sub = SimpleSub::new(string_to, BTreeMap::new());
            subs.insert(string_sub);
            subs
        }
        GenRegex::Union(side1, side2) => {
            let left_null = nullable(&Rc::clone(side1));
            let right_null = nullable(&Rc::clone(side2));
            let union_lr: HashSet<_> = left_null.union(&right_null).cloned().collect();
            union_lr
        }
        GenRegex::Intersect(side1, side2) => {
            let left_null = nullable(&Rc::clone(side1));
            let right_null = nullable(&Rc::clone(side2));
            let mut ret_set = HashSet::new();
            for left_elem in &left_null {
                for right_elem in &right_null {
                    let union_lr: AnySub = left_elem.clone().union(right_elem.clone());
                    let ret = merge(Rc::new(union_lr));
                    match ret {
                        MergeResult::SimpleSub(simple_sub) => {
                            ret_set.insert(simple_sub);
                        }
                        _ => {}
                    }
                }
            }
            ret_set
        }
        GenRegex::Concatenation(side1, side2) => {
            let left_null = nullable(&Rc::clone(side1));
            let right_null = nullable(&Rc::clone(side2));
            let mut ret_set = HashSet::new();
            for left_elem in &left_null {
                for right_elem in &right_null {
                    let union_lr: AnySub = left_elem.clone().union(right_elem.clone());
                    let ret = merge(Rc::new(union_lr));
                    match ret {
                        MergeResult::SimpleSub(simple_sub) => {
                            ret_set.insert(simple_sub);
                        }
                        _ => {}
                    }
                }
            }
            ret_set
        }
        GenRegex::Kleene(_) => {
            let mut ret = HashSet::new();
            ret.insert(SimpleSub::empty());
            ret
        }
        GenRegex::StringSlice(_, _) => {
            unimplemented!();
        }
        GenRegex::StringIndex(_) => {
            unimplemented!();
        }
        GenRegex::Complement(_) => {
            unimplemented!();
        }
        GenRegex::IfThenElse(_, _, _) => {
            unimplemented!();
        }
    }
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
