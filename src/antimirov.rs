use crate::classes::{CharExpression, GenRegex};
use std::rc::Rc;
use crate::print::print_gre;
use std::collections::{HashMap, HashSet};
use disjoint_sets::UnionFind;
use std::collections::BTreeSet;




type GenRegexPairSet = BTreeSet<(Rc<GenRegex>, Rc<GenRegex>)>;
type OuterSet = HashSet<(Rc<GenRegex>, GenRegexPairSet)>;


fn parse_string_vars(string_set: &HashSet<(Rc<GenRegex>, Rc<GenRegex>)>, union_find: &UnionFind<usize>, id_map: &HashMap<String, i32>, canonical_map: &HashMap<i32, i32>) -> (HashMap<Rc<GenRegex>, Rc<GenRegex>>, HashMap<Rc<GenRegex>, bool>){
    let string_dict = HashMap::new();
    let return_vals = HashMap::new();
    let truncate = HashMap::new();
    for elem in string_set {
        string_dict.entry(elem.0.clone()).or_insert(Vec::new()).push(elem.1);
    }
    for (key, value) in string_dict{
        let mut value_copy = value.clone();
        let mut last_pop = None;
        while value.len()>1{
            let mut union_elems = HashSet::new();
            let mut new_element_to_add = None;
            let mut literal_val = None;
            let mut i = 0;
            while i < value.len(){
                let expr = value[i].clone();
                match expr.as_ref() {
                    GenRegex::Concatenation(left, right) => {
                        union_elems.insert(left);
                        value[i] = right.clone();
                    }
                    GenRegex::CharExpression(c_expr) => match c_expr.as_ref(){
                        CharExpression::Literal(value) =>{
                            if value == ""{
                                value.remove(i);
                                last_pop = Some(value_copy.remove(i));
                                union_elems.insert(&expr.clone());
                                truncate.insert(key.clone(), true);
                                continue;
                            }else{
                                union_elems.insert(&expr);

                            }
                        }
                        _ =>{
                            union_elems.insert(&expr);
                        }

                    }
                    _ => {
                        if let GenRegex::StringVar(_) = expr.as_ref() {
                            value.remove(i);
                            last_pop = Some(value_copy.remove(i));
                            continue;
                        }
                        union_elems.insert(&expr);
                    }
                }
                value[i] = Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal("".to_string()))));
                i += 1;
            }
            let mut prev = None;
            for elem in union_elems {
                if let Some(p) = prev {
                    if let Some(leftId) = id_map.get(&prev) {
                        if let Some(rightId) = id_map.get(&elem) {
                            let rightId = *rightId;
                            let leftId = *leftId; // Dereference if you need the inner value directly
                    if let (Some(value1), Some(value2)) = (canonical_map.get(&leftId), canonical_map.get(&rightId)) {
                        if value1 != value2 {
                            truncate.insert(key, false);
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
                prev = Some(elem);
            }
        }
        if value_copy.len() != 0{
            return_vals.insert(key, value_copy[0]);
        }else{
            return_vals.insert(key, last_pop.expect("reason"));
        }
    }
    return (return_vals, truncate);

}

fn merge(substitutions: GenRegexPairSet) -> GenRegexPairSet{
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
    for sub in &substitutions{
        match &sub.0.as_ref() {
            GenRegex::StringVar(_) => {
                string_set.insert(sub.clone());
            },
            _ => {
                char_set.insert(sub.clone());
            }
        }
    }
    let (mut return_vals, truncate) = parse_string_vars(&string_set, &union_find, &id_map, &canonical_map); 
    
    if truncate.len() != return_vals.len(){
        return BTreeSet::new();  // Return an empty set if there’s an error

    }
    for sub in &char_set{
        let index_str_1 = print_gre(&sub.0);
        let index_str_2 = print_gre(&sub.1);
        let leftId = id_map[&index_str_1];
        let rightId = id_map[&index_str_2];
        if let (Some(value1), Some(value2)) = (canonical_map.get(&leftId), canonical_map.get(&rightId)) {
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
        let mut temp = value;
        loop {
            // Handle Concatenation case
            if let GenRegex::Concatenation ( left, right ) = temp.as_ref() {
                if let GenRegex::CharExpression(left_char) = left.as_ref() {
                    if let CharExpression::CharVar(left_char_1) = left_char.as_ref(){
                    if let Some(val_1) = canonical_map.get(&id_map[&print_gre(&left)]){
                        *left_char = Rc::new(CharExpression::Literal(string_map[val_1]));
                    }
                    }

                }

                if let GenRegex::StringVar(_) = right.as_ref(){
                    if truncate.get(key).copied().unwrap_or(false) {
                        *temp = left.clone();
                        // temp.right = Literal("");
                    } else {
                        *temp = right.clone();
                    }
                } else {
                    *temp = right.clone();
                }
            } else {
                // Handle CharVar case
                if let GenRegex::CharExpression(left_char) = temp.as_ref(){
                    if let CharExpression::CharVar(left_char_1) =  left_char.as_ref(){
                    if let Some(val_1) = canonical_map.get(&id_map[&print_gre(temp)]){
                        *left_char = Rc::new(CharExpression::Literal(string_map[val_1].clone()));
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
    for key in id_map.keys(){
        if key.starts_with("char(") && key.ends_with(")") {
    let name = Some(&key[5..key.len() - 1]);
    let c_obj = Rc::new(CharExpression::CharVar(name.expect("can't convert to char for some reason").to_string()));

    // Try to get the value from canonical_map
    if let Some(value) = canonical_map.get(&(union_find.find(id_map[key] as usize) as i32)) {
        // Insert CharExpression mapped to a literal value
        final_subs.insert(
            Rc::new(GenRegex::CharExpression(c_obj.clone())),
            Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(string_map[value].clone())))),
        );
    } else {
        // Attempt to get the map value from id_map (safe lookup)
        if let Some(id) = id_map.get(key) {
            let map_val = union_find.find(*id as usize);

            // Get the corresponding string from string_map
            if let Some(map_str) = string_map.get(&(map_val as i32)) {
                if map_str.starts_with("char(") && map_str.ends_with(")") {
                    let name_char = Some(&map_str[5..map_str.len() - 1]); // Extract the content inside "char(...)"
                    let c_obj_map = Rc::new(CharExpression::CharVar(name_char.expect("can't convert").to_string()));

                    // Insert the generated CharExpression object
                    final_subs.insert(Rc::new(GenRegex::CharExpression(c_obj.clone())), Rc::new(GenRegex::CharExpression(c_obj_map)));
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
        GenRegex::CharExpression(c_expr) => {
            match (deriv_char.as_ref(), c_expr.as_ref()) {
                (CharExpression::Literal(deriv_lit), CharExpression::Literal(literal_value)) =>{
                    if deriv_lit == literal_value{
                        let mut ret = HashSet::new();
                        ret.insert((empty_string(), BTreeSet::new()));
                        ret
                    }else{
                        HashSet::new()
                    }
                },
                (CharExpression::CharVar(_), CharExpression::Literal(_)) =>{
                    let mut ret = HashSet::new();
                    let mut subs = BTreeSet::new();
                    subs.insert((Rc::new(GenRegex::CharExpression(deriv_char.clone())), gre.clone()));
                    ret.insert((empty_string(), subs));
                    ret
                },
                (CharExpression::Literal(_), CharExpression::CharVar(_)) =>{
                    let mut ret = HashSet::new();
                    let mut subs = BTreeSet::new();
                    subs.insert((gre.clone(), Rc::new(GenRegex::CharExpression(deriv_char.clone()))));
                    ret.insert((empty_string(), subs));
                    ret
                },
                (CharExpression::CharVar(_), CharExpression::CharVar(_)) =>{
                    let mut ret = HashSet::new();
                    let mut subs = BTreeSet::new();
                    subs.insert((gre.clone(), Rc::new(GenRegex::CharExpression(deriv_char.clone())) ));
                    ret.insert((empty_string(), subs));
                    ret
                },
            }
        }
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
        },
        GenRegex::Union(side1, side2) => {
            let side1_deriv = derivative(&side1, deriv_char);
            let side2_deriv = derivative(&side2, deriv_char);
            side1_deriv.union(&side2_deriv).cloned().collect()
        }
        GenRegex::Intersect(left, right) => {
            let p_deriv = derivative(left, deriv_char);
            let q_deriv = derivative(right, deriv_char);
            let mut term1 = HashSet::new();

            for p_sub in p_deriv {
                for q_sub in &q_deriv {
                    let curr = (
                        Rc::new(GenRegex::Intersect(
                            sub_in(&p_sub.0, &q_sub.1.difference(&p_sub.1).cloned().collect::<BTreeSet<_>>()),
                            sub_in(&q_sub.0, &p_sub.1.difference(&q_sub.1).cloned().collect::<BTreeSet<_>>())
                        )),
                        merge(p_sub.1.union(&q_sub.1).cloned().collect::<BTreeSet<_>>())
                    );
                    if curr.1.len() == 0{
                        continue;
                    }
                    term1.insert(curr);
                }
            }

            term1
        },
        GenRegex::Concatenation(left, right) => {
            let left_deriv = derivative(left, deriv_char);
            //let right_deriv = derivative(right, deriv_char);

            // Create term1 set
            let mut term1 = HashSet::new();
            if !left_deriv.is_empty() {
                for sub in left_deriv {
                    if let GenRegex::CharExpression(c_expr) = sub.0.as_ref() {
                        if let CharExpression::Literal(lit) = c_expr.as_ref(){
//                    if let GenRegex::CharExpression(CharExpression::Literal(lit)) = sub.0.as_ref() {
                        if lit == "" {
                            let curr = (sub_in(right, &sub.1.clone()), sub.1.clone());
                            term1.insert(curr);
                        } else {
                            let curr = (Rc::new(GenRegex::Concatenation(sub.0.clone(), sub_in(right, &sub.1.clone()))), sub.1.clone());
                            term1.insert(curr);
                        }}
                    }
                }
            }

            let p_nullable = nullable(left);
            if !p_nullable.is_empty() {
                for sub in p_nullable {
                    let temp = sub_in(right, &sub);
                    let deriv = derivative(&temp, deriv_char);
                    if deriv.is_empty() {
                        continue;
                    }
                    term1 = term1.union(&deriv).cloned().collect();
                }
            }

            term1
        }
        GenRegex::Kleene(expr) => {
            let p_deriv = derivative(expr, deriv_char);
            let mut term1 = HashSet::new();

            for sub in p_deriv {
                let curr = (Rc::new(GenRegex::Concatenation(sub.0.clone(), sub_in(expr, &sub.1.clone()))), sub.1.clone());
                term1.insert(curr);
            }

            term1
        }
        _ => HashSet::new(),
    }
}

fn sub_in(expr: &Rc<GenRegex>, substitution: &GenRegexPairSet) -> Rc<GenRegex>{
    if substitution.len() == 0 {
        return Rc::clone(expr); // Return a clone of the Rc, as Rc handles reference counting
    }

    // Create a HashMap for substitutions
    let mut subs: HashMap<String, &Rc<GenRegex>> = HashMap::new();
    
    // Populate the HashMap with substitutions
    for sub in substitution.iter() {
        let key = print_gre(&sub.0);  // Assuming this converts GenRegex to String
        subs.insert(key, &sub.1);     // Insert the key-value pair into the HashMap
    }
    sub_in_helper(expr, subs)
}

fn sub_in_helper(expr: &Rc<GenRegex>, sub: HashMap<String, &Rc<GenRegex>>) -> Rc<GenRegex>{
    match expr.as_ref() {
        GenRegex::StringVar(_) => {
            let key = print_gre(expr);
            match sub.get(&key){
                Some(value) => Rc::clone(value),
                None => Rc::clone(expr)
            }
        }
        GenRegex::CharExpression(c_expr) => match c_expr.as_ref() {
            CharExpression::CharVar(_) => {
                let key = print_gre(expr);
                match sub.get(&key){
                    Some(value) => Rc::clone(value),
                    None => Rc::clone(expr)
                }
            },
            CharExpression::Literal(_) => Rc::clone(expr)
        }
        GenRegex::Intersect(left, right) =>{
            let leftSub = sub_in_helper(left, sub.clone());
            let rightSub = sub_in_helper(right, sub);
            Rc::new(GenRegex::Intersect(leftSub, rightSub))
        }
        GenRegex::Concatenation(left, right) =>{
            let leftSub = sub_in_helper(left, sub.clone());
            let rightSub = sub_in_helper(right, sub);
            Rc::new(GenRegex::Concatenation(leftSub, rightSub))
        }
        GenRegex::Union(left, right) =>{
            let leftSub = sub_in_helper(left, sub.clone());
            let rightSub = sub_in_helper(right, sub);
            Rc::new(GenRegex::Union(leftSub, rightSub))
        }
        GenRegex::Kleene(inner) =>{
            let innerSub = sub_in_helper(inner, sub);
            Rc::new(GenRegex::Kleene(innerSub))
        }
        _ => Rc::clone(expr)
    }
}

pub fn matching(expr: &Rc<GenRegex>, proposed: String) -> bool {
    println!("{}", proposed);
    if proposed.is_empty() {
        return !nullable(expr).is_empty();
    }
    let literal = Rc::new(CharExpression::Literal(String::from(&proposed[0..1])));
    let deriv = derivative(expr, &literal);
    if deriv.is_empty(){
        return false;
    }
    for elem in deriv{
        if matching(&elem.0, String::from(&proposed[1..])){
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
            CharExpression::CharVar(_name) => {
                BTreeSet::new()
            }
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
        ,
        GenRegex::Union(side1, side2) => {
            let left_null = nullable(&Rc::clone(side1));
            let right_null = nullable(&Rc::clone(side2));
            left_null.union(&right_null);
            left_null
        }
        GenRegex::Intersect(side1, side2) => {
            let  left_null = nullable(&Rc::clone(side1));
            let  right_null = nullable(&Rc::clone(side2));
            let mut retSet = BTreeSet::new();
            for left_elem in &left_null{
                for right_elem in &right_null{
                    left_elem.union(right_elem);
                    let ret = merge(left_elem.clone());
                    if ret.len() != 0{
                        retSet.insert(ret);
                    }
                }
            }
            retSet

        },
        GenRegex::Concatenation(side1, side2) => {
            let  left_null = nullable(&Rc::clone(side1));
            let  right_null = nullable(&Rc::clone(side2));
            let mut retSet = BTreeSet::new();
            for left_elem in &left_null{
                for right_elem in &right_null{
                    left_elem.union(right_elem);
                    let ret = merge(left_elem.clone());
                    if ret.len() != 0{
                        retSet.insert(ret);
                    }
                }
            }
            retSet
        },
        GenRegex::Kleene(_) => {
            let mut ret = BTreeSet::new();
            ret.insert(BTreeSet::new());
            ret
        },
        _ => {
            BTreeSet::new()
        }
    }
       


}
fn assign_unique_ids(substitutions: GenRegexPairSet, id_map: &mut HashMap<String, i32>, next_id: &mut i32) {
    for sub in &substitutions{
        match sub.0.as_ref(){
            GenRegex::StringVar(_) =>{
                let index_str = print_gre(&sub.0);
                id_map.entry(index_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
            GenRegex::CharExpression(c_expr)=> match c_expr.as_ref(){
                CharExpression::CharVar(_) =>{
                    let index_str = print_gre(&sub.0);
                    id_map.entry(index_str).or_insert_with(|| {
                        let id = *next_id;
                        *next_id += 1;
                        id
                    });
                }
                _ =>{}

            }
            _=>{}

        }
        match sub.1.as_ref(){
            GenRegex::StringVar(_) =>{
                let index_str = print_gre(&sub.0);
                id_map.entry(index_str).or_insert_with(|| {
                    let id = *next_id;
                    *next_id += 1;
                    id
                });
            }
            GenRegex::CharExpression(c_expr)=> match c_expr.as_ref(){
                CharExpression::CharVar(_) =>{
                    let index_str = print_gre(&sub.0);
                    id_map.entry(index_str).or_insert_with(|| {
                        let id = *next_id;
                        *next_id += 1;
                        id
                    });
                }
                _ =>{}

            }
            _=>{}

        }
    }
}
