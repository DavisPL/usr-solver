//!
//! Implementation of the Brzozowski Derivative
//!

#![allow(unused_variables)]

use crate::classes::{
    CharExpression, CharVar, GenRegex, MaybeCharExpression, Predicate, StringIndex,
};
use crate::predicate_evaluation::evaluateComplete;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub fn satisfiable_helper(
    gre: &Rc<GenRegex>,
    index: &mut i32,
    visited: &mut HashSet<Rc<GenRegex>>,
) -> bool {
    //let mut expr;
    match gre.as_ref() {
        GenRegex::IfThenElse(pred, left, right) => {
            let mut temp_left = left.clone();
            let mut temp_right = right.clone();
            if visited.contains(left) {
                temp_left = Rc::new(GenRegex::EmptySet);
            }
            if visited.contains(right) {
                temp_right = Rc::new(GenRegex::EmptySet);
            }
            let expr = &simplifies(&Rc::new(GenRegex::IfThenElse(
                pred.clone(),
                temp_left,
                temp_right,
            )));
            if matches!(nullableProjection(expr)[0][0].as_ref(), Predicate::False) {
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
                temp_left = Rc::new(GenRegex::EmptySet);
            }
            if visited.contains(right) {
                temp_right = Rc::new(GenRegex::EmptySet);
            }
            let expr = &simplifies(&Rc::new(GenRegex::Union(temp_left, temp_right)));
            if matches!(nullableProjection(expr)[0][0].as_ref(), Predicate::False) {
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
            if matches!(nullableProjection(expr)[0][0].as_ref(), Predicate::False) {
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
    let empty_string = || {
        Rc::new(GenRegex::CharExpression(CharExpression::Literal(
            String::new(),
        )))
    };
    let empty_set = || Rc::new(GenRegex::EmptySet);
    match gre.as_ref() {
        GenRegex::EmptySet => Rc::clone(gre),
        GenRegex::CharExpression(cExpr) => simplifies(&Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::Equals(
                Rc::new(MaybeCharExpression::CharExpression(cExpr.clone())),
                Rc::new(MaybeCharExpression::CharExpression(deriv_char.as_ref().clone())),
            )),
            empty_string(),
            empty_set(),
        ))),
        GenRegex::StringVar(sVar) => simplifies(&Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::Equals(
                Rc::new(MaybeCharExpression::StringIndex(StringIndex {
                    var: sVar.clone(),
                    index: 0,
                })),
                Rc::new(MaybeCharExpression::CharExpression(deriv_char.as_ref().clone())),
            )),
            Rc::new(GenRegex::StringSlice(sVar.clone(), 1)),
            empty_set(),
        ))),
        GenRegex::StringSlice(stringVar, index) => simplifies(&Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::Equals(
                Rc::new(MaybeCharExpression::StringIndex(StringIndex {
                    var: stringVar.clone(),
                    index: *index,
                })),
                Rc::new(MaybeCharExpression::CharExpression(deriv_char.as_ref().clone())),
            )),
            Rc::new(GenRegex::StringSlice(stringVar.clone(), index + 1)),
            empty_set(),
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
            let leftSide = Rc::new(GenRegex::Concatenation(
                Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
                Rc::clone(side2),
            ));
            let rightSide = Rc::new(GenRegex::Concatenation(
                Rc::clone(&nullable(&Rc::clone(side1))),
                Rc::clone(&derivative(&Rc::clone(side2), deriv_char)),
            ));

            simplifies(&Rc::new(GenRegex::Union(
                Rc::clone(&leftSide),
                Rc::clone(&rightSide),
            )))
        }
        GenRegex::Complement(side1) => simplifies(&Rc::new(GenRegex::Complement(Rc::clone(
            &derivative(&Rc::clone(side1), deriv_char),
        )))),
        GenRegex::Kleene(side1) => simplifies(&Rc::new(GenRegex::Concatenation(
            Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
            Rc::clone(side1),
        ))),
        GenRegex::IfThenElse(pred, side1, side2) => simplifies(&Rc::new(GenRegex::IfThenElse(
            Rc::clone(pred),
            Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
            Rc::clone(&derivative(&Rc::clone(side2), deriv_char)),
        ))),
        GenRegex::StringIndex(string_index) => {
            // TODO: unused?
            simplifies(&Rc::new(GenRegex::IfThenElse(
                Rc::new(Predicate::Equals(
                    Rc::new(MaybeCharExpression::StringIndex(string_index.clone())),
                    Rc::new(MaybeCharExpression::CharExpression(deriv_char.as_ref().clone())),
                )),
                empty_string(),
                empty_set(),
            )))
        }
    }
}

pub fn nullable(gre: &Rc<GenRegex>) -> Rc<GenRegex> {
    match gre.as_ref() {
        GenRegex::EmptySet => Rc::clone(gre),
        GenRegex::CharExpression(cExpr) => match cExpr {
            CharExpression::CharVar(_name) => {
                // TODO: Unused?
                Rc::new(GenRegex::EmptySet)
            }
            CharExpression::Literal(value) => {
                if value.is_empty() {
                    Rc::clone(gre)
                } else {
                    Rc::new(GenRegex::EmptySet)
                }
            }
        },
        GenRegex::StringSlice(string_var, index) => Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::EqualLength(Rc::new(string_var.clone()), *index)),
            Rc::new(GenRegex::CharExpression(CharExpression::Literal(
                String::from(""),
            ))),
            Rc::new(GenRegex::EmptySet),
        )),
        GenRegex::StringVar(string_var) => Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::EqualLength(Rc::new(string_var.clone()), 0)),
            Rc::new(GenRegex::CharExpression(CharExpression::Literal(
                String::from(""),
            ))),
            Rc::new(GenRegex::EmptySet),
        )),
        GenRegex::StringIndex(_string_index) => Rc::new(GenRegex::EmptySet),
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
            Rc::new(GenRegex::CharExpression(CharExpression::Literal(
                String::from(""),
            ))),
        )),
        GenRegex::Kleene(_) => Rc::new(GenRegex::CharExpression(CharExpression::Literal(
            String::from(""),
        ))),
        GenRegex::IfThenElse(pred, side1, side2) => Rc::new(GenRegex::IfThenElse(
            Rc::clone(pred),
            Rc::clone(&nullable(&Rc::clone(side1))),
            Rc::clone(&nullable(&Rc::clone(side2))),
        )),
    }
}

fn nullableProjectionHelper(expr: &Rc<GenRegex>) -> Rc<Predicate> {
    match expr.as_ref() {
        GenRegex::EmptySet => Rc::new(Predicate::False),

        GenRegex::CharExpression(cExpr) => match cExpr {
            CharExpression::CharVar(_name) => Rc::new(Predicate::False),
            CharExpression::Literal(_value) => Rc::new(Predicate::True),
        },

        GenRegex::IfThenElse(pred, true_expr, false_expr) => {
            let true_proj = nullableProjectionHelper(true_expr);
            let false_proj = nullableProjectionHelper(false_expr);
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
            let left_proj = nullableProjectionHelper(left);
            let right_proj = nullableProjectionHelper(right);

            match (left_proj.as_ref(), right_proj.as_ref()) {
                (Predicate::False, _) => Rc::clone(&right_proj),
                (_, Predicate::False) => Rc::clone(&left_proj),
                _ => Rc::new(Predicate::Or(left_proj, right_proj)), //Rewrite as left and
                                                                    //right
            }
        }

        GenRegex::Intersect(left, right) | GenRegex::Concatenation(left, right) => {
            let left_proj = nullableProjectionHelper(left);
            let right_proj = nullableProjectionHelper(right);

            match (left_proj.as_ref(), right_proj.as_ref()) {
                (Predicate::False, _) | (_, Predicate::False) => Rc::new(Predicate::False),
                _ => Rc::new(Predicate::And(left_proj, right_proj)),
            }
        }

        GenRegex::Complement(inner) => Rc::new(Predicate::Not(nullableProjectionHelper(inner))),

        _ => Rc::new(Predicate::False),
    }
}
pub fn nullableProjection(gre: &Rc<GenRegex>) -> Vec<Vec<Rc<Predicate>>> {
    let nullableGre = &nullable(gre);
    let nullablePredicates = nullableProjectionHelper(nullableGre);
    println!("{}", nullablePredicates);
    evaluateComplete(&nullablePredicates)
    //nullablePredicatesEval
}

pub fn matching(gre: &Rc<GenRegex>, proposed: String) -> bool {
    let expr = &simplifies(gre);
    if proposed.is_empty() {
        return !matches!(nullableProjection(expr)[0][0].as_ref(), Predicate::False);
    }
    let literal = Rc::new(CharExpression::Literal(String::from(&proposed[0..1])));
    matching(&derivative(expr, &literal), String::from(&proposed[1..]))
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
        _ => Rc::clone(gre),
    }
}

fn simplify_union(left_side: &Rc<GenRegex>, right_side: &Rc<GenRegex>) -> Rc<GenRegex> {
    let left = simplifies(left_side);
    let right = simplifies(right_side);

    match (&*left, &*right) {
        (GenRegex::EmptySet, _) => right,
        (_, GenRegex::EmptySet) => left,
        (GenRegex::CharExpression(c1), GenRegex::CharExpression(c2)) => {
            match (c1, c2) {
                (CharExpression::Literal(val1), CharExpression::Literal(val2)) if val1 == val2 => {
                    right
                }
                _ => Rc::new(GenRegex::Union(left, right)),
            }
        }
        _ => Rc::new(GenRegex::Union(left, right)),
    }
}

fn simplify_intersect(left_side: &Rc<GenRegex>, right_side: &Rc<GenRegex>) -> Rc<GenRegex> {
    let left = simplifies(left_side);
    let right = simplifies(right_side);

    match (&*left, &*right) {
        (GenRegex::EmptySet, _) | (_, GenRegex::EmptySet) => Rc::new(GenRegex::EmptySet),
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
                Rc::new(GenRegex::EmptySet),
            )),
        )),
    ))
}

fn simplify_concatenation(left_side: &Rc<GenRegex>, right_side: &Rc<GenRegex>) -> Rc<GenRegex> {
    let left = simplifies(left_side);
    let right = simplifies(right_side);

    match (&*left, &*right) {
        (GenRegex::EmptySet, _) | (_, GenRegex::EmptySet) => Rc::new(GenRegex::EmptySet),
        (GenRegex::CharExpression(c1), GenRegex::CharExpression(c2)) => {
            match (c1, c2) {
                (CharExpression::Literal(s1), _) if s1.is_empty() => right,
                (_, CharExpression::Literal(s2)) if s2.is_empty() => left,
                _ => Rc::new(GenRegex::Concatenation(left, right)),
            }
        }
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

    if let (GenRegex::EmptySet, GenRegex::EmptySet) = (&*simplified_true, &*simplified_false) {
        return simplified_true;
    };

    if let Predicate::Equals(left, right) = pred.as_ref() {
        if let (MaybeCharExpression::CharExpression(c1), MaybeCharExpression::CharExpression(c2)) =
            (left.as_ref(), right.as_ref())
        {
            if let (CharExpression::Literal(val1), CharExpression::Literal(val2)) =
                (c1, c2)
            {
                return if val1 == val2 {
                    simplified_true
                } else {
                    simplified_false
                };
            } else if let (CharExpression::Literal(val1), CharExpression::CharVar(_)) =
                (c1, c2)
            {
                if val1.is_empty() {
                    return simplified_false;
                }
            } else if let (CharExpression::CharVar(_), CharExpression::Literal(val1)) =
                (c1, c2)
            {
                if val1.is_empty() {
                    return simplified_false;
                }
            }
        }
    }

    Rc::new(GenRegex::IfThenElse(
        Rc::clone(pred),
        simplified_true,
        simplified_false,
    ))
}
