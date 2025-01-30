//!
//! Implementation of the Brzozowski Derivative
//!

// TODO: fix and remove
#![allow(unused_variables)]

use crate::classes::{
    CharExpression, CharVar, GenRegex, MaybeCharExpression, Predicate, StringIndex,
};
use crate::predicate_evaluation::evaluate_complete;
use std::collections::HashSet;
use std::rc::Rc;

/*pub fn string_var_normalization(gre: &Rc<GenRegex>) -> Rc<GenRegex>{
    match gre.as_ref(){
        GenRegex::Union(left, right) =>{
            Rc::new(GenRegex::Union(string_var_normalization(left), string_var_normalization(right)))
        },
        GenRegex::Intersect(left, right) =>{
            Rc::new(GenRegex::Intersect(string_var_normalization(left), string_var_normalization(right)))
        },
        GenRegex::Concatenation(left, right) =>{
            Rc::new(GenRegex::Concatenation(string_var_normalization(left), string_var_normalization(right)))
        },
        GenRegex::Kleene(inner) =>{
             Rc::new(GenRegex::Kleene(string_var_normalization(inner)))
        },
        GenRegex::Complement(inner) =>{
            Rc::new(GenRegex::Complement(string_var_normalization(inner)))
        },
        GenRegex::IfThenElse(pred, left, right)=>{
            Rc::new(GenRegex::IfThenElse(Rc::clone(pred), string_var_normalization(left), string_var_normalization(right)))
        },
        GenRegex::StringIndex(str_ind) => {
            Rc::new(GenRegex::StringVar(str_ind.var.clone()))
        },
        GenRegex::StringSlice(str_var, _) =>{
            Rc::new(GenRegex::StringVar(str_var.clone()))
        },
        _ => {
            gre.clone()
        }
    }
}
pub fn simplify_and_check_cycles(gre: &Rc<GenRegex>, visited: &mut HashSet<Rc<GenRegex>>) -> Rc<GenRegex>{
    match gre.as_ref(){
        GenRegex::IfThenElse(pred, left, right)=>{
            Rc::new(GenRegex::IfThenElse(pred.clone(), simplify_and_check_cycles(left, visited), simplify_and_check_cycles(right, visited)))
        }
        GenRegex::Union(left, right) => {
            Rc::new(GenRegex::Union(simplify_and_check_cycles(left, visited), simplify_and_check_cycles(right, visited)))
        }
        _ => {
            let temp = string_var_normalization(gre);
            println!("temp {}", temp);
            if visited.contains(&temp){
                println!("_----------------------------------------");
                return GenRegex::empty_set()
            }
            visited.insert(temp);
            gre.clone()
        }

    }
}*/
pub fn satisfiable_helper(
    gre: &Rc<GenRegex>,
    index: &mut i32,
    visited: &mut HashSet<Rc<GenRegex>>,
) -> bool {
    println!("Checking sat: {} (index {})", gre, index);
    //let mut expr;
    match gre.as_ref() {
        GenRegex::IfThenElse(pred, left, right) => {
            let mut temp_left = left.clone();
            let mut temp_right = right.clone();
            if visited.contains(left) {
                temp_left = GenRegex::empty_set();
            } else {
                visited.insert(left.clone());
            }
            if visited.contains(right) {
                temp_right = GenRegex::empty_set();
            } else {
                visited.insert(right.clone());
            }
            let expr = &simplifies(&Rc::new(GenRegex::IfThenElse(
                pred.clone(),
                temp_left,
                temp_right,
            )));
            if matches!(nullable_projection(expr)[0][0].as_ref(), Predicate::False) {
                let new_name = "f".to_owned() + &index.to_string();
                let c_var = Rc::new(CharExpression::CharVar(CharVar { name: new_name }));
                let deriv = simplifies(&derivative(expr, &c_var));
                *index += 1;
                return satisfiable_helper(&deriv, index, visited);
            }
            true
        }
        GenRegex::Union(left, right) => {
            let mut temp_left = left.clone();
            let mut temp_right = right.clone();
            if visited.contains(left) {
                temp_left = GenRegex::empty_set();
            }
            if visited.contains(right) {
                temp_right = GenRegex::empty_set();
            }
            let expr = &simplifies(&Rc::new(GenRegex::Union(temp_left, temp_right)));
            if matches!(nullable_projection(expr)[0][0].as_ref(), Predicate::False) {
                let new_name = "f".to_owned() + &index.to_string();
                let c_var = Rc::new(CharExpression::CharVar(CharVar { name: new_name }));
                let deriv = simplifies(&derivative(expr, &c_var));
                *index += 1;
                return satisfiable_helper(&deriv, index, visited);
            }
            true
        }
        GenRegex::EmptySet => false,
        _ => {
            if visited.contains(gre) {
                return false;
            } else {
                visited.insert(gre.clone());
            }
            let expr = &simplifies(gre);
            if matches!(nullable_projection(expr)[0][0].as_ref(), Predicate::False) {
                let new_name = "f".to_owned() + &index.to_string();
                let c_var = Rc::new(CharExpression::CharVar(CharVar { name: new_name }));
                let deriv = simplifies(&derivative(expr, &c_var));
                *index += 1;
                return satisfiable_helper(&deriv, index, visited);
            }
            true
        }
    }
}

pub fn satisfiable(gre: &Rc<GenRegex>) -> bool {
    let mut ind = 0;
    satisfiable_helper(gre, &mut ind, &mut HashSet::new())
}

pub fn derivative(gre: &Rc<GenRegex>, deriv_char: &Rc<CharExpression>) -> Rc<GenRegex> {
    println!("taking d({}, {})", gre, deriv_char);
    match gre.as_ref() {
        GenRegex::EmptySet => GenRegex::empty_set(),
        GenRegex::Epsilon => GenRegex::empty_set(),
        GenRegex::Sigma => GenRegex::epsilon(),
        GenRegex::Range(start, end) => simplifies(&Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::And(
                Rc::new(Predicate::GreaterThan(
                    Rc::new(MaybeCharExpression::CharExpression(
                        deriv_char.as_ref().clone(),
                    )),
                    *start,
                )),
                Rc::new(Predicate::LessThan(
                    Rc::new(MaybeCharExpression::CharExpression(
                        deriv_char.as_ref().clone(),
                    )),
                    *end,
                )),
            )),
            GenRegex::epsilon(),
            GenRegex::empty_set(),
        ))),
        GenRegex::CharExpression(c_expr) => simplifies(&Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::Equals(
                Rc::new(MaybeCharExpression::CharExpression(c_expr.clone())),
                Rc::new(MaybeCharExpression::CharExpression(
                    deriv_char.as_ref().clone(),
                )),
            )),
            GenRegex::epsilon(),
            GenRegex::empty_set(),
        ))),
        GenRegex::StringVar(s_var) => simplifies(&Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::Equals(
                Rc::new(MaybeCharExpression::StringIndex(StringIndex {
                    var: s_var.clone(),
                    index: 0,
                })),
                Rc::new(MaybeCharExpression::CharExpression(
                    deriv_char.as_ref().clone(),
                )),
            )),
            Rc::new(GenRegex::StringSlice(s_var.clone(), 1)),
            GenRegex::empty_set(),
        ))),
        GenRegex::StringSlice(string_var, index) => simplifies(&Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::Equals(
                Rc::new(MaybeCharExpression::StringIndex(StringIndex {
                    var: string_var.clone(),
                    index: *index,
                })),
                Rc::new(MaybeCharExpression::CharExpression(
                    deriv_char.as_ref().clone(),
                )),
            )),
            Rc::new(GenRegex::StringSlice(string_var.clone(), index + 1)),
            GenRegex::empty_set(),
        ))),
        GenRegex::Union(side1, side2) => simplifies(&Rc::new(GenRegex::Union(
            Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
            Rc::clone(&derivative(&Rc::clone(side2), deriv_char)),
        ))),
        GenRegex::Intersect(side1, side2) => simplifies(&Rc::new(GenRegex::Intersect(
            Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
            Rc::clone(&derivative(&Rc::clone(side2), deriv_char)),
        ))),
        GenRegex::Concatenation(side1, side2) => {
            let left_side = Rc::new(GenRegex::Concatenation(
                Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
                Rc::clone(side2),
            ));
            let right_side = Rc::new(GenRegex::Concatenation(
                Rc::clone(&nullable(&Rc::clone(side1))),
                Rc::clone(&derivative(&Rc::clone(side2), deriv_char)),
            ));

            simplifies(&Rc::new(GenRegex::Union(
                Rc::clone(&left_side),
                Rc::clone(&right_side),
            )))
        }
        GenRegex::Complement(side1) => simplifies(&Rc::new(GenRegex::Complement(Rc::clone(
            &derivative(&Rc::clone(side1), deriv_char),
        )))),
        GenRegex::Kleene(side1) => simplifies(&Rc::new(GenRegex::Concatenation(
            Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
            Rc::clone(gre),
        ))),
        GenRegex::IfThenElse(pred, side1, side2) => simplifies(&Rc::new(GenRegex::IfThenElse(
            Rc::clone(pred),
            Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
            Rc::clone(&derivative(&Rc::clone(side2), deriv_char)),
        ))),
        GenRegex::StringIndex(string_index) => simplifies(&Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::Equals(
                Rc::new(MaybeCharExpression::StringIndex(string_index.clone())),
                Rc::new(MaybeCharExpression::CharExpression(
                    deriv_char.as_ref().clone(),
                )),
            )),
            GenRegex::epsilon(),
            GenRegex::empty_set(),
        ))),
    }
}

pub fn nullable(gre: &Rc<GenRegex>) -> Rc<GenRegex> {
    match gre.as_ref() {
        GenRegex::EmptySet => GenRegex::empty_set(),
        GenRegex::Epsilon => GenRegex::epsilon(),
        GenRegex::Sigma => GenRegex::empty_set(),
        GenRegex::Range(_start, _end) => GenRegex::empty_set(),
        GenRegex::CharExpression(c_expr) => match c_expr {
            CharExpression::CharVar(_name) => GenRegex::empty_set(),
            CharExpression::Literal(value) => GenRegex::empty_set(),
        },
        GenRegex::StringSlice(string_var, index) => Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::EqualLength(Rc::new(string_var.clone()), *index)),
            GenRegex::epsilon(),
            GenRegex::empty_set(),
        )),
        GenRegex::StringVar(string_var) => Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::EqualLength(Rc::new(string_var.clone()), 0)),
            GenRegex::epsilon(),
            GenRegex::empty_set(),
        )),
        GenRegex::StringIndex(_string_index) => GenRegex::empty_set(),
        GenRegex::Union(side1, side2) => Rc::new(GenRegex::Union(
            Rc::clone(&nullable(&Rc::clone(side1))),
            Rc::clone(&nullable(&Rc::clone(side2))),
        )),
        GenRegex::Intersect(side1, side2) => Rc::new(GenRegex::Intersect(
            Rc::clone(&nullable(&Rc::clone(side1))),
            Rc::clone(&nullable(&Rc::clone(side2))),
        )),
        GenRegex::Concatenation(side1, side2) => Rc::new(GenRegex::Concatenation(
            Rc::clone(&nullable(&Rc::clone(side1))),
            Rc::clone(&nullable(&Rc::clone(side2))),
        )),
        GenRegex::Complement(side1) => Rc::new(GenRegex::Intersect(
            Rc::new(GenRegex::Complement(Rc::clone(&nullable(&Rc::clone(
                side1,
            ))))),
            GenRegex::epsilon(),
        )),
        GenRegex::Kleene(_) => GenRegex::epsilon(),
        GenRegex::IfThenElse(pred, side1, side2) => Rc::new(GenRegex::IfThenElse(
            Rc::clone(pred),
            Rc::clone(&nullable(&Rc::clone(side1))),
            Rc::clone(&nullable(&Rc::clone(side2))),
        )),
    }
}

fn nullable_projection_helper(expr: &Rc<GenRegex>) -> Rc<Predicate> {
    match expr.as_ref() {
        GenRegex::EmptySet => Rc::new(Predicate::False),
        GenRegex::Epsilon => Rc::new(Predicate::True),
        GenRegex::CharExpression(c_expr) => Rc::new(Predicate::False),
        GenRegex::IfThenElse(pred, true_expr, false_expr) => {
            let true_proj = nullable_projection_helper(true_expr);
            let false_proj = nullable_projection_helper(false_expr);
            //println!("{}", print_predicate(&true_proj));

            match (true_proj.as_ref(), false_proj.as_ref()) {
                (Predicate::False, Predicate::False) => Rc::new(Predicate::False),
                (Predicate::False, _) => Rc::new(Predicate::And(
                    Rc::new(Predicate::Not(Rc::clone(pred))),
                    Rc::clone(&false_proj),
                )),
                (_, Predicate::False) => {
                    Rc::new(Predicate::And(Rc::clone(pred), Rc::clone(&true_proj)))
                }
                _ => Rc::new(Predicate::Or(
                    Rc::new(Predicate::And(Rc::clone(pred), Rc::clone(&true_proj))),
                    Rc::new(Predicate::And(
                        Rc::new(Predicate::Not(Rc::clone(pred))),
                        Rc::clone(&false_proj),
                    )),
                )),
            }
        }

        GenRegex::Union(left, right) => {
            let left_proj = nullable_projection_helper(left);
            let right_proj = nullable_projection_helper(right);

            match (left_proj.as_ref(), right_proj.as_ref()) {
                (Predicate::False, _) => Rc::clone(&right_proj),
                (_, Predicate::False) => Rc::clone(&left_proj),
                _ => Rc::new(Predicate::Or(left_proj, right_proj)), //Rewrite as left and
                                                                    //right
            }
        }

        GenRegex::Intersect(left, right) | GenRegex::Concatenation(left, right) => {
            let left_proj = nullable_projection_helper(left);
            let right_proj = nullable_projection_helper(right);

            match (left_proj.as_ref(), right_proj.as_ref()) {
                (Predicate::False, _) | (_, Predicate::False) => Rc::new(Predicate::False),
                _ => Rc::new(Predicate::And(left_proj, right_proj)),
            }
        }

        GenRegex::Complement(inner) => Rc::new(Predicate::Not(nullable_projection_helper(inner))),

        GenRegex::Sigma => Rc::new(Predicate::False),
        GenRegex::StringVar(_) => Rc::new(Predicate::False),
        GenRegex::StringIndex(_) => Rc::new(Predicate::False),
        GenRegex::StringSlice(_, _) => Rc::new(Predicate::False),
        GenRegex::Kleene(_) => Rc::new(Predicate::True),
        GenRegex::Range(_, _) => Rc::new(Predicate::False),
    }
}
pub fn nullable_projection(gre: &Rc<GenRegex>) -> Vec<Vec<Rc<Predicate>>> {
    let nullable_gre = &nullable(gre);
    println!("{}", nullable_gre);
    let nullable_predicates = nullable_projection_helper(nullable_gre);
    println!("{}", nullable_predicates);
    evaluate_complete(&nullable_predicates)
    //nullable_predicatesEval
}

pub fn matching(gre: &Rc<GenRegex>, proposed: &str) -> bool {
    println!("the proposed string is {}", proposed);
    println!("the proposed gre is {}", gre);
    let expr = &simplifies(gre);
    if proposed.is_empty() {
        return !matches!(nullable_projection(expr)[0][0].as_ref(), Predicate::False);
    }
    let head = proposed.chars().next().unwrap();
    let tail = &proposed[1..];
    let literal = Rc::new(CharExpression::Literal(head));
    let deriv = derivative(expr, &literal);
    matching(&deriv, tail)
}

fn simplifies(gre: &Rc<GenRegex>) -> Rc<GenRegex> {
    match gre.as_ref() {
        GenRegex::Union(left_side, right_side) => simplify_union(left_side, right_side),
        GenRegex::Intersect(left_side, right_side) => simplify_intersect(left_side, right_side),
        GenRegex::Concatenation(left_side, right_side) => {
            simplify_concatenation(left_side, right_side)
        }
        GenRegex::IfThenElse(pred, true_branch, false_branch) => {
            simplify_if_then_else(pred, true_branch, false_branch)
        }
        GenRegex::Complement(inner) => GenRegex::complement(&simplifies(inner)),
        _ => Rc::clone(gre),
    }
}

fn simplify_union(left_side: &Rc<GenRegex>, right_side: &Rc<GenRegex>) -> Rc<GenRegex> {
    let left = simplifies(left_side);
    let right = simplifies(right_side);

    match (&*left, &*right) {
        (GenRegex::EmptySet, _) => right,
        (_, GenRegex::EmptySet) => left,
        (GenRegex::CharExpression(c1), GenRegex::CharExpression(c2)) => match (c1, c2) {
            (CharExpression::Literal(val1), CharExpression::Literal(val2)) if val1 == val2 => right,
            _ => Rc::new(GenRegex::Union(left, right)),
        },
        _ => Rc::new(GenRegex::Union(left, right)),
    }
}

fn simplify_intersect(left_side: &Rc<GenRegex>, right_side: &Rc<GenRegex>) -> Rc<GenRegex> {
    let left = simplifies(left_side);
    let right = simplifies(right_side);

    match (&*left, &*right) {
        (GenRegex::EmptySet, _) | (_, GenRegex::EmptySet) => GenRegex::empty_set(),
        (
            GenRegex::IfThenElse(pred1, true1, false1),
            GenRegex::IfThenElse(pred2, true2, false2),
        ) => simplify_intersect_if_then_else(pred1, true1, false1, pred2, true2, false2),
        _ => Rc::new(GenRegex::Intersect(left, right)),
    }
}

fn simplify_intersect_if_then_else(
    pred1: &Rc<Predicate>,
    true1: &Rc<GenRegex>,
    false1: &Rc<GenRegex>,
    pred2: &Rc<Predicate>,
    true2: &Rc<GenRegex>,
    false2: &Rc<GenRegex>,
) -> Rc<GenRegex> {
    match (false1.as_ref(), false2.as_ref()) {
        (GenRegex::EmptySet, GenRegex::EmptySet) => Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::And(Rc::clone(pred1), Rc::clone(pred2))),
            Rc::new(GenRegex::Intersect(Rc::clone(true1), Rc::clone(true2))),
            Rc::clone(false1),
        )),
        (_, GenRegex::EmptySet) => {
            create_complex_if_then_else(pred1, pred2, true1, true2, false1, false2, true)
        }
        (GenRegex::EmptySet, _) => {
            create_complex_if_then_else(pred2, pred1, true2, true1, false2, false1, false)
        }
        _ => create_full_intersect_if_then_else(pred1, pred2, true1, true2, false1, false2),
    }
}

fn create_complex_if_then_else(
    pred1: &Rc<Predicate>,
    pred2: &Rc<Predicate>,
    true1: &Rc<GenRegex>,
    true2: &Rc<GenRegex>,
    false1: &Rc<GenRegex>,
    false2: &Rc<GenRegex>,
    is_first_branch: bool,
) -> Rc<GenRegex> {
    let inner_pred = if is_first_branch {
        Rc::new(Predicate::And(
            Rc::clone(pred2),
            Rc::new(Predicate::Not(Rc::clone(pred1))),
        ))
    } else {
        Rc::new(Predicate::And(
            Rc::clone(pred1),
            Rc::new(Predicate::Not(Rc::clone(pred2))),
        ))
    };

    Rc::new(GenRegex::IfThenElse(
        Rc::new(Predicate::And(Rc::clone(pred1), Rc::clone(pred2))),
        Rc::new(GenRegex::Intersect(Rc::clone(true1), Rc::clone(true2))),
        Rc::new(GenRegex::IfThenElse(
            inner_pred,
            Rc::new(GenRegex::Intersect(
                if is_first_branch {
                    Rc::clone(false1)
                } else {
                    Rc::clone(false2)
                },
                if is_first_branch {
                    Rc::clone(true2)
                } else {
                    Rc::clone(true1)
                },
            )),
            if is_first_branch {
                Rc::clone(false2)
            } else {
                Rc::clone(false1)
            },
        )),
    ))
}

fn create_full_intersect_if_then_else(
    pred1: &Rc<Predicate>,
    pred2: &Rc<Predicate>,
    true1: &Rc<GenRegex>,
    true2: &Rc<GenRegex>,
    false1: &Rc<GenRegex>,
    false2: &Rc<GenRegex>,
) -> Rc<GenRegex> {
    Rc::new(GenRegex::IfThenElse(
        Rc::new(Predicate::And(Rc::clone(pred1), Rc::clone(pred2))),
        Rc::new(GenRegex::Intersect(Rc::clone(true1), Rc::clone(true2))),
        Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::And(
                Rc::clone(pred1),
                Rc::new(Predicate::Not(Rc::clone(pred2))),
            )),
            Rc::new(GenRegex::Intersect(Rc::clone(false2), Rc::clone(true1))),
            Rc::new(GenRegex::IfThenElse(
                Rc::new(Predicate::And(
                    Rc::clone(pred2),
                    Rc::new(Predicate::Not(Rc::clone(pred1))),
                )),
                Rc::new(GenRegex::Intersect(Rc::clone(false1), Rc::clone(true2))),
                GenRegex::empty_set(),
            )),
        )),
    ))
}

fn simplify_concatenation(left_side: &Rc<GenRegex>, right_side: &Rc<GenRegex>) -> Rc<GenRegex> {
    let left = simplifies(left_side);
    let right = simplifies(right_side);

    match (&*left, &*right) {
        (GenRegex::EmptySet, _) | (_, GenRegex::EmptySet) => GenRegex::empty_set(),
        (GenRegex::Epsilon, _) => right,
        (_, GenRegex::Epsilon) => left,
        _ => Rc::new(GenRegex::Concatenation(left, right)),
    }
}

fn simplify_if_then_else(
    pred: &Rc<Predicate>,
    true_branch: &Rc<GenRegex>,
    false_branch: &Rc<GenRegex>,
) -> Rc<GenRegex> {
    let simplified_true = simplifies(true_branch);
    let simplified_false = simplifies(false_branch);
    //println!("true {}, false {}", simplified_true, simplified_false);

    if let (GenRegex::EmptySet, GenRegex::EmptySet) = (&*simplified_true, &*simplified_false) {
        return simplified_true;
    };
    if let (GenRegex::IfThenElse(inner_pred, inner_true, inner_false), GenRegex::EmptySet) =
        (&*simplified_true, &*simplified_false)
    {
        if let GenRegex::EmptySet = inner_false.as_ref() {
            return Rc::new(GenRegex::IfThenElse(
                Rc::new(Predicate::And(Rc::clone(pred), Rc::clone(inner_pred))),
                Rc::clone(inner_true),
                Rc::clone(&simplified_false),
            ));
        }
    }

    if let Predicate::Equals(left, right) = pred.as_ref() {
        if let (MaybeCharExpression::CharExpression(c1), MaybeCharExpression::CharExpression(c2)) =
            (left.as_ref(), right.as_ref())
        {
            if let (CharExpression::Literal(val1), CharExpression::Literal(val2)) = (c1, c2) {
                return if val1 == val2 {
                    simplified_true
                } else {
                    simplified_false
                };
            }
        }
    }

    Rc::new(GenRegex::IfThenElse(
        Rc::clone(pred),
        simplified_true,
        simplified_false,
    ))
}
