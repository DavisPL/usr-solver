//!
//! Implementation of the Antimirov Derivative
//!

use crate::classes::StringIndex;
use crate::classes::{CharExpression, GenRegex, StringVar, Predicate, MaybeCharExpression, SubExpr, MergeResult, AnySub, SimpleSub, AntimirovDerivativeElement, CharVar};
//use crate::classes::Pair;
//use crate::classes::Subs::Sub;
use disjoint_sets::UnionFind;
use std::collections::BTreeSet;
use std::collections::BTreeMap;
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

fn union_over_set(
    union_find: &mut UnionFind<usize>, 
    union_set: &HashSet<Rc<CharExpression>>, 
    expr_to_id: &mut HashMap<Rc<CharExpression>, usize>, 
    id_to_expr: &mut HashMap<usize, Rc<CharExpression>>, 
    canonical_map: &mut HashMap<Rc<CharExpression>, Rc<CharExpression>>
) -> bool {
    let mut prev: std::option::Option<Rc<CharExpression>> = None;

    for element in union_set{
        if matches!(element.as_ref(), CharExpression::Literal(_)){
            canonical_map.insert(element.clone(), element.clone());
        }

        if let Some(prev_exists) = prev {
            let prev_id: usize;
            let curr_id: usize;
            if expr_to_id.contains_key(&prev_exists){
                prev_id = expr_to_id[&prev_exists];
            }else{
                prev_id = expr_to_id.len()+1;
                expr_to_id.insert(prev_exists.clone(), prev_id);
                id_to_expr.insert(prev_id, prev_exists.clone());
            }
            if expr_to_id.contains_key(element.as_ref()){
                curr_id = expr_to_id[element.as_ref()];
            }else{
                curr_id = expr_to_id.len()+1;
                expr_to_id.insert(element.clone(), curr_id);
                //expr_to_id[element.as_ref()] = curr_id;
                id_to_expr.insert(curr_id, element.clone());
            }// By this point in the code we should have the ID for the 2 elements we are unioning
            if canonical_map.contains_key(&prev_exists) && canonical_map.contains_key(element.as_ref()) && canonical_map[&prev_exists] != canonical_map[element.as_ref()] {
                return false;
            }
            union_find.union(prev_id, curr_id);
            if canonical_map.contains_key(element.as_ref()){
                canonical_map.insert(prev_exists, canonical_map[element.as_ref()].clone());
            }else if canonical_map.contains_key(&prev_exists){
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
    for sub in substitutions.get_str_map().values(){
        for sub_expr in sub{
            count += sub_expr.get_head().len();
        }

    }
    for c_exprs in substitutions.get_char_map().values(){
        count += c_exprs.len();
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
    let mut union_find: UnionFind<usize> = UnionFind::new(count_union_elems(&substitutions)+2);

    for (_,  eq_exprs) in &mut str_eq_class{
        let mut ind = 0;
        while eq_exprs.len() > 1{
            let mut length_flag = false;
            let mut union_set: HashSet<Rc<CharExpression>> = HashSet::new();
            let mut i = 0;
            while i < eq_exprs.len(){
                let curr_sub_expr = &eq_exprs[i];
                if ind < curr_sub_expr.head_length(){
                    let temp = &curr_sub_expr[ind];
                    union_set.insert(Rc::new(temp.clone()));
                    i+=1;
                }else if curr_sub_expr.get_tail() && eq_exprs.len() > 1{
                    eq_exprs.remove(i);
                }else{
                    for j in 0..eq_exprs.len() {
                        if i != j{
                            let r_prime_expr = &eq_exprs[j];
                            if ind < r_prime_expr.head_length(){
                                return MergeResult::Bottom;
                            }else{
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
            if length_flag{
                break;
            }
            ind += 1;
            if !union_over_set(&mut union_find, &union_set, &mut expr_to_id, &mut id_to_expr, &mut canonical_map){
                return MergeResult::Bottom;
            } //TODO: Union everything together here (add in union_find element)
        }
    }
    let mut combined_expr: SimpleSub = SimpleSub::empty();
    for(var, eq_exprs) in &char_eq_class{
        let mut u_set: HashSet<_> = eq_exprs.into_iter()
        .map(|expr| Rc::new((expr).clone())) // Dereference `expr` (&&CharExpression) and clone
        .collect();
        u_set.insert(Rc::new(CharExpression::CharVar(var.clone())));
        if !union_over_set(&mut union_find, &u_set, &mut expr_to_id, &mut id_to_expr, &mut canonical_map){
            return MergeResult::Bottom;
        }
        //TODO: Union
    }
    for var in char_eq_class.keys(){
        let id_var = expr_to_id[&Rc::new(CharExpression::CharVar(var.clone()))];
        let found_expr = id_to_expr[&union_find.find(id_var)].clone();
        if CharExpression::CharVar(var.clone()) != *found_expr{
            combined_expr.set_char_var(var.clone(), found_expr.as_ref().clone());
        }
    }

    //let string_subs = sub_in(string_subs, char_subs); //TODO: implement sub_in
                                                      //
    for (var, mut eq_exprs) in str_eq_class{
        let  sub_expr_vector = eq_exprs[0].get_mut_head();
        for i in 0..sub_expr_vector.len() {
            match &sub_expr_vector[i] {
                CharExpression::CharVar(c_var) =>{
                    let substitution_value = combined_expr.get_char_var(c_var);
                    match substitution_value {
                        Some(v) => {
                            // The key was found, and `v` is the value, so update the vector element
                            sub_expr_vector[i] = v.clone();
                            println!("Updated value at index {}: {:?}", i, v);
                        },
                        None => {
                            // The key was not found, so do nothing
                            println!("No value found for key at index {}", i);
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

fn sub_difference(sub1: Rc<SimpleSub>, sub2: Rc<SimpleSub>)->MergeResult{
    if let MergeResult::SimpleSub(result)=merge(Rc::new(sub1.as_ref().clone().union(sub2.as_ref().clone()))){
        let mut retsub=SimpleSub::empty();
        for (char_var,_) in result.get_char_map(){
            retsub.remove_char_map(char_var);
        }
        for (string_var,sub_expr1) in result.get_str_map(){
            if let Some(sub_expr2)=sub2.get_string_var(string_var){
                if let Some(mut sub)=sub_expr_match(&sub_expr1, &sub_expr2, string_var){
                    retsub.get_char_map_mut().append(&mut sub.get_char_map_mut());
                    retsub.get_str_map_mut().append(&mut sub.get_str_map_mut());
                }
                else{
                    return MergeResult::Bottom;
                }
            }
        }
        return MergeResult::SimpleSub(retsub);
    }
    else{
        MergeResult::Bottom
    }
}
fn sub_expr_match(sub_expr1: &SubExpr, sub_expr2: &SubExpr, str_var: &StringVar)->Option<SimpleSub>{
    let mut retval=SimpleSub::empty();
    if sub_expr1.is_empty() && sub_expr2.is_empty(){
        return Some(retval);
    }
    else if sub_expr1.head_length()==0 && sub_expr1.get_tail(){
        retval.set_string_var(str_var.clone(), sub_expr2.clone());
        return Some(retval);
    }
    else if sub_expr2.head_length()==0 && sub_expr2.get_tail(){
        retval.set_string_var(str_var.clone(), sub_expr1.clone());
        return Some(retval);
    }
    else if sub_expr1.is_empty() || sub_expr2.is_empty() {
        return None;
    }
    let trunc_sub_expr1=SubExpr::new(sub_expr1.get_head()[1..].to_vec(), sub_expr1.get_tail());
    let trunc_sub_expr2=SubExpr::new(sub_expr2.get_head()[1..].to_vec(), sub_expr2.get_tail());
    match sub_expr_match(&trunc_sub_expr1, &trunc_sub_expr2, str_var){
        Some(val)=> retval=val,
        None=> return None
    }
    let head1=&sub_expr1.get_head()[0];
    let head2=&sub_expr2.get_head()[0];
    if let CharExpression::CharVar(key)=head1{
        retval.set_char_var(key.clone(), head2.clone());
    }
    else if let CharExpression::CharVar(key)=head2{
        retval.set_char_var(key.clone(), head1.clone());
    }
    return Some(retval);
}
pub fn derivative(gre: &Rc<GenRegex>, deriv_char: &Rc<CharExpression>) -> HashSet<AntimirovDerivativeElement> {
    let empty_string = || {
        Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
            String::new(),
        ))))
    };

    match gre.as_ref() {
        GenRegex::EmptySet => HashSet::from([AntimirovDerivativeElement::new(
            Rc::new(GenRegex::EmptySet),
            MergeResult::Bottom
        )]),
        GenRegex::CharExpression(c_expr) => match (deriv_char.as_ref(), c_expr.as_ref()) {
            (CharExpression::Literal(deriv_lit), CharExpression::Literal(literal_value)) => {
                if deriv_lit == literal_value {
                    HashSet::from([AntimirovDerivativeElement::new(
                        empty_string(),
                        MergeResult::SimpleSub(SimpleSub::empty())
                    )])
                } else {
                    HashSet::from([AntimirovDerivativeElement::new(
                        Rc::new(GenRegex::EmptySet),
                        MergeResult::Bottom
                    )])
                }
            }
            (CharExpression::CharVar(d_var), CharExpression::Literal(_)) => {
                let mut char_to = BTreeMap::new();
                char_to.insert(d_var.clone(), c_expr.as_ref().clone());
                let subs = MergeResult::SimpleSub(SimpleSub::new(
                 BTreeMap::new(),
                    char_to
                ));
                let ret = HashSet::from([AntimirovDerivativeElement::new(
                 empty_string(),
                subs
                )]);
                ret
            }
            (_, CharExpression::CharVar(c_var)) => {
                let mut char_to = BTreeMap::new();
                char_to.insert(c_var.clone(), deriv_char.as_ref().clone());
                let subs = MergeResult::SimpleSub(SimpleSub::new(
                 BTreeMap::new(),
                    char_to
                ));
                let ret = HashSet::from([AntimirovDerivativeElement::new(
                 empty_string(),
                    subs
                )]);
                ret
            }
        },
        GenRegex::StringVar(string_var) => {

            let mut head = Vec::new();
            head.push(deriv_char.as_ref().clone());
            
            let subexpr = SubExpr::new(
                head,
                true
            );


            let mut string_to = BTreeMap::new();
            string_to.insert(string_var.as_ref().clone(), subexpr);

            let substitution = MergeResult::SimpleSub(SimpleSub::new(
               string_to,
               BTreeMap::new(),
            ));

            let ret = HashSet::from([AntimirovDerivativeElement::new(
             gre.clone(),
             substitution
            )]);
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
            let mut retSet = HashSet::new();

            for p_sub in p_deriv {
                for q_sub in &q_deriv {
                    match (p_sub.get_subs(), q_sub.get_subs()){
                        (MergeResult::SimpleSub(left_elem), MergeResult::SimpleSub(right_elem))=>{
                            let unionLR: AnySub = left_elem.clone().union(right_elem.clone());
                            let ret = merge(Rc::new(unionLR));
                            match ret {
                                MergeResult::SimpleSub(_)=>{
                                    let left_minus_right = sub_difference(Rc::new(left_elem.clone()), Rc::new(right_elem.clone())); //TODO: fix sub_diff
                                    let right_minus_left = sub_difference(Rc::new(right_elem.clone()), Rc::new(left_elem.clone()));
                                    println!("{} right", right_elem);
                                    println!("{} left", left_elem);
                                    println!("{} left-minus-right", left_minus_right);
                                    println!("{} right-minus-left", right_minus_left);
                                    match (left_minus_right, right_minus_left) {
                                        (MergeResult::SimpleSub(l_minus_r), MergeResult::SimpleSub(r_minus_l))=>{
                                            let p_prime_sub = sub_in(p_sub.get_expr(), &l_minus_r);
                                            let q_prime_sub = sub_in(q_sub.get_expr(), &r_minus_l);
                                            let final_expr = Rc::new(GenRegex::Intersect(p_prime_sub, q_prime_sub));
                                            let term = AntimirovDerivativeElement::new(
                                                final_expr,
                                                ret
                                            );
                                            retSet.insert(term);
                                        }
                                        _ =>{}
                                    }
                                },
                                _=>{}
                            }
                        }
                        _ =>{}
                    }

                    /*let merged = merge(p_sub.1.union(&q_sub.1).cloned().collect::<BTreeSet<_>>());
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
                    //}*/
                }
            }

            retSet 
                //unimplemented!();
        }
        GenRegex::Concatenation(left, right) => {
            let left_deriv = derivative(left, deriv_char);
            let right_deriv = derivative(right, deriv_char);

            let mut retSet = HashSet::new();
            for sub in left_deriv {
                match sub.get_subs(){
                    MergeResult::SimpleSub(simple_sub)=>{
                        if let GenRegex::CharExpression(c_expr) = sub.get_expr().as_ref() {
                            if let CharExpression::Literal(lit) = c_expr.as_ref() {
                                //                    if let GenRegex::CharExpression(CharExpression::Literal(lit)) = sub.0.as_ref() {
                                if lit.is_empty() {
                                    let curr = AntimirovDerivativeElement::new(
                                        sub_in(right, &simple_sub), sub.get_subs().clone()
                                    );
                                    retSet.insert(curr);
                                } else {
                                    let curr = AntimirovDerivativeElement::new(
                                        Rc::new(GenRegex::Concatenation(
                                            sub.get_expr().clone(),
                                            sub_in(right, &simple_sub),
                                        )),
                                        sub.get_subs().clone(),
                                    );
                                    retSet.insert(curr);
                                }
                            }
                        } else {
                            let curr = AntimirovDerivativeElement::new(
                                Rc::new(GenRegex::Concatenation(
                                    sub.get_expr().clone(),
                                    sub_in(right, simple_sub),
                                )),
                                sub.get_subs().clone(),
                            );
                            retSet.insert(curr);
                        }

                    }
                    _=>{}
                }
            }

            let p_nullable = nullable(left);
            for n_sub in p_nullable {
                for q_sub in &right_deriv {
                    match (q_sub.get_subs()){
                        (MergeResult::SimpleSub(right_elem))=>{
                            let unionLR: AnySub = n_sub.clone().union(right_elem.clone());
                            let ret = merge(Rc::new(unionLR));
                            match ret {
                                MergeResult::SimpleSub(_)=>{
                                    let right_minus_left = sub_difference(Rc::new(n_sub.clone()), Rc::new(right_elem.clone()));
                                    match (right_minus_left) {
                                        ( MergeResult::SimpleSub(r_minus_l))=>{
                                            let q_prime_sub = sub_in(q_sub.get_expr(), &r_minus_l);
                                            let term = AntimirovDerivativeElement::new(
                                                q_prime_sub,
                                                ret
                                            );
                                            retSet.insert(term);
                                        }
                                        _ =>{}
                                    }
                                },
                                _=>{}
                            }
                        }
                        _ =>{}
                    }




                }
            }

            retSet
            //unimplemented!();
        }
        GenRegex::Kleene(expr) => {
            let p_deriv = derivative(expr, deriv_char);
            let mut term1 = HashSet::new();

            for sub in p_deriv {
                match sub.get_subs() {
                    MergeResult::SimpleSub(s_sub) =>{
                        let curr = AntimirovDerivativeElement::new(
                             Rc::new(GenRegex::Concatenation(
                                sub.get_expr().clone(),
                                sub_in(gre, s_sub),
                            )),
                             MergeResult::SimpleSub(s_sub.clone())
                        );
                        term1.insert(curr);

                    },
                    _ =>{}
                }
            }

            term1
        }
        _ =>{
            unimplemented!();
            
        }
    }
}

fn sub_in(expr: &Rc<GenRegex>, substitution: &SimpleSub) -> Rc<GenRegex> {
    if substitution.get_str_map().is_empty() && substitution.get_char_map().is_empty() {
        return expr.clone(); // Returns a clone of expr.
    }
    match expr.as_ref(){
        GenRegex::EmptySet => Rc::clone(expr),
        GenRegex::CharExpression(char_expr)=>{
            match char_expr.as_ref() {
                CharExpression::CharVar(char_var) => {
                    match substitution.get_char_var(char_var){
                        Some(value) => Rc::new(GenRegex::CharExpression(Rc::new(value.clone()))),
                        None => expr.clone(),
                    }
                },
                CharExpression::Literal(_) => expr.clone(),
            }
        },
        GenRegex::StringVar(string_var) => {
            match substitution.get_string_var(string_var){
                Some(value) => {
                    value.to_gen_regex(string_var)
                },
                None => expr.clone(),
            }
        },
        GenRegex::StringIndex(string_index) => {
            match substitution.get_string_var(&string_index.var){
                Some(value) => {
                    let index=string_index.index as usize;
                    let length=value.get_head().len();
                    if index < length{
                        Rc::new(GenRegex::CharExpression(Rc::new(value.get_head()[index].clone())))
                    }
                    else{
                        if value.get_tail(){
                            Rc::new(GenRegex::StringIndex(Rc::new(StringIndex{var: string_index.var.clone(), index:((index-length+1) as i32)})))
                        }
                        else{
                            Rc::new(GenRegex::EmptySet)
                        }
                    }
                },
                None => expr.clone(),
            }
        },
        GenRegex::StringSlice(string_var, _) => todo!(),
        GenRegex::Union(gen_regex1, gen_regex2) => {
            Rc::new(GenRegex::Union(sub_in(gen_regex1,substitution), sub_in(gen_regex2,substitution)))
        },
        GenRegex::Intersect(gen_regex1, gen_regex2) => {
            Rc::new(GenRegex::Intersect(sub_in(gen_regex1,substitution), sub_in(gen_regex2,substitution)))
        },
        GenRegex::Concatenation(gen_regex1, gen_regex2) => {
            Rc::new(GenRegex::Concatenation(sub_in(gen_regex1,substitution), sub_in(gen_regex2,substitution)))
        },
        GenRegex::Kleene(gen_regex) => {
            Rc::new(GenRegex::Kleene(sub_in(gen_regex,substitution)))
        },
        GenRegex::Complement(gen_regex) => {
            Rc::new(GenRegex::Complement(sub_in(gen_regex,substitution)))
        },
        GenRegex::IfThenElse(predicate, gen_regex1, gen_regex2) => todo!(),
    }
}

pub fn satisfiable(expr: &Rc<GenRegex>) -> bool{
    let mut ind = 0;
    return satisfiable_helper(expr, ind, HashSet::new());
}
pub fn satisfiable_helper(expr: &Rc<GenRegex>, mut index: i32, mut visited: HashSet<GenRegex>) -> bool{
    println!("{}", expr);
    if visited.contains(expr){
        return false;
    }else{
        visited.insert(expr.as_ref().clone());
    }
    if nullable(expr).is_empty(){
        let new_name = "f".to_owned() + &index.to_string();
        let c_var = Rc::new(CharExpression::CharVar(CharVar{name: new_name}));
        let deriv = derivative(expr, &c_var);
        if deriv.is_empty(){
            return false
        }
        index += 1;
        for elem in deriv{
            if satisfiable_helper(&elem.get_expr(), index, visited.clone()){
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
        if matching(&elem.get_expr(), String::from(&proposed[1..])) {
            return true;
        }
    }
    false
}

pub fn nullable(gre: &Rc<GenRegex>) -> HashSet<SimpleSub> {
    let empty_string = || {
        Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
            String::new(),
        ))))
    };
    match gre.as_ref() {
        GenRegex::EmptySet => HashSet::new(),
        GenRegex::CharExpression(cExpr) => match cExpr.as_ref() {
            CharExpression::CharVar(_) => HashSet::new(),
            CharExpression::Literal(value) => {
                if value.is_empty() {
                    let mut ret = HashSet::new();
                    ret.insert(SimpleSub::empty());
                    ret
                } else {
                    HashSet::new()
                }
            }
        },
        GenRegex::StringVar(s_var) => {
            let mut subs = HashSet::new();
            let mut string_to = BTreeMap::new();
            string_to.insert(s_var.as_ref().clone(), SubExpr::empty());
            let string_sub = SimpleSub::new(
                string_to,
                 BTreeMap::new()
            );
            subs.insert(string_sub);
            subs
        }
        GenRegex::Union(side1, side2) => {
            let left_null = nullable(&Rc::clone(side1));
            let right_null = nullable(&Rc::clone(side2));
            let unionLR: HashSet<_> = left_null.union(&right_null).cloned().collect();
            unionLR
        }
        GenRegex::Intersect(side1, side2) => {
            let left_null = nullable(&Rc::clone(side1));
            let right_null = nullable(&Rc::clone(side2));
            let mut retSet = HashSet::new();
            for left_elem in &left_null {
                for right_elem in &right_null {
                    let unionLR: AnySub = left_elem.clone().union(right_elem.clone());
                    let ret = merge(Rc::new(unionLR));
                    match ret {
                        MergeResult::SimpleSub(simple_sub)=>{
                            retSet.insert(simple_sub);
                        },
                        _=>{}
                    }
                }
            }
            retSet
        }
        GenRegex::Concatenation(side1, side2) => {
            let left_null = nullable(&Rc::clone(side1));
            let right_null = nullable(&Rc::clone(side2));
            let mut retSet = HashSet::new();
            for left_elem in &left_null {
                for right_elem in &right_null {
                    let unionLR: AnySub = left_elem.clone().union(right_elem.clone());
                    let ret = merge(Rc::new(unionLR));
                    match ret {
                        MergeResult::SimpleSub(simple_sub)=>{
                            retSet.insert(simple_sub);
                        },
                        _=>{}
                    }
                }
            }
            retSet
        }
        GenRegex::Kleene(_) => {
            let mut ret = HashSet::new();
            ret.insert(SimpleSub::empty());
            ret
        }
        _=>{
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
