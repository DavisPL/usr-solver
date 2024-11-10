use crate::classes::{CharExpression, GenRegex, StringVar};
use crate::print::print_gre;
use disjoint_sets::UnionFind;
use std::collections::BTreeSet;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

type GenRegexPairSet = BTreeSet<(Rc<GenRegex>, Rc<GenRegex>)>;
type OuterSet = HashSet<(Rc<GenRegex>, GenRegexPairSet)>;

//TODO: DEbug
fn parse_string_vars(
    string_set: &HashSet<(Rc<GenRegex>, Rc<GenRegex>)>,
    union_find: &mut UnionFind<usize>,
    id_map: &HashMap<String, i32>,
    canonical_map: &mut HashMap<i32, i32>,
) -> (
    HashMap<Rc<GenRegex>, Rc<GenRegex>>,
    HashMap<Rc<GenRegex>, bool>,
) {
    let mut string_dict = HashMap::new();
    let mut id_dict = HashMap::new();
    let mut return_vals = HashMap::new();
    let mut truncate = HashMap::new();
    let mut to_remove = Vec::new(); // Collect indices of elements to remove

    for elem in string_set {
        string_dict
            .entry(print_gre(&elem.0))
            .or_insert(Vec::new())
            .push(elem.1.clone());
    }
    for elem in string_set {
        id_dict.insert(print_gre(&elem.0.clone()), elem.0.clone());
    }
    for (key, mut value) in string_dict {
        let mut value_copy = value.clone();
        let mut last_pop = None;
        while value.len() > 1 {
            let mut union_elems = HashSet::new();
            //let mut new_element_to_add: std::option::Option<Rc<GenRegex>> = None;
 //           let mut literal_val: std::option::Option<String> = None;
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
                    if let CharExpression::Literal(val) = c_expr_temp.as_ref(){
                        if val.is_empty(){
                            truncate.insert(id_dict[&key].clone(), false);
                            return (HashMap::new(), truncate);

                        }
                    }
                }
                if let Some(p) = prev {
                    if let Some(leftId) = id_map.get(&print_gre(&p)) {
                        if let Some(rightId) = id_map.get(&print_gre(&elem)) {
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
        if value_copy.len() != 0 {
            let string_var = Rc::new(StringVar {
                name: String::from(&key[4..key.len() - 1]),
            });
            let str_var = Rc::new(GenRegex::StringVar(string_var));
            println!("{}", value_copy[0].clone());
            return_vals.insert(str_var, value_copy[0].clone());
        } else {
            let string_var = Rc::new(StringVar {
                name: String::from(&key[4..key.len() - 1]),
            });
            let str_var = Rc::new(GenRegex::StringVar(string_var));
            return_vals.insert(str_var, last_pop.expect("reason"));

        }
    }
    return (return_vals, truncate);
}

fn merge(substitutions: GenRegexPairSet) -> GenRegexPairSet {
    if substitutions.len() == 0{
        let t_gre = Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal("".to_string()))));
        let mut ret_set = BTreeSet::new();
        ret_set.insert((t_gre.clone(), t_gre.clone()));
        return ret_set;
    }
    let mut id_map: HashMap<String, i32> = HashMap::new();
    let mut string_map: HashMap<i32, String> = HashMap::new();
    let mut canonical_map: HashMap<i32, i32> = HashMap::new();
    let mut next_id = 1;
    assign_unique_ids(substitutions.clone(), &mut id_map, &mut next_id);
    for (expr, id) in &id_map {
        string_map.insert(*id, expr.to_string());
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
        let index_str_1 = print_gre(&sub.0);
        let index_str_2 = print_gre(&sub.1);
        let leftId = id_map[&index_str_1];
        let rightId = id_map[&index_str_2];
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
        loop {
            // Handle Concatenation case
            if let GenRegex::Concatenation(left, right) = temp.clone().as_ref() {
                if let GenRegex::CharExpression(left_char) = left.as_ref() {
                    if let CharExpression::CharVar(left_char_1) = left_char.as_ref() {
                        if let Some(val_1) = canonical_map.get(&id_map[&print_gre(&left)]) {
                            let temp_left_char =
                                Rc::new(CharExpression::Literal(string_map[val_1].clone()));
                            temp = Rc::new(GenRegex::Concatenation(
                                Rc::new(GenRegex::CharExpression(temp_left_char)),
                                right.clone(),
                            ));
                        }
                    }
                }

                if let GenRegex::StringVar(_) = right.as_ref() {
                    if truncate.get(key).copied().unwrap_or(false) {
                        temp = left.clone();
                        // temp.right = Literal("");
                    } else {
                        temp = right.clone();
                    }
                } else {
                    temp = right.clone();
                }
            } else {
                // Handle CharVar case
                if let GenRegex::CharExpression(left_char) = temp.as_ref() {
                    if let CharExpression::CharVar(left_char_1) = left_char.as_ref() {
                        if let Some(val_1) = canonical_map.get(&id_map[&print_gre(&temp)]) {
                            temp = Rc::new(GenRegex::CharExpression(Rc::new(
                                CharExpression::Literal(string_map[val_1].clone()),
                            )));
                        }
                    }
                }

                //temp = None;
                break;
            }
        }

        let final_key = key.clone(); // Assuming StringVar takes a String
        final_subs.insert(final_key, value.clone()); // Assuming final_subs is a HashMap
    }
    for key in id_map.keys() {
        if key.starts_with("char(") && key.ends_with(")") {
            let name = Some(&key[5..key.len() - 1]);
            let c_obj = Rc::new(CharExpression::CharVar(
                name.expect("can't convert to char for some reason")
                    .to_string(),
            ));

            // Try to get the value from canonical_map
            if let Some(value) = canonical_map.get(&(union_find.find(id_map[key] as usize) as i32))
            {
                // Insert CharExpression mapped to a literal value
                final_subs.insert(
                    Rc::new(GenRegex::CharExpression(c_obj.clone())),
                    Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
                        string_map[value].clone(),
                    )))),
                );
            } else {
                // Attempt to get the map value from id_map (safe lookup)
                if let Some(id) = id_map.get(key) {
                    let map_val = union_find.find(*id as usize);

                    // Get the corresponding string from string_map
                    if let Some(map_str) = string_map.get(&(map_val as i32)) {
                        if map_str.starts_with("char(") && map_str.ends_with(")") {
                            let name_char = Some(&map_str[5..map_str.len() - 1]); // Extract the content inside "char(...)"
                            let c_obj_map = Rc::new(CharExpression::CharVar(
                                name_char.expect("can't convert").to_string(),
                            ));

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
    return final_set;

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
                    let subDiff = &p_sub.1.difference(&q_sub.1).cloned().collect::<BTreeSet<_>>();
                    for sub in subDiff{
                        println!("p-q {}", sub.1);
                    }
                    let subDiff2 = &q_sub.1.difference(&p_sub.1).cloned().collect::<BTreeSet<_>>();
                    for sub in subDiff2{
                        println!("q-p{}", sub.1);
                    }
                    let merged = merge(p_sub.1.union(&q_sub.1).cloned().collect::<BTreeSet<_>>());
                    if merged.is_empty(){
                        continue;
                    }
                    let p_new = sub_in(left, &merged);
                    let q_new = sub_in(right, &merged);
                    let p_new_deriv = derivative(&p_new, deriv_char);
                    let q_new_deriv = derivative(&q_new, deriv_char);
                    for p_sub_new in &p_new_deriv{
                        for q_sub_new in &q_new_deriv{
                            let curr = (
                                Rc::new(GenRegex::Intersect(
                                        p_sub_new.0.clone(),
                                        q_sub_new.0.clone()
                                )),
                                merged.clone()
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
                    println!("nullable check {}", temp);
                    let deriv = derivative(&temp, deriv_char);
                    let mut derivatives = HashSet::new();
                    if deriv.is_empty() {
                        continue;
                    }
                    for elem in deriv{
                        let elem_term = elem.0;
                        println!("{}", elem_term);
                        let elem_subs = elem.1;
                        let elem_subs_final = merge(elem_subs.union(&sub).cloned().collect::<BTreeSet<_>>());

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
        _ => HashSet::new(),
    }
}

fn sub_in(expr: &Rc<GenRegex>, substitution: &GenRegexPairSet) -> Rc<GenRegex> {
    if substitution.is_empty() {
        return Rc::clone(expr); // Return a clone of the Rc, as Rc handles reference counting
    }

    // Create a HashMap for substitutions
    let mut subs: HashMap<String, &Rc<GenRegex>> = HashMap::new();

    // Populate the HashMap with substitutions
    for sub in substitution.iter() {
        let key = print_gre(&sub.0); // Assuming this converts GenRegex to String
        subs.insert(key, &sub.1); // Insert the key-value pair into the HashMap
    }
    sub_in_helper(expr, subs)
}

fn sub_in_helper(expr: &Rc<GenRegex>, sub: HashMap<String, &Rc<GenRegex>>) -> Rc<GenRegex> {
    match expr.as_ref() {
        GenRegex::StringVar(_) => {
            let key = print_gre(expr);
            match sub.get(&key) {
                Some(value) => Rc::clone(value),
                None => Rc::clone(expr),
            }
        }
        GenRegex::CharExpression(c_expr) => match c_expr.as_ref() {
            CharExpression::CharVar(_) => {
                let key = print_gre(expr);
                match sub.get(&key) {
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

pub fn matching(expr: &Rc<GenRegex>, proposed: String) -> bool {
    println!("{}", expr);
    if proposed.is_empty() {
        return !nullable(expr).is_empty();
    }
    let literal = Rc::new(CharExpression::Literal(String::from(&proposed[0..1])));
    let deriv = derivative(expr, &literal);
    if deriv.is_empty() {
        return false;
    }
    for elem in &deriv{
        println!("deriv {} {}", elem.0, proposed);

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
        GenRegex::EmptySet => BTreeSet::new(),
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
            println!("{}", left_null.len());
            println!("{}", right_null.len());
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
        _ => BTreeSet::new(),
    }
}
fn assign_unique_ids(
    substitutions: GenRegexPairSet,
    id_map: &mut HashMap<String, i32>,
    next_id: &mut i32,
) {
    for sub in &substitutions {
        match sub.0.as_ref() {
            GenRegex::StringVar(_) => {
                let index_str = print_gre(&sub.0);
                id_map.entry(index_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
            GenRegex::CharExpression(c_expr) => match c_expr.as_ref() {
                CharExpression::CharVar(_) => {
                    let index_str = print_gre(&sub.0);
                    id_map.entry(index_str).or_insert_with(|| {
                        let id = *next_id;
                        *next_id += 1;
                        id
                    });
                }
                _ => {}
            },
            _ => {}
        }
        match sub.1.as_ref() {
            GenRegex::StringVar(_) => {
                let index_str = print_gre(&sub.1);
                id_map.entry(index_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
            GenRegex::CharExpression(c_expr) =>{
                let index_str = print_gre(&sub.1);
                id_map.entry(index_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            },
            _ => {}
        }
    }
}
