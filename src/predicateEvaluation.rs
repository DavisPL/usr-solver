use crate::classes::{GenRegex, Predicate, CharExpression,  StringVar, StringIndex};
use crate::unionFind::{UnionFind};
use std::rc::Rc;
use std::collections::{HashMap, HashSet};
use either::Either;
use crate::print::{print_predicate, print_equals_arg, print_char_expression,  print_string_var, print_gre};

pub fn flatten_and_predicates(pred: &Rc<Predicate>) -> Vec<Rc<Predicate>>{
    match pred.as_ref(){
        Predicate::And(children) =>{
            let mut flattened: Vec<Rc<Predicate>> = Vec::new();
            for child in children{
                flattened.push(convertToDNF(&Rc::clone(child)))
            }
            return flattened;
        },
        _ => {
            return vec![Rc::clone(pred)];
        }
    }
}

pub fn evaluateComplete(pred: &Rc<Predicate>) -> Rc<Predicate>{
    let predicate = convertToDNF(pred);
    let mut uf = &mut UnionFind::new();
    return evaluate(&predicate, uf);
}

fn evaluate(pred: &Rc<Predicate>, union_find: &mut UnionFind) -> Rc<Predicate>{
    //let uf = union_find.unwrap_or_else(|| UnionFind::new());


    let alphabet: HashSet<String> = vec!["a".to_string(), "b".to_string()].into_iter().collect();


    match pred.as_ref(){
        Predicate::And(predicates) =>{
            
            let mut final_preds = Vec::new();
            let mut not_equality_preds = HashSet::new();
            let mut length_preds: HashMap<String, i32> = HashMap::new();
            let mut not_allowed_lengths: HashMap<String, HashSet<i32>> = HashMap::new();
            let mut equalities = HashSet::new();

            let all_preds = flatten_and_predicates(pred);

            for p in all_preds{
                match p.as_ref(){
                    Predicate::Not(_) =>{
                        not_equality_preds.insert(p);
                    }
                    Predicate::Equals(..) =>{
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
            for p in equalities{
                if let Predicate::Equals(left, right) = &*p {
                    match union_find.union(left.clone(), right.clone()) {
                        Ok(result) => match result {
                            _ => final_preds.push(p),
                        },
                        Err(_) => {
                            return Rc::new(Predicate::False);
                        },
                    }
                }
            }
            let mut cant_equal_chars: HashMap<String, HashSet<String>> = HashMap::new();
            for not_pred in not_equality_preds {
                if let Predicate::Not(inner) = &*not_pred {
                    if let Predicate::Equals(left, right) = &**inner {
                        let uf_left = union_find.find(left.clone());
                        let uf_right = union_find.find(right.clone());
                        let uf_left_obj = union_find.string_to_object(&uf_left);
                        let uf_right_obj = union_find.string_to_object(&uf_right);

                        if uf_left == uf_right {
                            return Rc::new(Predicate::False);
                        }
                        match (uf_left_obj, uf_right_obj) { //Need to fix with correct Eithers TODO
                            (Either::Left(c_expr_1), Either::Left(c_expr_2)) =>{
                                match c_expr_1.as_ref() {
                                    CharExpression::Literal(_) =>{
                                        match c_expr_2.as_ref() {
                                            CharExpression::CharVar(_) =>{
                                                final_preds.push(not_pred)
                                            }
                                            _ => {}
                                        }
                                    },
                                    CharExpression::CharVar(_) =>{
                                        match c_expr_2.as_ref() {
                                            CharExpression::Literal(val) =>{
                                                cant_equal_chars.entry(uf_left)
                                                    .or_insert_with(HashSet::new)
                                                    .insert(val.clone());
                                                final_preds.push(not_pred);
                                            }
                                            _ => {
                                                final_preds.push(not_pred);
                                            }

                                    }

                                }

                            }},
                            (Either::Right(..), Either::Left(c_expr)) =>{
                                match c_expr.as_ref() {
                                    CharExpression::Literal(val) =>{
                                        cant_equal_chars.entry(uf_left)
                                            .or_insert_with(HashSet::new)
                                            .insert(val.clone());
                                        final_preds.push(not_pred);
                                    }
                                    _ => {
                                        final_preds.push(not_pred);
                                    }
                                }
                            },
                            (_, _) => {
                                final_preds.push(not_pred);
                            }
                        }

                    } else if let Predicate::EqualLength(var_name, length) = &**inner{
                        //let Predicate::EqualLength(var_name, length) = &**inner;
                        let mut flag = false;
                        for (key, value) in union_find.parent.iter() {
                            let object = union_find.string_to_object(&key.to_string());
                            let objectV = union_find.string_to_object(&value.to_string());
                            if let Either::Right(str_var) = object {
                                if str_var.var.name == var_name.name && str_var.index >= *length {
                                    if !matches!(objectV, Either::Right(_)) {
                                        flag = true;
                                        break;
                                    } else if let Either::Right(str_i) = objectV {
                                        let str_ind = str_i.as_ref();
                                        if str_ind.var.name != str_var.var.name || str_ind.index == str_var.index {
                                            flag = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        if flag{
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
                if not_allowed_lengths.get(&var_name)
                    .map_or(false, |lengths| lengths.contains(&length)) {
                    return Rc::new(Predicate::False);
                }

                for (key, value) in union_find.parent.iter() {
                    let object = union_find.string_to_object(&key.to_string());
                    let objectV = union_find.string_to_object(&value.to_string());
                    if let Either::Right(str_var) = object {
                        if str_var.var.name == var_name && str_var.index >= length {
                            if !matches!(objectV, Either::Right(_)) {
                                return Rc::new(Predicate::False);
                            } else if let Either::Right(str_i) = objectV {
                                let str_ind = str_i.as_ref();
                                if str_ind.var.name != str_var.var.name || str_ind.index != str_var.index {
                                    println!("damn it {} {}", key, value);
                                    return Rc::new(Predicate::False);
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


        },
        Predicate::Or(predicates) => {
            let mut final_set = Vec::new();
            
            for p in predicates {
                let p_eval = evaluate(&p.clone(), &mut UnionFind::new());
                match &*p_eval {
                    Predicate::True => return Rc::new(Predicate::True),
                    Predicate::False => {
                        continue;
                    },
                    _ => final_set.push(p_eval),
                }
            }

            match final_set.len() {
                0 => Rc::new(Predicate::False),
                1 => final_set[0].clone(),
                _ => Rc::new(Predicate::Or(final_set)),
            }
        },
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
    
    let estimated_size = distributed.iter()
        .take(3)
        .map(|group| group.len())
        .product();
    let mut dnf_result = Vec::with_capacity(estimated_size);

    for group in product {
        let estimated_group_size: usize = group.iter()
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
        _ => Rc::new(Predicate::Or(dnf_result))
    }
}

fn cartesian_product<'a>(vectors: &[&'a [Rc<Predicate>]]) -> Vec<Vec<Rc<Predicate>>> {
    if vectors.is_empty() {
        return vec![];
    }

    let total_size = vectors.iter()
        .map(|v| v.len())
        .product();

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
    fn flatten_predicates(preds: &[Rc<Predicate>], constructor: fn(Vec<Rc<Predicate>>) -> Predicate) -> Vec<Rc<Predicate>> {
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
            
            _ => Rc::clone(pred)
        }
        
        _ => Rc::clone(pred),
    }
}
