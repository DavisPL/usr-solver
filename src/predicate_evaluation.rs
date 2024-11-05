// Better to fix and remove, allowing for now
#![allow(non_snake_case)]

use crate::classes::{CharExpression, Predicate, StringVar};
use disjoint_sets::UnionFind;
// TODO: Unused imports
// Replace with Display trait
// use crate::print::{
//     print_char_expression, print_equals_arg, print_gre, print_predicate, print_string_var,
// };
use either::Either;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub fn flatten_and_predicates(pred: &Rc<Predicate>) -> Vec<Rc<Predicate>> {
    match pred.as_ref() {
        Predicate::And(children) => {
            let mut flattened: Vec<Rc<Predicate>> = Vec::new();
            for child in children {
                flattened.push(convertToDNF(&Rc::clone(child)))
            }
            flattened
        }
        _ => {
            vec![Rc::clone(pred)]
        }
    }
}

fn assign_unique_ids(predicate: &Predicate, id_map: &mut HashMap<String, i32>, next_id: &mut i32) {
    match predicate {
        Predicate::Equals(left, right) => match (left.as_ref(), right.as_ref()) {
            (Either::Right(str_ind_1), Either::Right(str_ind_2)) => {
                let mut index_str = format!(
                    "StringIndex(var: {}, index: {})",
                    str_ind_1.var.name, str_ind_1.index
                );
                id_map.entry(index_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
                index_str = format!(
                    "StringIndex(var: {}, index: {})",
                    str_ind_2.var.name, str_ind_2.index
                );
                id_map.entry(index_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
            (Either::Left(char_expr_1), Either::Right(str_ind_2)) => {
                let index_str = format!(
                    "StringIndex(var: {}, index: {})",
                    str_ind_2.var.name, str_ind_2.index
                );
                id_map.entry(index_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
                let expr_str = match char_expr_1.as_ref() {
                    CharExpression::CharVar(name) => format!("CharVar({})", name),
                    CharExpression::Literal(value) => format!("Literal({})", value),
                };
                id_map.entry(expr_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
            (Either::Right(str_ind_2), Either::Left(char_expr_1)) => {
                let index_str = format!(
                    "StringIndex(var: {}, index: {})",
                    str_ind_2.var.name, str_ind_2.index
                );
                id_map.entry(index_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
                let expr_str = match char_expr_1.as_ref() {
                    CharExpression::CharVar(name) => format!("CharVar({})", name),
                    CharExpression::Literal(value) => format!("Literal({})", value),
                };
                id_map.entry(expr_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
            (Either::Left(char_expr_1), Either::Left(char_expr_2)) => {
                let mut expr_str = match char_expr_1.as_ref() {
                    CharExpression::CharVar(name) => format!("CharVar({})", name),
                    CharExpression::Literal(value) => format!("Literal({})", value),
                };
                id_map.entry(expr_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
                expr_str = match char_expr_2.as_ref() {
                    CharExpression::CharVar(name) => format!("CharVar({})", name),
                    CharExpression::Literal(value) => format!("Literal({})", value),
                };
                id_map.entry(expr_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
        },
        Predicate::And(predicates) | Predicate::Or(predicates) => {
            // Recurse down for each sub-predicate in `And` or `Or` lists
            for sub_predicate in predicates {
                assign_unique_ids(sub_predicate, id_map, next_id);
            }
        }
        Predicate::Not(sub_predicate) => {
            // Recurse down for single `Not` sub-predicate
            assign_unique_ids(sub_predicate, id_map, next_id);
        }
        Predicate::EqualLength(var, _) => {
            let var_str = format!("StringVar({})", var.name);
            id_map.entry(var_str).or_insert_with(|| {
                let id = *next_id;
                *next_id += 1;
                id
            });
        }
        _ => {}
    }
}

pub fn evaluateComplete(pred: &Rc<Predicate>) -> Rc<Predicate> {
    let mut id_map: HashMap<String, i32> = HashMap::new();
    let mut string_map: HashMap<i32, String> = HashMap::new();
    let mut canonical_map: HashMap<i32, i32> = HashMap::new();
    let mut next_id = 1;
    assign_unique_ids(pred, &mut id_map, &mut next_id);
    for (expr, id) in &id_map {
        string_map.insert(*id, expr.to_string());
        //println!("{} {} {}", expr, id, id_map.len());
    }
    let mut uf: UnionFind<usize> = UnionFind::new((next_id) as usize);
    let predicate = convertToDNF(pred);
    //let uf = &mut UnionFind2::new();
    evaluate(
        &predicate,
        &mut uf,
        &mut id_map,
        //&mut string_map,
        &mut canonical_map,
    )
}

fn evaluate(
    pred: &Rc<Predicate>,
    union_find: &mut UnionFind,
    id_map: &mut HashMap<String, i32>,
    //string_map: &mut HashMap<i32, String>,
    map: &mut HashMap<i32, i32>,
) -> Rc<Predicate> {
    //let uf = union_find.unwrap_or_else(|| UnionFind::new());

    let alphabet: HashSet<String> = vec!["a".to_string(), "b".to_string()].into_iter().collect();

    match pred.as_ref() {
        Predicate::And(_predicates) => {
            // TODO: Unused variable?

            let mut final_preds = Vec::new();
            let mut not_equality_preds = HashSet::new();
            let mut length_preds: HashMap<String, i32> = HashMap::new();
            let not_allowed_lengths: HashMap<String, HashSet<i32>> = HashMap::new();
            let mut equalities = HashSet::new();

            let all_preds = flatten_and_predicates(pred);

            for p in all_preds {
                match p.as_ref() {
                    Predicate::Not(_) => {
                        not_equality_preds.insert(p);
                    }
                    Predicate::Equals(..) => {
                        equalities.insert(p);
                    }
                    Predicate::EqualLength(var, length) => {
                        if let Some(temp) = length_preds.get(&var.name) {
                            if *temp != *length {
                                return Rc::new(Predicate::False);
                            }
                        }
                        length_preds.insert(var.name.clone(), *length);
                    }
                    Predicate::False => {
                        return Rc::new(Predicate::False);
                    }
                    _ => {}
                }
            }
            for p in equalities {
                let leftId;
                let rightId;
                if let Predicate::Equals(left, right) = p.as_ref() {
                    match (left.as_ref(), right.as_ref()) {
                        (Either::Right(str_ind_1), Either::Right(str_ind_2)) => {
                            let mut index_str = format!(
                                "StringIndex(var: {}, index: {})",
                                str_ind_1.var.name, str_ind_1.index
                            );
                            leftId = id_map[&index_str];
                            index_str = format!(
                                "StringIndex(var: {}, index: {})",
                                str_ind_2.var.name, str_ind_2.index
                            );
                            rightId = id_map[&index_str];
                        }
                        (Either::Left(char_expr_1), Either::Right(str_ind_2)) => {
                            let index_str = format!(
                                "StringIndex(var: {}, index: {})",
                                str_ind_2.var.name, str_ind_2.index
                            );
                            leftId = id_map[&index_str];
                            let expr_str = match char_expr_1.as_ref() {
                                CharExpression::CharVar(name) => format!("CharVar({})", name),
                                CharExpression::Literal(value) => format!("Literal({})", value),
                            };
                            rightId = id_map[&expr_str];
                        }
                        (Either::Right(str_ind_2), Either::Left(char_expr_1)) => {
                            let index_str = format!(
                                "StringIndex(var: {}, index: {})",
                                str_ind_2.var.name, str_ind_2.index
                            );
                            let expr_str = match char_expr_1.as_ref() {
                                CharExpression::CharVar(name) => format!("CharVar({})", name),
                                CharExpression::Literal(value) => format!("Literal({})", value),
                            };
                            leftId = id_map[&index_str];
                            rightId = id_map[&expr_str];
                        }
                        (Either::Left(char_expr_1), Either::Left(char_expr_2)) => {
                            let mut expr_str = match char_expr_1.as_ref() {
                                CharExpression::CharVar(name) => format!("CharVar({})", name),
                                CharExpression::Literal(value) => format!("Literal({})", value),
                            };
                            leftId = id_map[&expr_str];
                            expr_str = match char_expr_2.as_ref() {
                                CharExpression::CharVar(name) => format!("CharVar({})", name),
                                CharExpression::Literal(value) => format!("Literal({})", value),
                            };
                            rightId = id_map[&expr_str];
                        }
                    }
                    if leftId == rightId {
                        final_preds.push(p);
                    }
                    if let (Some(value1), Some(value2)) = (map.get(&leftId), map.get(&rightId)) {
                        if value1 != value2 {
                            return Rc::new(Predicate::False);
                        } else {
                            union_find.union(leftId as usize, rightId as usize);
                        }
                    } else if let Some(value1) = map.get(&leftId) {
                        union_find.union(leftId as usize, rightId as usize);
                        let new_canon = union_find.find(leftId as usize);
                        map.insert(new_canon as i32, *value1);
                    } else if let Some(value1) = map.get(&rightId) {
                        union_find.union(leftId as usize, rightId as usize);
                        let new_canon = union_find.find(leftId as usize);
                        map.insert(new_canon as i32, *value1);
                    } else {
                        union_find.union(leftId as usize, rightId as usize);
                    }
                }
            }
            let  cant_equal_chars: HashMap<String, HashSet<String>> = HashMap::new();
            println!("hello there");
            for not_pred in not_equality_preds {
                if let Predicate::Not(inner) = &*not_pred {
                    let  leftId;
                    let  rightId;
                    if let Predicate::Equals(left, right) = inner.as_ref() {
                        match (left.as_ref(), right.as_ref()) {
                            (Either::Right(str_ind_1), Either::Right(str_ind_2)) => {
                                let mut index_str = format!(
                                    "StringIndex(var: {}, index: {})",
                                    str_ind_1.var.name, str_ind_1.index
                                );
                                leftId = id_map[&index_str];
                                index_str = format!(
                                    "StringIndex(var: {}, index: {})",
                                    str_ind_2.var.name, str_ind_2.index
                                );
                                rightId = id_map[&index_str];
                                if leftId == rightId {
                                    return Rc::new(Predicate::False);
                                }
                                if let (Some(_), Some(_)) =
                                    (map.get(&leftId), map.get(&rightId))
                                {
                                    final_preds.push(not_pred)
                                }
                            }
                            (Either::Left(char_expr_1), Either::Right(str_ind_2)) => {
                                let index_str = format!(
                                    "StringIndex(var: {}, index: {})",
                                    str_ind_2.var.name, str_ind_2.index
                                );
                                leftId = id_map[&index_str];
                                let expr_str = match char_expr_1.as_ref() {
                                    CharExpression::CharVar(name) => format!("CharVar({})", name),
                                    CharExpression::Literal(value) => format!("Literal({})", value),
                                };
                                rightId = id_map[&expr_str];
                                if leftId == rightId {
                                    return Rc::new(Predicate::False);
                                }
                                if let (Some(_), Some(_)) =
                                    (map.get(&leftId), map.get(&rightId))
                                {
                                    final_preds.push(not_pred)
                                }
                            }
                            (Either::Right(str_ind_2), Either::Left(char_expr_1)) => {
                                let index_str = format!(
                                    "StringIndex(var: {}, index: {})",
                                    str_ind_2.var.name, str_ind_2.index
                                );
                                let expr_str = match char_expr_1.as_ref() {
                                    CharExpression::CharVar(name) => format!("CharVar({})", name),
                                    CharExpression::Literal(value) => format!("Literal({})", value),
                                };
                                leftId = id_map[&index_str];
                                rightId = id_map[&expr_str];
                                if leftId == rightId {
                                    return Rc::new(Predicate::False);
                                }
                                if let (Some(_), Some(_)) =
                                    (map.get(&leftId), map.get(&rightId))
                                {
                                    final_preds.push(not_pred)
                                }
                            }
                            (Either::Left(char_expr_1), Either::Left(char_expr_2)) => {
                                let mut expr_str = match char_expr_1.as_ref() {
                                    CharExpression::CharVar(name) => format!("CharVar({})", name),
                                    CharExpression::Literal(value) => format!("Literal({})", value),
                                };
                                leftId = id_map[&expr_str];
                                expr_str = match char_expr_2.as_ref() {
                                    CharExpression::CharVar(name) => format!("CharVar({})", name),
                                    CharExpression::Literal(value) => format!("Literal({})", value),
                                };
                                rightId = id_map[&expr_str];
                                if leftId == rightId {
                                    return Rc::new(Predicate::False);
                                }
                                if let (Some(_), Some(_)) =
                                    (map.get(&leftId), map.get(&rightId))
                                {
                                    final_preds.push(not_pred)
                                }
                            }
                        }
                    } else if let Predicate::EqualLength(var_name, length) = &**inner {
                        //let Predicate::EqualLength(var_name, length) = &**inner;
                        let mut flag = false;
                        for (key, value) in id_map.iter() {
                            if key.starts_with("StringIndex"){
                                if let Some(start_pos) = key.find("var: ") {
                                    let start = start_pos + 5;  // 5 is the length of "var: "
                                    if let Some(end_pos) = key[start..].find(",") {
                                        if Some(key[start..start + end_pos].to_string()) == Some(var_name.name.clone()){
                                            if let Some(start_pos_index) = key.find("index: ") {
                                                // Skip past "index: " to the start of the actual index value
                                                let start_index = start_pos_index + 7; // 7 is the length of "index: "
                                                // Extract the index value, assuming it ends with a closing parenthesis or end of the string
                                                if let Some(end_pos_index) = key[start_index..].find(")") {
                                                    let index_value = key[start_index..start_index + end_pos_index].to_string();
                                                    if let Ok(index) = index_value.parse::<i32>() {
                                                        // If parsing is successful, compare the index with length
                                                        if index >= *length && union_find.find(*value as usize) != *value as usize {
                                                            flag = true;
                                                            break;
                                                        }
                                                    }
                                                }
                                            }

                                        }

                                    }
                                }
                            }
                        }
                        if flag {
                            continue;
                        }

                        final_preds.push(not_pred);
                    }
                }
            }
            for (_, chars) in cant_equal_chars {
                if alphabet.iter().all(|c| chars.contains(c)) {
                    return Rc::new(Predicate::False);
                }
            }

            for (var_name, length) in length_preds {
                if not_allowed_lengths
                    .get(&var_name)
                    .map_or(false, |lengths| lengths.contains(&length))
                {
                    return Rc::new(Predicate::False);
                }
                for (key, value) in id_map.iter() {
                    if key.starts_with("StringIndex"){
                        if let Some(start_pos) = key.find("var: ") {
                            let start = start_pos + 5;  // 5 is the length of "var: "
                            if let Some(end_pos) = key[start..].find(",") {
                                if Some(key[start..start + end_pos].to_string()) == Some(var_name.clone()){
                                    if let Some(start_pos_index) = key.find("index: ") {
                                        // Skip past "index: " to the start of the actual index value
                                        let start_index = start_pos_index + 7; // 7 is the length of "index: "
                                        // Extract the index value, assuming it ends with a closing parenthesis or end of the string
                                        if let Some(end_pos_index) = key[start_index..].find(")") {
                                            let index_value = key[start_index..start_index + end_pos_index].to_string();
                                            if let Ok(index) = index_value.parse::<i32>() {
                                                // If parsing is successful, compare the index with length
                                                if index >= length && union_find.find(*value as usize) != *value as usize {
                                                     return Rc::new(Predicate::False);
                                                }
                                            }
                                        }
                                    }

                                }

                            }
                        }
                    }
                }
                let string_var = Rc::new(StringVar { name: var_name });
                final_preds.push(Rc::new(Predicate::EqualLength(string_var, length)));
            }
            match final_preds.len() {
                0 => Rc::new(Predicate::True),
                1 => final_preds[0].clone(),
                _ => Rc::new(Predicate::And(final_preds)),
            }
        }
        Predicate::Or(predicates) => {
            let mut final_set = Vec::new();

            for p in predicates {
                let mut canonical_map: HashMap<i32, i32> = HashMap::new();
                let mut uf: UnionFind<usize> = UnionFind::new(id_map.len()+1);
                let p_eval = evaluate(&p.clone(), &mut uf, id_map, &mut canonical_map);
                match &*p_eval {
                    Predicate::True => return Rc::new(Predicate::True),
                    Predicate::False => {
                        continue;
                    }
                    _ => final_set.push(p_eval),
                }
            }

            match final_set.len() {
                0 => Rc::new(Predicate::False),
                1 => final_set[0].clone(),
                _ => Rc::new(Predicate::Or(final_set)),
            }
        }
        _ => pred.clone(),
    }
}

fn distribute_ors(predicates: Vec<Rc<Predicate>>) -> Rc<Predicate> {
    let mut distributed: Vec<&[Rc<Predicate>]> = Vec::with_capacity(predicates.len());

    for pred in &predicates {
        match pred.as_ref() {
            Predicate::Or(sub_preds) => {
                distributed.push(sub_preds.as_slice());
            }
            _ => {
                distributed.push(std::slice::from_ref(pred));
            }
        }
    }

    let product = cartesian_product(&distributed);

    let estimated_size = distributed
        .iter()
        .take(3)
        .map(|group| group.len())
        .product();
    let mut dnf_result = Vec::with_capacity(estimated_size);

    for group in product {
        let estimated_group_size: usize = group
            .iter()
            .map(|pred| match pred.as_ref() {
                Predicate::And(sub_preds) => sub_preds.len(),
                _ => 1,
            })
            .sum();

        let mut flattened_group = Vec::with_capacity(estimated_group_size);

        for pred in group {
            match pred.as_ref() {
                Predicate::And(sub_preds) => {
                    flattened_group.extend(sub_preds.iter().map(Rc::clone));
                }
                _ => {
                    flattened_group.push(Rc::clone(&pred));
                }
            }
        }

        dnf_result.push(Rc::new(Predicate::And(flattened_group)));
    }

    match dnf_result.len() {
        0 => Rc::new(Predicate::True),
        1 => dnf_result.pop().unwrap(),
        _ => Rc::new(Predicate::Or(dnf_result)),
    }
}

fn cartesian_product(vectors: &[&[Rc<Predicate>]]) -> Vec<Vec<Rc<Predicate>>> {
    if vectors.is_empty() {
        return vec![];
    }

    let total_size = vectors.iter().map(|v| v.len()).product();

    let mut result = Vec::with_capacity(total_size);

    for item in vectors[0].iter() {
        let mut initial = Vec::with_capacity(vectors.len());
        initial.push(Rc::clone(item));
        result.push(initial);
    }

    for vector in vectors.iter().skip(1) {
        let mut new_result = Vec::with_capacity(result.len() * vector.len());

        for existing in result {
            for item in vector.iter() {
                let mut new_combination = Vec::with_capacity(existing.len() + 1);
                new_combination.extend_from_slice(&existing);
                new_combination.push(Rc::clone(item));
                new_result.push(new_combination);
            }
        }

        result = new_result;
    }

    result
}
pub fn convertToDNF(pred: &Rc<Predicate>) -> Rc<Predicate> {
    fn flatten_predicates(
        preds: &[Rc<Predicate>],
        constructor: fn(Vec<Rc<Predicate>>) -> Predicate,
    ) -> Vec<Rc<Predicate>> {
        let mut flattened = Vec::with_capacity(preds.len());
        for pred in preds {
            match pred.as_ref() {
                Predicate::Or(children) if matches!(constructor(vec![]), Predicate::Or(_)) => {
                    flattened.extend(children.iter().cloned());
                }
                Predicate::And(children) if matches!(constructor(vec![]), Predicate::And(_)) => {
                    flattened.extend(children.iter().cloned());
                }
                _ => flattened.push(Rc::clone(pred)),
            }
        }
        flattened
    }

    match pred.as_ref() {
        Predicate::Or(children) => {
            let mut dnf_children = Vec::with_capacity(children.len());

            for child in children {
                dnf_children.push(convertToDNF(child));
            }

            let flattened = flatten_predicates(&dnf_children, Predicate::Or);

            Rc::new(Predicate::Or(flattened))
        }

        Predicate::And(children) => {
            let mut dnf_children = Vec::with_capacity(children.len());

            for child in children {
                dnf_children.push(convertToDNF(child));
            }

            let flattened = flatten_predicates(&dnf_children, Predicate::And);

            distribute_ors(flattened)
        }

        Predicate::Not(sub_pred) => match sub_pred.as_ref() {
            Predicate::And(children) => {
                let mut new_children = Vec::with_capacity(children.len());
                for child in children {
                    new_children.push(convertToDNF(&Rc::new(Predicate::Not(Rc::clone(child)))));
                }
                Rc::new(Predicate::Or(new_children))
            }

            Predicate::Or(children) => {
                let mut new_children = Vec::with_capacity(children.len());
                for child in children {
                    new_children.push(convertToDNF(&Rc::new(Predicate::Not(Rc::clone(child)))));
                }
                Rc::new(Predicate::And(new_children))
            }

            Predicate::Not(sub) => convertToDNF(sub),

            Predicate::True => Rc::new(Predicate::False),
            Predicate::False => Rc::new(Predicate::True),

            _ => Rc::clone(pred),
        },

        _ => Rc::clone(pred),
    }
}
