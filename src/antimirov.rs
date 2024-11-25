//!
//! Implementation of the Antimirov Derivative
//!

use crate::classes::{CharExpression, GenRegex, StringVar, Predicate, MaybeCharExpression};
use disjoint_sets::UnionFind;
use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::brzozowski;

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
        if let GenRegex::CharExpression(c_expr) = gre.as_ref(){
            match c_expr.as_ref() {
                CharExpression::Literal(_) =>{
                    false
                }
                CharExpression::CharVar(_) =>{
                    true
                }
            }
        }else{
            false
        }
        //self.0.starts_with("char(") && self.0.ends_with(")")
    }

    fn get_char_var(gre: &Rc<GenRegex>) -> Option<CharExpression> {
        if is_char_var(gre) {
            if let GenRegex::CharExpression(c_expr) = gre.as_ref(){
                Some(c_expr.as_ref().clone())
            }else{
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
        if let GenRegex::StringVar(_) = gre.as_ref(){
            true
        }else{
            false
        }
    }

    fn get_string_var(gre: &Rc<GenRegex>) -> Option<StringVar> {
        if is_string_var(gre) {
            if let GenRegex::StringVar(s_var) = gre.as_ref(){
                Some(s_var.as_ref().clone())
            }else{
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
fn parse_string_vars(
    string_set: &GenRegexHashSet,
    union_find: &mut UnionFind<usize>,
    id_map: &HashMap<GenRegex, i32>,
    canonical_map: &mut HashMap<i32, i32>,
) -> (GenRegexHashMap, GenRegexBoolHashMap) {
    let mut string_dict = HashMap::new();
    let mut id_dict = HashMap::new();
    let mut return_vals = HashMap::new();
    let mut truncate = HashMap::new();
    let mut to_remove = Vec::new(); // Collect indices of elements to remove

    for elem in string_set {
        string_dict
            .entry(&elem.0)
            .or_insert(Vec::new())
            .push(elem.1.clone());
    }
    for elem in string_set {
        id_dict.insert(&elem.0, elem.0.clone());
    }
    for (key, mut value) in string_dict {
        let mut value_copy = value.clone();
        let mut last_pop = None;
        while value.len() > 1 {
            let mut union_elems = HashSet::new();
            // let mut new_element_to_add: std::option::Option<Rc<GenRegex>> = None;
            // let mut literal_val: std::option::Option<String> = None;
            let mut i = 0;
            while i < value.len() {
                let temp = value[i].clone(); // Clone the value to avoid borrowing issues
                let expr = temp.clone(); // Clone again for further processing

                match expr.as_ref() {
                    GenRegex::Concatenation(left, right) => {
                        union_elems.insert(left.clone()); // Insert a cloned value to avoid borrowing
                        value[i] = right.clone(); // Safe mutation after using immutable reference
                    }
                    GenRegex::CharExpression(c_expr) => match c_expr.as_ref() {
                        CharExpression::Literal(val) => {
                            if val.is_empty() {
                                // If the literal is empty, remove it and keep track of the popped value
                                value.remove(i);
                                to_remove.push(i);
                                last_pop = Some(value_copy.remove(i));
                                union_elems.insert(expr.clone()); // Insert cloned value
                                truncate.insert(id_dict[&key].clone(), true);
                                continue;
                            } else {
                                union_elems.insert(expr.clone()); // Insert cloned value
                            }
                        }
                        _ => {
                            union_elems.insert(expr.clone()); // Insert cloned value for non-literal cases
                        }
                    },
                    _ => {
                        if let GenRegex::StringVar(_) = expr.as_ref() {
                            // Handle StringVar case and remove element
                            value.remove(i);
                            to_remove.push(i);
                            last_pop = Some(value_copy.remove(i));
                        }
                        union_elems.insert(expr.clone()); // Insert cloned value
                    }
                }

                i += 1; // Increment index after processing an element
            }
            let mut prev: std::option::Option<Rc<GenRegex>> = None;
            for elem in union_elems {
                if let GenRegex::CharExpression(c_expr_temp) = elem.as_ref() {
                    if let CharExpression::Literal(val) = c_expr_temp.as_ref() {
                        if val.is_empty() {
                            truncate.insert(id_dict[&key].clone(), false);
                            return (HashMap::new(), truncate);
                        }
                    }
                }
                if let Some(p) = prev {
                    if let Some(leftId) = id_map.get(&p) {
                        if let Some(rightId) = id_map.get(&elem) {
                            let rightId = *rightId;
                            let leftId = *leftId; // Dereference if you need the inner value directly
                            if let (Some(value1), Some(value2)) =
                                (canonical_map.get(&leftId), canonical_map.get(&rightId))
                            {
                                if value1 != value2 {
                                    truncate.insert(id_dict[&key].clone(), false);
                                    return (HashMap::new(), truncate);
                                } else {
                                    union_find.union(leftId as usize, rightId as usize);
                                }
                            } else if let Some(value1) = canonical_map.get(&leftId) {
                                union_find.union(leftId as usize, rightId as usize);
                                let new_canon = union_find.find(leftId as usize);
                                canonical_map.insert(new_canon as i32, *value1);
                            } else if let Some(value1) = canonical_map.get(&rightId) {
                                union_find.union(leftId as usize, rightId as usize);
                                let new_canon = union_find.find(leftId as usize);
                                canonical_map.insert(new_canon as i32, *value1);
                            } else {
                                union_find.union(leftId as usize, rightId as usize);
                            }
                        }
                    }
                }
                prev = Some(elem.clone());
            }
        }
        if !value_copy.is_empty() {
            let string_var = Rc::new(get_string_var(key).expect("expected string var"));
            let str_var = Rc::new(GenRegex::StringVar(string_var));
            return_vals.insert(str_var, value_copy[0].clone());
        } else {
            let string_var = Rc::new(get_string_var(key).expect("expected string var"));
            let str_var = Rc::new(GenRegex::StringVar(string_var));
            return_vals.insert(str_var, last_pop.expect("last_pop expected nonempty"));
        }
    }
    (return_vals, truncate)
}

fn merge(substitutions: GenRegexPairSet) -> GenRegexPairSet {
    if substitutions.is_empty() {
        let t_gre = Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
            "".to_string(),
        ))));
        let mut ret_set = BTreeSet::new();
        ret_set.insert((t_gre.clone(), t_gre.clone()));
        return ret_set;
    }
    let mut id_map: HashMap<GenRegex, i32> = HashMap::new();
    let mut string_map: HashMap<i32, GenRegex> = HashMap::new();
    let mut canonical_map: HashMap<i32, i32> = HashMap::new();
    let mut next_id = 1;
    assign_unique_ids(substitutions.clone(), &mut id_map, &mut next_id);
    for (expr, id) in &id_map {
        string_map.insert(*id, expr.clone());
    }
    let mut union_find: UnionFind<usize> = UnionFind::new((next_id) as usize);
    let mut string_set = HashSet::new();
    let mut char_set = HashSet::new();
    let mut final_subs: HashMap<Rc<GenRegex>, Rc<GenRegex>> = HashMap::new();
    for sub in &substitutions {
        match &sub.0.as_ref() {
            GenRegex::StringVar(_) => {
                string_set.insert(sub.clone());
            }
            _ => {
                char_set.insert(sub.clone());
            }
        }
    }
    let (mut return_vals, truncate) =
        parse_string_vars(&string_set, &mut union_find, &id_map, &mut canonical_map);

    if truncate.len() > return_vals.len() {
        return BTreeSet::new(); // Return an empty set if there’s an error
    }
    for sub in &char_set {
        let index_str_1 = &sub.0;
        let index_str_2 = &sub.1;
        let leftId = id_map[index_str_1];
        let rightId = id_map[index_str_2];
        if let (Some(value1), Some(value2)) =
            (canonical_map.get(&leftId), canonical_map.get(&rightId))
        {
            if value1 != value2 {
                return BTreeSet::new();
            } else {
                union_find.union(leftId as usize, rightId as usize);
            }
        } else if let Some(value1) = canonical_map.get(&leftId) {
            union_find.union(leftId as usize, rightId as usize);
            let new_canon = union_find.find(leftId as usize);
            canonical_map.insert(new_canon as i32, *value1);
        } else if let Some(value1) = canonical_map.get(&rightId) {
            union_find.union(leftId as usize, rightId as usize);
            let new_canon = union_find.find(leftId as usize);
            canonical_map.insert(new_canon as i32, *value1);
        } else {
            union_find.union(leftId as usize, rightId as usize);
        }
    }
    for (key, value) in return_vals.iter_mut() {
        let mut temp = value.clone();
        //let mut temp_l;
        while let GenRegex::Concatenation(left, right) = temp.clone().as_ref() {
            let mut temp_l = left.clone();
            if let GenRegex::CharExpression(left_char) = left.as_ref() {
                if let CharExpression::CharVar(_) = left_char.as_ref() {
                    if let Some(val_1) = canonical_map.get(&id_map[left]) {
                        temp_l = Rc::new(string_map[val_1].clone());
                        /*temp = Rc::new(GenRegex::Concatenation(
                            Rc::new(GenRegex::CharExpression(temp_left_char)),
                            right.clone(),
                        ));*/
                    }
                }
            }

            if let GenRegex::StringVar(_) = right.as_ref() {
                if truncate.get(key).copied().unwrap_or(false) {
                    //let GenRegex::Concatenation(left_n, right_n) = temp.clone().as_ref();
                    temp = temp_l.clone()
                    // temp.right = Literal("");
                } else {
                    temp = right.clone();
                }
            } else {
                temp = right.clone();
            }
        }

        let final_key = key.clone(); // Assuming StringVar takes a String
        final_subs.insert(final_key, value.clone()); // Assuming final_subs is a HashMap
    }
    for key in id_map.keys() {
        if let Some(c_expr) = get_char_var(&Rc::new(key.clone())) {
            let c_obj = Rc::new(c_expr);

            // Try to get the value from canonical_map
            if let Some(value) = canonical_map.get(&(union_find.find(id_map[key] as usize) as i32))
            {
                // Insert CharExpression mapped to a literal value
                final_subs.insert(
                    Rc::new(GenRegex::CharExpression(c_obj.clone())),
                    Rc::new(string_map[value].clone()),
                );
            } else {
                // Attempt to get the map value from id_map (safe lookup)
                if let Some(id) = id_map.get(key) {
                    let map_val = union_find.find(*id as usize);

                    // Get the corresponding string from string_map
                    if let Some(map_str) = string_map.get(&(map_val as i32)) {
                        if let Some(c_expr) = get_char_var(&Rc::new(map_str.clone())) {
                            let c_obj_map = Rc::new(c_expr);

                            // Insert the generated CharExpression object
                            final_subs.insert(
                                Rc::new(GenRegex::CharExpression(c_obj.clone())),
                                Rc::new(GenRegex::CharExpression(c_obj_map)),
                            );
                        }
                    }
                }
            }
        }
    }
    let mut final_set = BTreeSet::new();
    for item in final_subs.iter() {
        final_set.insert((item.0.clone(), item.1.clone())); // Insert items directly without collect()
    }
    final_set

    //
}

pub fn derivative(gre: &Rc<GenRegex>, deriv_char: &Rc<CharExpression>) -> OuterSet {
    let empty_string = || {
        Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
            String::new(),
        ))))
    };
    match gre.as_ref() {
        GenRegex::EmptySet => HashSet::new(),
        GenRegex::CharExpression(c_expr) => match (deriv_char.as_ref(), c_expr.as_ref()) {
            (CharExpression::Literal(deriv_lit), CharExpression::Literal(literal_value)) => {
                if deriv_lit == literal_value {
                    let mut ret = HashSet::new();
                    ret.insert((empty_string(), BTreeSet::new()));
                    ret
                } else {
                    HashSet::new()
                }
            }
            (CharExpression::CharVar(_), CharExpression::Literal(_)) => {
                let mut ret = HashSet::new();
                let mut subs = BTreeSet::new();
                subs.insert((
                    Rc::new(GenRegex::CharExpression(deriv_char.clone())),
                    gre.clone(),
                ));
                ret.insert((empty_string(), subs));
                ret
            }
            (CharExpression::Literal(_), CharExpression::CharVar(_)) => {
                let mut ret = HashSet::new();
                let mut subs = BTreeSet::new();
                subs.insert((
                    gre.clone(),
                    Rc::new(GenRegex::CharExpression(deriv_char.clone())),
                ));
                ret.insert((empty_string(), subs));
                ret
            }
            (CharExpression::CharVar(_), CharExpression::CharVar(_)) => {
                let mut ret = HashSet::new();
                let mut subs = BTreeSet::new();
                subs.insert((
                    gre.clone(),
                    Rc::new(GenRegex::CharExpression(deriv_char.clone())),
                ));
                ret.insert((empty_string(), subs));
                ret
            }
        },
        GenRegex::StringVar(_) => {
            let mut ret = HashSet::new();
            let mut subs = BTreeSet::new();
            let concat = &Rc::new(GenRegex::Concatenation(
                Rc::new(GenRegex::CharExpression(deriv_char.clone())),
                Rc::clone(gre),
            ));
            subs.insert((Rc::clone(gre), concat.clone()));
            ret.insert((Rc::clone(gre), subs));
            ret
        }
        GenRegex::Union(side1, side2) => {
            let side1_deriv = derivative(side1, deriv_char);
            let side2_deriv = derivative(side2, deriv_char);
            side1_deriv.union(&side2_deriv).cloned().collect()
        }
        GenRegex::Intersect(left, right) => {
            let p_deriv = derivative(left, deriv_char);
            let q_deriv = derivative(right, deriv_char);
            let mut term1 = HashSet::new();

            for p_sub in p_deriv {
                for q_sub in &q_deriv {
                    let merged = merge(p_sub.1.union(&q_sub.1).cloned().collect::<BTreeSet<_>>());
                    if merged.is_empty() {
                        continue;
                    }
                    let p_new = sub_in(left, &merged);
                    let q_new = sub_in(right, &merged);
                    let p_new_deriv = derivative(&p_new, deriv_char);
                    let q_new_deriv = derivative(&q_new, deriv_char);
                    for p_sub_new in &p_new_deriv {
                        for q_sub_new in &q_new_deriv {
                            let curr = (
                                Rc::new(GenRegex::Intersect(
                                    p_sub_new.0.clone(),
                                    q_sub_new.0.clone(),
                                )),
                                merged.clone(),
                            );
                            term1.insert(curr);
                        }
                    }

                    //if curr.1.len() == 0 {
                    //   continue;
                    //}
                }
            }

            term1
        }
        GenRegex::Concatenation(left, right) => {
            let left_deriv = derivative(left, deriv_char);
            //let right_deriv = derivative(right, deriv_char);

            // Create term1 set
            let mut term1 = HashSet::new();
            if !left_deriv.is_empty() {
                for sub in left_deriv {
                    if let GenRegex::CharExpression(c_expr) = sub.0.as_ref() {
                        if let CharExpression::Literal(lit) = c_expr.as_ref() {
                            //                    if let GenRegex::CharExpression(CharExpression::Literal(lit)) = sub.0.as_ref() {
                            if lit.is_empty() {
                                let curr = (sub_in(right, &sub.1.clone()), sub.1.clone());
                                term1.insert(curr);
                            } else {
                                let curr = (
                                    Rc::new(GenRegex::Concatenation(
                                        sub.0.clone(),
                                        sub_in(right, &sub.1.clone()),
                                    )),
                                    sub.1.clone(),
                                );
                                term1.insert(curr);
                            }
                        }
                    } else {
                        let curr = (
                            Rc::new(GenRegex::Concatenation(
                                sub.0.clone(),
                                sub_in(right, &sub.1.clone()),
                            )),
                            sub.1.clone(),
                        );
                        term1.insert(curr);
                    }
                }
            }

            let p_nullable = nullable(left);
            if !p_nullable.is_empty() {
                for sub in p_nullable {
                    let temp = sub_in(right, &sub);
                    let deriv = derivative(&temp, deriv_char);
                    let mut derivatives = HashSet::new();
                    if deriv.is_empty() {
                        continue;
                    }
                    for elem in deriv {
                        let elem_term = elem.0;
                        let elem_subs = elem.1;
                        let elem_subs_final =
                            merge(elem_subs.union(&sub).cloned().collect::<BTreeSet<_>>());

                        if elem_subs_final.is_empty() {
                            continue;
                        }
                        derivatives.insert((elem_term, elem_subs_final));
                    }
                    term1 = term1.union(&derivatives).cloned().collect();
                }
            }

            term1
        }
        GenRegex::Kleene(expr) => {
            let p_deriv = derivative(expr, deriv_char);
            let mut term1 = HashSet::new();

            for sub in p_deriv {
                let curr = (
                    Rc::new(GenRegex::Concatenation(
                        sub.0.clone(),
                        sub_in(gre, &sub.1.clone()),
                    )),
                    sub.1.clone(),
                );
                term1.insert(curr);
            }

            term1
        }
        GenRegex::Complement(_) =>{
            let term =  brzozowski::derivative(gre, deriv_char);
            let subs = BTreeSet::new();
            let mut ret = HashSet::new();
            ret.insert((term, subs));
            ret
            
        }
        GenRegex::StringIndex(_) =>{
            let term =  brzozowski::derivative(gre, deriv_char);
            let subs = BTreeSet::new();
            let mut ret = HashSet::new();
            ret.insert((term, subs));
            ret
            
        }
        GenRegex::StringSlice(_, _) =>{
            let term =  brzozowski::derivative(gre, deriv_char);
            let  subs = BTreeSet::new();
            let mut ret = HashSet::new();
            ret.insert((term, subs));
            ret
            
        }
        GenRegex::IfThenElse(pred, p, q) => {
            let deriv_p = derivative(p, deriv_char);
            let deriv_q = derivative(q, deriv_char);
            let mut ret = HashSet::new();
            for elem in deriv_p{
                let term = (Rc::new(GenRegex::IfThenElse(pred.clone(), elem.0, Rc::new(GenRegex::EmptySet))), elem.1);
                ret.insert(term);
            }
            for elem in deriv_q{
                let term = (Rc::new(GenRegex::IfThenElse(pred.clone(), Rc::new(GenRegex::EmptySet), elem.0)), elem.1);
                ret.insert(term);
            }
            ret

        }
    }
}

fn sub_in(expr: &Rc<GenRegex>, substitution: &GenRegexPairSet) -> Rc<GenRegex> {
    if substitution.is_empty() {
        return Rc::clone(expr); // Return a clone of the Rc, as Rc handles reference counting
    }

    // Create a HashMap for substitutions
    let mut subs: HashMap<GenRegex, &Rc<GenRegex>> = HashMap::new();

    // Populate the HashMap with substitutions
    for sub in substitution.iter() {
        let key = &sub.0; // Assuming this converts GenRegex to String
        subs.insert(key.as_ref().clone(), &sub.1); // Insert the key-value pair into the HashMap
    }
    sub_in_helper(expr, subs)
}

fn sub_in_helper(expr: &Rc<GenRegex>, sub: HashMap<GenRegex, &Rc<GenRegex>>) -> Rc<GenRegex> {
    match expr.as_ref() {
        GenRegex::StringVar(_) => {
            let key = expr;
            match sub.get(key) {
                Some(value) => Rc::clone(value),
                None => Rc::clone(expr),
            }
        }
        GenRegex::CharExpression(c_expr) => match c_expr.as_ref() {
            CharExpression::CharVar(_) => {
                let key = expr;
                match sub.get(key) {
                    Some(value) => Rc::clone(value),
                    None => Rc::clone(expr),
                }
            }
            CharExpression::Literal(_) => Rc::clone(expr),
        },
        GenRegex::Intersect(left, right) => {
            let leftSub = sub_in_helper(left, sub.clone());
            let rightSub = sub_in_helper(right, sub);
            Rc::new(GenRegex::Intersect(leftSub, rightSub))
        }
        GenRegex::Concatenation(left, right) => {
            let leftSub = sub_in_helper(left, sub.clone());
            let rightSub = sub_in_helper(right, sub);
            Rc::new(GenRegex::Concatenation(leftSub, rightSub))
        }
        GenRegex::Union(left, right) => {
            let leftSub = sub_in_helper(left, sub.clone());
            let rightSub = sub_in_helper(right, sub);
            Rc::new(GenRegex::Union(leftSub, rightSub))
        }
        GenRegex::Kleene(inner) => {
            let innerSub = sub_in_helper(inner, sub);
            Rc::new(GenRegex::Kleene(innerSub))
        }
        _ => Rc::clone(expr),
    }
}

pub fn satisfiable(expr: &Rc<GenRegex>, mut index: i32, mut visited: BTreeSet<GenRegex>) -> bool{
    if visited.contains(expr){
        return false;
    }else{
        visited.insert(expr.as_ref().clone());
    }
    if nullable(expr).is_empty(){
        let new_name = "f".to_owned() + &index.to_string();
        let c_var = Rc::new(CharExpression::CharVar(new_name));
        let deriv = derivative(expr, &c_var);
        if deriv.is_empty(){
            return false
        }
        index += 1;
        for elem in deriv{
            if satisfiable(&elem.0, index, visited.clone()){
                return true;
            }
        }
        return false;
    }
    true
}
pub fn matching(expr: &Rc<GenRegex>, proposed: String) -> bool {
    if proposed.is_empty() {
        return !nullable(expr).is_empty();
    }
    let literal = Rc::new(CharExpression::Literal(String::from(&proposed[0..1])));
    let deriv = derivative(expr, &literal);
    if deriv.is_empty() {
        return false;
    }
    for elem in deriv {
        if matching(&elem.0, String::from(&proposed[1..])) {
            return true;
        }
    }
    false
}

pub fn nullable(gre: &Rc<GenRegex>) -> BTreeSet<GenRegexPairSet> {
    let empty_string = || {
        Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
            String::new(),
        ))))
    };
    match gre.as_ref() {
        GenRegex::EmptySet | GenRegex::StringIndex(..) => BTreeSet::new(),
        GenRegex::CharExpression(cExpr) => match cExpr.as_ref() {
            CharExpression::CharVar(_name) => BTreeSet::new(),
            CharExpression::Literal(value) => {
                if value.is_empty() {
                    let mut ret = BTreeSet::new();
                    ret.insert(BTreeSet::new());
                    ret
                } else {
                    BTreeSet::new()
                }
            }
        },
        GenRegex::StringVar(_) => {
            let mut subs = BTreeSet::new();
            subs.insert((gre.clone(), empty_string()));
            let mut ret = BTreeSet::new();
            ret.insert(subs);
            ret
        }
        GenRegex::Union(side1, side2) => {
            let left_null = nullable(&Rc::clone(side1));
            let right_null = nullable(&Rc::clone(side2));
            let unionLR: BTreeSet<_> = left_null.union(&right_null).cloned().collect();
            unionLR
        }
        GenRegex::Intersect(side1, side2) => {
            let left_null = nullable(&Rc::clone(side1));
            let right_null = nullable(&Rc::clone(side2));
            let mut retSet = BTreeSet::new();
            for left_elem in &left_null {
                for right_elem in &right_null {
                    let unionLR: BTreeSet<_> = left_elem.union(right_elem).cloned().collect();
                    let ret = merge(unionLR.clone());
                    if !ret.is_empty() {
                        retSet.insert(ret);
                    }
                }
            }
            retSet
        }
        GenRegex::Concatenation(side1, side2) => {
            let left_null = nullable(&Rc::clone(side1));
            let right_null = nullable(&Rc::clone(side2));
            let mut retSet = BTreeSet::new();
            for left_elem in &left_null {
                for right_elem in &right_null {
                    let unionLR: BTreeSet<_> = left_elem.union(right_elem).cloned().collect();
                    let ret = merge(unionLR.clone());
                    if !ret.is_empty() {
                        retSet.insert(ret);
                    }
                }
            }
            retSet
        }
        GenRegex::Kleene(_) => {
            let mut ret = BTreeSet::new();
            ret.insert(BTreeSet::new());
            ret
        }
        GenRegex::StringSlice(string_var, index)=>{
                let alphabet = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
                let length = *index as usize;
                let mut results = BTreeSet::new();
                generate_combinations(&alphabet, length, Vec::new(), &mut results, string_var, None, false);
                results
        }
        _ => BTreeSet::new(),
    }
}

fn s_from_p(pred: &Rc<Predicate>) -> BTreeSet<GenRegexPairSet>{
    match pred.as_ref() {
        Predicate::True =>{
            let mut ret = BTreeSet::new();
            ret.insert(BTreeSet::new());
            ret

        }
        Predicate::False =>{
            BTreeSet::new()
        }
        Predicate::EqualLength(s_var, index)=>{
                let alphabet = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
                let length = *index as usize;
                let mut results = BTreeSet::new();
                generate_combinations(&alphabet, length, Vec::new(), &mut results, s_var, None, false);
                results

        }
        Predicate::Equals(maybe_1, maybe_2)=> match (maybe_1.as_ref(), maybe_2.as_ref()){
            (MaybeCharExpression::CharExpression(c_expr_1), MaybeCharExpression::CharExpression(c_expr_2)) => match (c_expr_1.as_ref(), c_expr_2.as_ref()) {
                (CharExpression::Literal(val_1), CharExpression::Literal(val_2)) =>{
                    if val_1 == val_2{
                        let mut ret = BTreeSet::new();
                        ret.insert(BTreeSet::new());
                        ret
                    }else{
                        BTreeSet::new()
                    }

                },
                (CharExpression::Literal(_), CharExpression::CharVar(_)) =>{
                    let mut sub = BTreeSet::new();
                    sub.insert((Rc::new(GenRegex::CharExpression(Rc::clone(c_expr_1))), Rc::new(GenRegex::CharExpression(Rc::clone(c_expr_2)))));
                    let mut ret = BTreeSet::new();
                    ret.insert(sub);
                    ret
                }
                _ =>{
                    let mut sub = BTreeSet::new();
                    sub.insert((Rc::new(GenRegex::CharExpression(Rc::clone(c_expr_1))), Rc::new(GenRegex::CharExpression(Rc::clone(c_expr_2)))));
                    //sub.insert((maybe_1, maybe_2));
                    let mut ret = BTreeSet::new();
                    ret.insert(sub);
                    ret
                }
            }
            (MaybeCharExpression::StringIndex(s_ind), MaybeCharExpression::CharExpression(c_expr)) =>{
                    let alphabet = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
                    let length = s_ind.index as usize;
                    let mut results = BTreeSet::new();
                    generate_combinations(&alphabet, length, Vec::new(), &mut results, &s_ind.var, Some(c_expr.as_ref().clone()), true);
                    results


            }
            (MaybeCharExpression::CharExpression(c_expr), MaybeCharExpression::StringIndex(s_ind)) =>{
                    let alphabet = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
                    let length = s_ind.index as usize;
                    let mut results = BTreeSet::new();
                    generate_combinations(&alphabet, length, Vec::new(), &mut results, &s_ind.var, Some(c_expr.as_ref().clone()), true);
                    results


            }
            (MaybeCharExpression::StringIndex(s_ind_1), MaybeCharExpression::StringIndex(s_ind_2)) =>{
                    let alphabet = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
                    let length = s_ind_1.index as usize;
                    let mut results_1 = BTreeSet::new();
                    let mut results_2 = BTreeSet::new();
                    let c_var = Rc::new(CharExpression::CharVar("f".to_string()));
                    generate_combinations(&alphabet, length, Vec::new(), &mut results_1, &s_ind_1.var, Some(c_var.as_ref().clone()), true);
                    generate_combinations(&alphabet, length, Vec::new(), &mut results_2, &s_ind_2.var, Some(c_var.as_ref().clone()), true);
                    let mut results = BTreeSet::new();
                    for elem in results_1{
                        for elem_2 in &results_2{
                            results.insert(elem.union(&elem_2).cloned().collect());
                        }
                    }
                    results
                
            }
        }
        Predicate::And(preds) => {
            let mut results = BTreeSet::new();
            union_substitutions(preds, 0, BTreeSet::new(), &mut results);
            let mut final_res = BTreeSet::new();
            for res in results{
                let ret_elem = merge(res.clone());
                if !ret_elem.is_empty() {
                    final_res.insert(ret_elem);
                }

            }
            final_res

        }
        Predicate::Or(preds) => {
            let mut results = BTreeSet::new();
            for p in preds{
                let temp = s_from_p(p);
                results = results.union(&temp).cloned().collect();
            }
            results

        }
        _ => {BTreeSet::new()}

    
}
}
fn union_substitutions(
    predicates: &Vec<Rc<Predicate>>, // Predicates each yielding a set of substitutions
    current_index: usize,  // Current predicate being processed
    current_union: GenRegexPairSet, // Current set of substitutions being formed
    results: &mut BTreeSet<GenRegexPairSet>, // Store the resulting substitutions (not sets)
) {
    // Base case: If we've processed all predicates, add the union to results
    if current_index == predicates.len() {
            results.insert(current_union);
        return;
    }

    // Recursive case: Process the sets for the current predicate
    let sets = s_from_p(&predicates[current_index]);

    // For each set of substitutions in the current predicate's result
    for set in sets {
        // Union the current set of substitutions with the ongoing union
        let mut new_union = current_union.clone();
        new_union = new_union.union(&set).cloned().collect();

        // Recur with the next predicate
        union_substitutions(predicates, current_index + 1, new_union, results);
    }
}

fn generate_combinations(
    alphabet: &[char],
    length: usize,
    current: Vec<char>,
    results: &mut BTreeSet<GenRegexPairSet>,
    string_var: &Rc<StringVar>,
    end_char: Option<CharExpression>,       // New parameter to specify a last character
    append_variable: bool          // New flag to append the variable at the end
) {
    // Base case: If the current length equals the target length (minus 1 if adding end_char).
    let target_length = if end_char.is_some() { length - 1 } else { length };

    if current.len() == target_length {
        let mut regex = build_concatenation(&current);

        // If there's an end_char, append it to the regex.
        if let Some(last_char) = end_char {
            let last_regex = GenRegex::CharExpression(Rc::new(last_char));
            regex = GenRegex::Concatenation(Rc::new(regex), Rc::new(last_regex));
        }

        // If append_variable is true, add `string_var` as the final concatenation element.
        if append_variable {
            let string_var_regex = GenRegex::StringVar(string_var.clone());
            regex = GenRegex::Concatenation(Rc::new(regex), Rc::new(string_var_regex));
        }

        // Add the generated pattern as a substitution pair.
        let mut new_sub = BTreeSet::new();
        new_sub.insert((Rc::new(GenRegex::StringVar(string_var.clone())), Rc::new(regex)));
        results.insert(new_sub);
        return;
    }

    // Recursively add each character from the alphabet to the current combination.
    for &ch in alphabet {
        let mut new_current = current.clone();
        new_current.push(ch);
        generate_combinations(alphabet, length, new_current, results, string_var, end_char.clone(), append_variable);
    }
}

fn build_concatenation(chars: &[char]) -> GenRegex {
    let mut iter = chars.iter();
    let first = iter.next().expect("Expected at least one character");

    // Start with the first character as a GenRegex::CharExpression
    let mut regex = GenRegex::CharExpression(Rc::new(CharExpression::Literal(first.to_string())));
    
    // Chain each subsequent character as a concatenation
    for &ch in iter {
        let next_regex = GenRegex::CharExpression(Rc::new(CharExpression::Literal(ch.to_string())));
        regex = GenRegex::Concatenation(Rc::new(regex), Rc::new(next_regex));
    }
    regex
}


fn assign_unique_ids(
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
}
