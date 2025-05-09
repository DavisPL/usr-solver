//!
//! Predicate evaluation and manipulation functions
//!

// TODO: fix and remove
#![allow(clippy::single_match)]
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::types::expr::{CharExpression, CharVar, MaybeCharExpression, StringIndex, StringVar};
use crate::types::predicate::Predicate;

use disjoint_sets::UnionFind;

use std::collections::{HashMap, HashSet};
//use std::ffi::CStr;
use std::ffi::CString;
use std::rc::Rc;
use z3::Context;
use z3::Solver as OtherSolver;
use z3::*;
use z3_sys::*;
/*use z3::{
  ast::{self, Ast, Bool},
  Config, Context, Solver as Z3Solver,
};*/

fn is_char_var(mce: &Rc<MaybeCharExpression>) -> bool {
    if let MaybeCharExpression::CharExpression(c_expr) = mce.as_ref() {
        match c_expr {
            CharExpression::Literal(_) => false,
            CharExpression::CharVar(_) => true,
        }
    } else {
        false
    }
}

fn get_char_var(mce: &Rc<MaybeCharExpression>) -> Option<CharExpression> {
    if is_char_var(mce) {
        if let MaybeCharExpression::CharExpression(c_expr) = mce.as_ref() {
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

fn is_string_index(mce: &Rc<MaybeCharExpression>) -> bool {
    matches!(mce.as_ref(), MaybeCharExpression::StringIndex(_))
}

fn get_string_index(mce: &Rc<MaybeCharExpression>) -> Option<StringIndex> {
    if is_string_index(mce) {
        if let MaybeCharExpression::StringIndex(s_var) = mce.as_ref() {
            Some(s_var.clone())
        } else {
            None
        }
    } else {
        None
    }
}

/*pub fn flatten_and_predicates(pred: &Rc<Predicate>) -> Vec<Rc<Predicate>> {
    match pred.as_ref() {
        Predicate::And(left, right) => {
            let mut flattened: Vec<Rc<Predicate>> = Vec::new();
            flattened.push(convertToDNF(&Rc::clone(left)));
            flattened.push(convertToDNF(&Rc::clone(right)));
            flattened
        }
        _ => {
            vec![Rc::clone(pred)]
        }
    }
}*/

fn assign_unique_ids(
    predicate: &Predicate,
    id_map: &mut HashMap<MaybeCharExpression, i32>,
    next_id: &mut i32,
) {
    match predicate {
        Predicate::Equals(left, right) => {
            id_map.entry(left.as_ref().clone()).or_insert_with(|| {
                let id = *next_id;
                *next_id += 1;
                id
            });
            id_map.entry(right.as_ref().clone()).or_insert_with(|| {
                let id = *next_id;
                *next_id += 1;
                id
            });
        }
        Predicate::And(left, right) | Predicate::Or(left, right) => {
            // Recurse down for each sub-predicate in `And` or `Or` lists
            assign_unique_ids(left, id_map, next_id);
            assign_unique_ids(right, id_map, next_id);
        }
        Predicate::Not(sub_predicate) => {
            assign_unique_ids(sub_predicate, id_map, next_id);
        }
        _ => {}
    }
}

pub fn evaluate_complete(pred: &Rc<Predicate>) -> bool {
    /*unsafe {
        let cfg = Z3_mk_config();
        let ctx = Z3_mk_context(cfg);

        let solver = Z3_mk_solver(ctx);
        let mut mce_map: HashMap<MaybeCharExpression, Z3_ast> = HashMap::new();
        let mut svar_map: HashMap<StringVar, Z3_ast> = HashMap::new();
        let constraint = construct_constraint(pred, &ctx, &mut mce_map, &mut svar_map);

        Z3_solver_assert(ctx, solver, constraint);
        /*let smtlib_string = Z3_solver_to_string(ctx, solver);
        let smtlib_str = {
            let c_str = CStr::from_ptr(smtlib_string);

            c_str.to_str().unwrap()
        };
        println!("SMT-LIB Constraints:\n{}", smtlib_str);*/

        for (key, value) in mce_map.iter() {
            match key {
                MaybeCharExpression::CharExpression(_) => {
                    let len_var = Z3_mk_seq_length(ctx, *value);
                    let int_sort = Z3_mk_int_sort(ctx);
                    let length_ast = Z3_mk_int(ctx, 1, int_sort);
                    let new_constraint = Z3_mk_eq(ctx, len_var, length_ast);
                    Z3_solver_assert(ctx, solver, new_constraint);
                }
                _ => {} // Do nothing for other types
            }
        }

        // Check satisfiability
        let result = Z3_solver_check(ctx, solver);
        if result == Z3_L_TRUE {
            println!("Satisfiable");
          //  return true;
        } else if result == Z3_L_FALSE{
            println!("Unsatisfiable");
            //return false;
        }

        // Clean up
        //Z3_del_config(cfg);
        //Z3_del_context(ctx);
    }*/

    let mut c_vars: HashSet<CharVar> = HashSet::new();
    let mut s_vars: HashSet<StringVar> = HashSet::new();
    let pred_string = predicate_to_string(pred, &mut c_vars, &mut s_vars);
    let prefix = "";
    let mut string_decls = String::new();

    let mut assertions = String::new();

    for var in s_vars {
        string_decls.push_str(&("(declare-const ".to_string() + &var.name + "  String)"));
    }
    for var in c_vars {
        string_decls.push_str(&("(declare-const ".to_string() + &var.name + "  String)"));
        assertions.push_str(&("(assert (= (str.len ".to_string() + &var.name + ") 1))"));
    }
    let final_string =
        prefix.to_string() + &string_decls + &assertions + "(assert " + &pred_string + ")";
    let cfg = z3::Config::new();
    let context = Context::new(&cfg);
    //println!("{}", final_string);

    let solver = OtherSolver::new(&context);

    //println!("{}", final_string);
    solver.from_string(final_string); // Parse the string and add constraints
    match solver.check() {
        SatResult::Sat => true,
        SatResult::Unsat => false,
        SatResult::Unknown => false,
    }

    /*let mut id_map: HashMap<MaybeCharExpression, i32> = HashMap::new();
    //let mut canonical_map: HashMap<i32, i32> = HashMap::new();
    let mut next_id = 1;
    assign_unique_ids(pred, &mut id_map, &mut next_id);
    //let mut uf: UnionFind<usize> = UnionFind::new((next_id) as usize);
    //let predicate2 = convertToCNF(pred);
    //for inner_vec in &predicate2 {
    //   for inner in inner_vec {
    //      print!("{}", inner);
    // }
    //println!("");
    //}
    let predicate = convertToDNF(pred);
    //let uf = &mut UnionFind2::new();
    evaluate(&predicate, &mut id_map /*&mut canonical_map*/)*/
}
pub fn cnf_to_string(
    pred: Vec<Vec<Rc<Predicate>>>,
    variables: &mut HashSet<CharVar>,
    s_variables: &mut HashSet<StringVar>,
) -> String {
    let mut final_string = "(and ".to_string();
    for disjunct in pred {
        let mut disjunct_string = "(or ".to_string();
        for p in disjunct {
            disjunct_string.push_str(&predicate_to_string(&p, variables, s_variables));
        }
        disjunct_string.push(')');
        final_string.push_str(&disjunct_string);
    }
    final_string.push(')');
    final_string
}

pub fn construct_constraint(
    pred: &Rc<Predicate>,
    ctx: &Z3_context,
    mce_map: &mut HashMap<MaybeCharExpression, Z3_ast>,
    svar_map: &mut HashMap<StringVar, Z3_ast>,
) -> Z3_ast {
    unsafe {
        match pred.as_ref() {
            Predicate::Not(inner) => {
                Z3_mk_not(*ctx, construct_constraint(inner, ctx, mce_map, svar_map))
            }
            Predicate::And(left, right) => {
                let assertions: [Z3_ast; 2] = [
                    construct_constraint(left, ctx, mce_map, svar_map),
                    construct_constraint(right, ctx, mce_map, svar_map),
                ];
                Z3_mk_and(*ctx, 2, assertions.as_ptr())
            }
            Predicate::Or(left, right) => {
                let assertions: [Z3_ast; 2] = [
                    construct_constraint(left, ctx, mce_map, svar_map),
                    construct_constraint(right, ctx, mce_map, svar_map),
                ];
                Z3_mk_or(*ctx, 2, assertions.as_ptr())
            }
            Predicate::False => Z3_mk_false(*ctx),
            Predicate::True => Z3_mk_true(*ctx),
            Predicate::Equals(left, right) => {
                let left_ast = maybe_char_expr_to_ast(left, ctx, mce_map, svar_map);
                let right_ast = maybe_char_expr_to_ast(right, ctx, mce_map, svar_map);
                Z3_mk_eq(*ctx, left_ast, right_ast)
            }
            Predicate::EqualLength(s_var, length) => {
                let var_ast = stringvar_to_ast(s_var, ctx, svar_map);
                let len_var = Z3_mk_seq_length(*ctx, var_ast);
                let int_sort = Z3_mk_int_sort(*ctx);
                let length_ast = Z3_mk_int(*ctx, *length, int_sort);
                Z3_mk_eq(*ctx, len_var, length_ast)
            }
            Predicate::LessThan(c_var, bound) => {
                let left_ast = maybe_char_expr_to_ast(c_var, ctx, mce_map, svar_map);
                let lo_str = CString::new(" ").expect("CString::new failed");
                let lo_ast = Z3_mk_string(*ctx, lo_str.as_ptr());
                let c_str = CString::new(bound.to_string()).expect("CString::new failed");
                let high_ast = Z3_mk_string(*ctx, c_str.as_ptr());
                let reg = Z3_mk_re_range(*ctx, lo_ast, high_ast);
                Z3_mk_seq_in_re(*ctx, left_ast, reg)

                //todo!()
            }
            _ => unimplemented!(),
        }
    }
}

pub fn predicate_to_string(
    pred: &Rc<Predicate>,
    variables: &mut HashSet<CharVar>,
    s_variables: &mut HashSet<StringVar>,
) -> String {
    match pred.as_ref() {
        Predicate::Not(inner) => {
            let inside = predicate_to_string(inner, variables, s_variables);
            "(not ".to_string() + &inside + ")"
        }
        Predicate::And(left, right) => {
            let left_side = predicate_to_string(left, variables, s_variables);
            let right_side = predicate_to_string(right, variables, s_variables);
            "(and ".to_string() + &left_side + " " + &right_side + ")"
        }
        Predicate::Or(left, right) => {
            let left_side = predicate_to_string(left, variables, s_variables);
            let right_side = predicate_to_string(right, variables, s_variables);
            "(or ".to_string() + &left_side + " " + &right_side + ")"
        }
        Predicate::False => "false".to_string(),
        Predicate::True => "true".to_string(),
        Predicate::Equals(left, right) => {
            let left_op = mce_to_string(left, variables, s_variables);
            let right_op = mce_to_string(right, variables, s_variables);
            "(= ".to_string() + &left_op + " " + &right_op + ")"
        }
        Predicate::EqualLength(s_var, length) => {
            s_variables.insert(s_var.as_ref().clone());
            let str_op = s_var.name.clone();
            "(= (str.len ".to_string() + &str_op + ") " + &length.to_string() + ")"
        }
        Predicate::LessThan(c_var, bound) => {
            let cvar_ast = mce_to_string(c_var, variables, s_variables);
            //return "true".to_string();
            //"(= (< (extract 2 3 s) "b") true)"
            "(str.in_re ".to_string()
                + &cvar_ast
                + " (re.range \" \" \""
                + &bound.to_string()
                + "\"))"
            //todo!()
        }
        Predicate::GreaterThan(c_var, bound) => {
            let cvar_ast = mce_to_string(c_var, variables, s_variables);
            "not((str.<= ".to_string() + &cvar_ast + " \"" + &bound.to_string() + "\"))"
            //todo!()
        } //_ => unimplemented!(),
    }
}

pub fn mce_to_string(
    mce: &Rc<MaybeCharExpression>,
    variables: &mut HashSet<CharVar>,
    s_variables: &mut HashSet<StringVar>,
) -> String {
    match mce.as_ref() {
        MaybeCharExpression::StringIndex(s_ind) => {
            s_variables.insert(s_ind.var.clone());
            "(str.at ".to_string() + &s_ind.var.name + " " + &s_ind.index.to_string() + ")"
        }
        MaybeCharExpression::CharExpression(c_expr) => {
            match c_expr {
                CharExpression::CharVar(c_var) => {
                    variables.insert(c_var.clone());
                    c_var.name.clone()
                    //todo!()
                }
                CharExpression::Literal(literal) => "\"".to_string() + &literal.to_string() + "\"",
            }
        }
    }
}

pub fn maybe_char_expr_to_ast(
    variable: &Rc<MaybeCharExpression>,
    ctx: &Z3_context,
    mce_map: &mut HashMap<MaybeCharExpression, Z3_ast>,
    svar_map: &mut HashMap<StringVar, Z3_ast>,
) -> Z3_ast {
    if mce_map.contains_key(variable) {
        return mce_map[variable];
    }
    unsafe {
        match variable.as_ref() {
            MaybeCharExpression::StringIndex(s_ind) => {
                //let symbol_t = CString::new((s_ind.var.to_string()).clone()).expect("CString::new failed");

                //let symbol = Z3_mk_string_symbol(*ctx, symbol_t.as_ptr());
                let str_var = stringvar_to_ast(&s_ind.var, ctx, svar_map);
                let int_sort = Z3_mk_int_sort(*ctx);
                let index = Z3_mk_int(*ctx, s_ind.index, int_sort);
                let str_ind = Z3_mk_seq_at(*ctx, str_var, index);
                mce_map.insert(variable.as_ref().clone(), str_ind);
                str_ind
            }
            MaybeCharExpression::CharExpression(c_expr) => match c_expr {
                CharExpression::CharVar(c_var) => {
                    let symbol_t = CString::new(c_var.name.clone()).expect("CString::new failed");
                    let symbol = Z3_mk_string_symbol(*ctx, symbol_t.as_ptr());

                    let str_sort = Z3_mk_string_sort(*ctx);
                    let char_var = Z3_mk_const(*ctx, symbol, str_sort);
                    mce_map.insert(variable.as_ref().clone(), char_var);
                    char_var
                    /*let bv_sort = Z3_mk_bv_sort(*ctx, 8);
                    let symbol_t = CString::new(c_var.name.clone()).expect("CString::new failed");

                    let symbol = Z3_mk_string_symbol(*ctx, symbol_t.as_ptr());
                    let char_var = Z3_mk_const(*ctx, symbol, bv_sort);
                    mce_map.insert(variable.as_ref().clone(), char_var);
                    char_var*/
                }
                CharExpression::Literal(literal) => {
                    /*let code_point = *literal as u32;
                    let int_sort = Z3_mk_int_sort(*ctx);
                    let code_point_ast = Z3_mk_int(*ctx, code_point as i32, int_sort);

                    let char_ast = Z3_mk_int2bv(*ctx, 8, code_point_ast);*/
                    /*let bv_sort = Z3_mk_bv_sort(*ctx, 8);
                    let char_code = literal as u32;
                    let char_ast = Z3_mk_bv_numeral(*ctx, char_code, bv_sort);*/

                    let c_str = CString::new(literal.to_string()).expect("CString::new failed");
                    let char_ast = Z3_mk_string(*ctx, c_str.as_ptr());
                    mce_map.insert(variable.as_ref().clone(), char_ast);
                    char_ast
                }
            },
        }
    }
}

pub fn stringvar_to_ast(
    variable: &StringVar,
    ctx: &Z3_context,
    map: &mut HashMap<StringVar, Z3_ast>,
) -> Z3_ast {
    if map.contains_key(variable) {
        return map[variable];
    }
    unsafe {
        let symbol_t = CString::new(variable.name.clone()).expect("CString::new failed");
        let symbol = Z3_mk_string_symbol(*ctx, symbol_t.as_ptr());

        let str_sort = Z3_mk_string_sort(*ctx);
        let str_var = Z3_mk_const(*ctx, symbol, str_sort);
        map.insert(variable.clone(), str_var);
        str_var
    }

    //todo!()
}

pub fn evaluate_conjunction(
    all_preds: &Vec<Rc<Predicate>>,
    union_find: &mut UnionFind,
    id_map: &mut HashMap<MaybeCharExpression, i32>,
    lit_map: &mut HashMap<i32, char>,
    map: &mut HashMap<i32, i32>,
) -> Vec<Rc<Predicate>> {
    let mut final_preds = Vec::new();
    let mut not_equality_preds = HashSet::new();
    let mut length_preds: HashMap<String, i32> = HashMap::new();
    let not_allowed_lengths: HashMap<String, HashSet<i32>> = HashMap::new();
    let mut char_ranges: HashMap<MaybeCharExpression, (Option<char>, Option<char>)> =
        HashMap::new();

    let mut equalities = HashSet::new();

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
                        return vec![Rc::new(Predicate::False)];
                    }
                }
                length_preds.insert(var.name.clone(), *length);
            }
            Predicate::False => {
                return vec![Rc::new(Predicate::False)];
                //return Rc::new(Predicate::False);
            }
            Predicate::LessThan(var, end) => {
                let entry = char_ranges
                    .entry(var.as_ref().clone())
                    .or_insert((None, None));
                if entry.1.is_none() || *end < entry.1.unwrap() {
                    if let Some(start) = entry.0 {
                        if start > *end {
                            return vec![Rc::new(Predicate::False)];
                        }
                        entry.1 = Some(*end);
                    } else {
                        entry.1 = Some(*end);
                    }
                }
            }
            Predicate::GreaterThan(var, start) => {
                let entry = char_ranges
                    .entry(var.as_ref().clone())
                    .or_insert((None, None));
                if entry.0.is_none() || *start > entry.0.unwrap() {
                    if let Some(end) = entry.0 {
                        if *start > end {
                            return vec![Rc::new(Predicate::False)];
                        }
                        entry.0 = Some(*start);
                    } else {
                        entry.0 = Some(*start);
                    }
                }
            }
            _ => {}
        }
    }
    for p in equalities {
        let leftId;
        let rightId;
        if let Predicate::Equals(left, right) = p.as_ref() {
            leftId = id_map[left];
            rightId = id_map[right];
            match left.as_ref() {
                MaybeCharExpression::CharExpression(CharExpression::Literal(lit_char)) => {
                    map.insert(leftId, leftId);
                    lit_map.insert(leftId, *lit_char);
                }
                _ => {}
            }
            match right.as_ref() {
                MaybeCharExpression::CharExpression(CharExpression::Literal(lit_char)) => {
                    map.insert(rightId, rightId);
                    lit_map.insert(rightId, *lit_char);
                }
                _ => {}
            }
            if leftId != rightId {
                final_preds.push(p.clone());
            }
            let uf_left = union_find.find(leftId as usize) as i32;
            let uf_right = union_find.find(rightId as usize) as i32;
            if let (Some(value1), Some(value2)) = (map.get(&uf_left), map.get(&uf_right)) {
                if value1 != value2 {
                    return vec![Rc::new(Predicate::False)];
                } else {
                    union_find.union(leftId as usize, rightId as usize);
                }
            } else if let Some(value1) = map.get(&uf_left) {
                union_find.union(leftId as usize, rightId as usize);
                let new_canon = union_find.find(leftId as usize);
                map.insert(new_canon as i32, *value1);
            } else if let Some(value1) = map.get(&uf_right) {
                union_find.union(leftId as usize, rightId as usize);
                let new_canon = union_find.find(leftId as usize);
                map.insert(new_canon as i32, *value1);
            } else {
                union_find.union(leftId as usize, rightId as usize);
            }
        }
    }
    for (var, (start, end)) in &char_ranges {
        let var_id = union_find.find(id_map[var] as usize);

        if let Some(lit_id) = map.get(&(var_id as i32)) {
            let lit = lit_map[lit_id];
            if let (Some(s), Some(e)) = (start, end) {
                if lit < *s || lit > *e {
                    return vec![Rc::new(Predicate::False)];
                } else {
                    continue;
                }
            } else {
                if let Some(s) = start {
                    if lit < *s {
                        return vec![Rc::new(Predicate::False)];
                    }
                }

                if let Some(e) = end {
                    if lit > *e {
                        return vec![Rc::new(Predicate::False)];
                    }
                }

                continue;
            }
        } else {
            // If the value is not in the map, create a summarized predicate with start and end
            if let (Some(s), Some(e)) = (start, end) {
                let summarized = Rc::new(Predicate::And(
                    Rc::new(Predicate::LessThan(Rc::new(var.clone()), *e)),
                    Rc::new(Predicate::GreaterThan(Rc::new(var.clone()), *s)),
                ));
                final_preds.push(summarized);
            } else {
                if let Some(s) = start {
                    let summarized = Rc::new(Predicate::GreaterThan(Rc::new(var.clone()), *s));
                    final_preds.push(summarized);
                }
                if let Some(e) = end {
                    let summarized = Rc::new(Predicate::LessThan(Rc::new(var.clone()), *e));
                    final_preds.push(summarized);
                }
                continue;
            }
        }
    }

    // let cant_equal_chars: HashMap<String, HashSet<String>> = HashMap::new();
    for not_pred in not_equality_preds {
        if let Predicate::Not(inner) = not_pred.as_ref() {
            let leftId;
            let rightId;
            if let Predicate::Equals(left, right) = inner.as_ref() {
                //Think this needs to
                //be redone?
                leftId = id_map[left];
                rightId = id_map[right];
                if leftId == rightId
                    || union_find.find(leftId as usize) == union_find.find(rightId as usize)
                {
                    return vec![Rc::new(Predicate::False)];
                    //return Rc::new(Predicate::False);
                }
                if let (Some(_), Some(_)) = (map.get(&leftId), map.get(&rightId)) {
                    final_preds.push(not_pred.clone())
                }
            } else if let Predicate::EqualLength(var_name, length) = &**inner {
                //let Predicate::EqualLength(var_name, length) = &**inner;
                if let Some(temp) = length_preds.get(&var_name.name) {
                    if *temp == *length {
                        return vec![Rc::new(Predicate::False)];
                    }
                }
                let mut flag = false;
                for (key, value) in id_map.iter() {
                    if is_string_index(&Rc::new(key.clone())) {
                        //if key.starts_with("StringIndex") {
                        let str_ind = get_string_index(&Rc::new(key.clone())).expect("string ind"); //Maybe should rewrite? TODO: check clarity of predicate evaluation
                        if str_ind.var.name == var_name.name.clone()
                            && str_ind.index >= *length
                            && union_find.find(*value as usize) != *value as usize
                        {
                            flag = true;
                            break;
                        }
                    }
                }
                if flag {
                    continue;
                }

                final_preds.push(not_pred.clone());
            }
        }
    }
    /*for (_, chars) in cant_equal_chars {
        if alphabet.iter().all(|c| chars.contains(c)) {
            return vec![Rc::new(Predicate::False)];
        }
    }*/

    for (var_name, length) in length_preds {
        if not_allowed_lengths
            .get(&var_name)
            .is_some_and(|lengths| lengths.contains(&length))
        {
            return vec![Rc::new(Predicate::False)];
            //return Rc::new(Predicate::False);
        }
        for (key, value) in id_map.iter() {
            if is_string_index(&Rc::new(key.clone())) {
                let str_ind = get_string_index(&Rc::new(key.clone())).expect("string ind");
                if str_ind.var.name == var_name.clone()
                    && str_ind.index >= length
                    && union_find.find(*value as usize) != *value as usize
                {
                    return vec![Rc::new(Predicate::False)];
                    //return Rc::new(Predicate::False);
                }
            }
        }
        let string_var = Rc::new(StringVar { name: var_name });
        final_preds.push(Rc::new(Predicate::EqualLength(string_var, length)));
    }
    match final_preds.len() {
        0 => vec![Rc::new(Predicate::True)],
        //1 => vec![final_preds[0].clone(),
        _ => final_preds,
    }
}
fn evaluate(
    pred: &Vec<Vec<Rc<Predicate>>>,
    id_map: &mut HashMap<MaybeCharExpression, i32>,
    //string_map: &mut HashMap<i32, String>,
    //map: &mut HashMap<i32, i32>,
) -> Vec<Vec<Rc<Predicate>>> {
    //let uf = union_find.unwrap_or_else(|| UnionFind::new());

    let mut final_set = Vec::new();
    for conjunct in pred {
        let mut canonical_map: HashMap<i32, i32> = HashMap::new();
        let mut lit_map: HashMap<i32, char> = HashMap::new();
        let mut uf: UnionFind<usize> = UnionFind::new(id_map.len() + 1);
        let p_eval = evaluate_conjunction(
            &conjunct.clone(),
            &mut uf,
            id_map,
            &mut lit_map,
            &mut canonical_map,
        );
        if !p_eval.is_empty() {
            match p_eval[0].as_ref() {
                Predicate::True => {
                    return vec![vec![Rc::new(Predicate::True)]];
                }
                Predicate::False => {
                    continue;
                }
                _ => {
                    return vec![vec![Rc::new(Predicate::True)]];
                }
            }
        }
        final_set.push(p_eval);
        /*match &*p_eval {
            Predicate::True => return Rc::new(Predicate::True),
            Predicate::False => {
                continue;
            }
            _ => final_set.push(p_eval),
        }*/
    }
    match final_set.len() {
        0 => vec![vec![Rc::new(Predicate::False)]],
        //1 => final_set[0].clone(),
        _ => final_set,
    }
}

pub fn internalize_all_nots(pred: &Rc<Predicate>) -> Rc<Predicate> {
    match pred.as_ref() {
        Predicate::Or(left, right) => Rc::new(Predicate::Or(
            internalize_all_nots(left),
            internalize_all_nots(right),
        )),

        Predicate::And(left, right) => Rc::new(Predicate::And(
            internalize_all_nots(left),
            internalize_all_nots(right),
        )),

        Predicate::Not(sub_pred) => match sub_pred.as_ref() {
            Predicate::And(left, right) => Rc::new(Predicate::Or(
                internalize_all_nots(&Rc::new(Predicate::Not(Rc::clone(left)))),
                internalize_all_nots(&Rc::new(Predicate::Not(Rc::clone(right)))),
            )),

            Predicate::Or(left, right) => Rc::new(Predicate::And(
                internalize_all_nots(&Rc::new(Predicate::Not(Rc::clone(left)))),
                internalize_all_nots(&Rc::new(Predicate::Not(Rc::clone(right)))),
            )),

            Predicate::Not(sub) => internalize_all_nots(sub),

            Predicate::True => Rc::new(Predicate::False),
            Predicate::False => Rc::new(Predicate::True),
            Predicate::LessThan(var, val) => Rc::new(Predicate::GreaterThan(
                var.clone(),
                char::from_u32(*val as u32 + 1).expect("Invalid char after subtraction"),
            )),
            Predicate::GreaterThan(var, val) => Rc::new(Predicate::LessThan(
                var.clone(),
                char::from_u32(*val as u32 - 1).expect("Invalid char after subtraction"),
            )),

            _ => Rc::clone(pred),
        },

        _ => Rc::clone(pred),
    }
}

pub fn dnf_helper(pred: &Rc<Predicate>) -> Vec<Vec<Rc<Predicate>>> {
    match pred.as_ref() {
        Predicate::Or(left, right) => {
            let mut left_dnf = dnf_helper(left);
            let right_dnf = dnf_helper(right);
            left_dnf.extend(right_dnf);
            left_dnf
            //Rc::new(Predicate::Or(flattened))
        }

        Predicate::And(left, right) => {
            let left_dnf = dnf_helper(left);
            let right_dnf = dnf_helper(right);
            let mut ret: Vec<Vec<Rc<Predicate>>> = vec![];
            for l in left_dnf {
                for r in &right_dnf {
                    ret.push([l.as_slice(), r.as_slice()].concat());
                }
            }
            ret
        }

        _ => vec![vec![pred.clone()]],
    }
}
pub fn convertToDNF(pred: &Rc<Predicate>) -> Vec<Vec<Rc<Predicate>>> {
    // Maybe Change this to a
    // Vec<Vec<Predicate>>
    let internalized = internalize_all_nots(pred);
    dnf_helper(&internalized)
}

pub fn cnf_helper(pred: &Rc<Predicate>) -> Vec<Vec<Rc<Predicate>>> {
    match pred.as_ref() {
        Predicate::Or(left, right) => {
            let left_cnf = cnf_helper(left);
            let right_cnf = cnf_helper(right);
            let mut result = vec![];
            for l in left_cnf {
                for r in &right_cnf {
                    result.push([l.as_slice(), r.as_slice()].concat());
                }
            }
            result
        }

        Predicate::And(left, right) => {
            let mut left_cnf = cnf_helper(left);
            let right_cnf = cnf_helper(right);
            left_cnf.extend(right_cnf);
            left_cnf
        }

        _ => vec![vec![pred.clone()]],
    }
}

pub fn convertToCNF(pred: &Rc<Predicate>) -> Vec<Vec<Rc<Predicate>>> {
    // Maybe Change this to a
    // Vec<Vec<Predicate>>
    let internalized = internalize_all_nots(pred);
    cnf_helper(&internalized)
}

/*fn evaluate_cnf(
    pred: &Vec<Vec<Rc<Predicate>>>,
    id_map: &mut HashMap<MaybeCharExpression, i32>,
    //string_map: &mut HashMap<i32, String>,
    //map: &mut HashMap<i32, i32>,
) -> Vec<Vec<Rc<Predicate>>> {

    let mut canonical_map: HashMap<i32, i32> = HashMap::new();
    let mut lit_map: HashMap<i32, char> = HashMap::new();
    let mut uf: UnionFind<usize> = UnionFind::new(id_map.len() + 1);
    let mut final_preds = Vec::new();
    let mut not_equality_preds = HashSet::new();
    let mut length_preds: HashMap<String, i32> = HashMap::new();
    let not_allowed_lengths: HashMap<String, HashSet<i32>> = HashMap::new();

    for disjunct in pred {
        if (disjunct.len() == 1){
            match pred.as_ref() {
                Predicate::Equals(left, right) =>{
                    let leftId;
                    let rightId;
                    leftId = id_map[left];
                    rightId = id_map[right];
                    match left.as_ref() {
                        MaybeCharExpression::CharExpression(CharExpression::Literal(lit_char)) => {
                            map.insert(leftId, leftId);
                            lit_map.insert(leftId, *lit_char);
                        }
                        _ => {}
                    }
                    match right.as_ref() {
                        MaybeCharExpression::CharExpression(CharExpression::Literal(lit_char)) => {
                            map.insert(rightId, rightId);
                            lit_map.insert(rightId, *lit_char);
                        }
                        _ => {}
                    }
                    if leftId != rightId {
                        final_preds.push(p.clone());
                    }
                    let uf_left = union_find.find(leftId as usize) as i32;
                    let uf_right = union_find.find(rightId as usize) as i32;
                    if let (Some(value1), Some(value2)) = (map.get(&uf_left), map.get(&uf_right)) {
                        if value1 != value2 {
                            return vec![Rc::new(Predicate::False)];
                        } else {
                            union_find.union(leftId as usize, rightId as usize);
                        }
                    } else if let Some(value1) = map.get(&uf_left) {
                        union_find.union(leftId as usize, rightId as usize);
                        let new_canon = union_find.find(leftId as usize);
                        map.insert(new_canon as i32, *value1);
                    } else if let Some(value1) = map.get(&uf_right) {
                        union_find.union(leftId as usize, rightId as usize);
                        let new_canon = union_find.find(leftId as usize);
                        map.insert(new_canon as i32, *value1);
                    } else {
                        union_find.union(leftId as usize, rightId as usize);
                    }



                }
                Predicate::EqualLength(string_var, length)
                Predicate::Not(inner) => {
                    match inner.as_ref() {
                        Predicate::Equals(left, right)
                        Predicate::EqualLength(string_var, length)
                    }
                }

            }
        }
    }
}*/
