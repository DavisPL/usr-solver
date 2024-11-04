//! Main entrypoint

// Better to fix and remove, allowing for now
#![allow(non_snake_case)]

mod classes;
mod predicate_evaluation;
mod print;
mod union_find;

use classes::{CharExpression, GenRegex, Predicate, StringIndex, StringVar};
use either::Either;
use std::rc::Rc;

// TODO: remove unused imports
// use predicateEvaluation::{convertToDNF, evaluateComplete, flatten_and_predicates};
use predicate_evaluation::evaluateComplete;
// These should use Display instead
// use print::{
//     print_char_expression, print_equals_arg, print_gre, print_predicate, print_string_var,
// };
use print::print_predicate;

// This is Brzozowski derivative, right?

fn derivative(gre: &Rc<GenRegex>, deriv_char: &Rc<CharExpression>) -> Rc<GenRegex> {
    let empty_string = || {
        Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
            String::new(),
        ))))
    };
    let empty_set = || Rc::new(GenRegex::EmptySet);
    match gre.as_ref() {
        GenRegex::EmptySet => return Rc::clone(gre),
        GenRegex::CharExpression(cExpr) => {
            return simplifies(&Rc::new(GenRegex::IfThenElse(
                Rc::new(Predicate::Equals(
                    Either::Left(Rc::clone(cExpr)),
                    Either::Left(Rc::clone(deriv_char)),
                )),
                empty_string(),
                empty_set(),
            )));
        }
        GenRegex::StringVar(sVar) => {
            return simplifies(&Rc::new(GenRegex::IfThenElse(
                Rc::new(Predicate::Equals(
                    Either::Right(Rc::new(StringIndex {
                        var: Rc::clone(sVar),
                        index: 0,
                    })),
                    Either::Left(Rc::clone(deriv_char)),
                )),
                Rc::new(GenRegex::StringSlice(Rc::clone(sVar), 1)),
                empty_set(),
            )));
        }
        GenRegex::StringSlice(stringVar, index) => {
            return simplifies(&Rc::new(GenRegex::IfThenElse(
                Rc::new(Predicate::Equals(
                    Either::Right(Rc::new(StringIndex {
                        var: Rc::clone(stringVar),
                        index: *index,
                    })),
                    Either::Left(Rc::clone(deriv_char)),
                )),
                Rc::new(GenRegex::StringSlice(Rc::clone(stringVar), index + 1)),
                empty_set(),
            )));
        }
        GenRegex::Union(side1, side2) => {
            return simplifies(&Rc::new(GenRegex::Union(
                Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
                Rc::clone(&derivative(&Rc::clone(side2), deriv_char)),
            )));
        }
        GenRegex::Intersect(side1, side2) => {
            return simplifies(&Rc::new(GenRegex::Intersect(
                Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
                Rc::clone(&derivative(&Rc::clone(side2), deriv_char)),
            )));
        }
        GenRegex::Concatenation(side1, side2) => {
            let leftSide = Rc::new(GenRegex::Concatenation(
                Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
                Rc::clone(side2),
            ));
            let rightSide = Rc::new(GenRegex::Concatenation(
                Rc::clone(&nullable(&Rc::clone(side1))),
                Rc::clone(&derivative(&Rc::clone(side2), deriv_char)),
            ));

            return simplifies(&Rc::new(GenRegex::Union(
                Rc::clone(&leftSide),
                Rc::clone(&rightSide),
            )));
        }
        GenRegex::Complement(side1) => {
            return simplifies(&Rc::new(GenRegex::Complement(Rc::clone(&derivative(
                &Rc::clone(side1),
                deriv_char,
            )))));
        }
        GenRegex::Kleene(side1) => {
            return simplifies(&Rc::new(GenRegex::Concatenation(
                Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
                Rc::clone(side1),
            )));
        }
        GenRegex::IfThenElse(pred, side1, side2) => {
            return simplifies(&Rc::new(GenRegex::IfThenElse(
                Rc::clone(pred),
                Rc::clone(&derivative(&Rc::clone(side1), deriv_char)),
                Rc::clone(&derivative(&Rc::clone(side2), deriv_char)),
            )));
        }
        GenRegex::StringIndex(string_index) => {
            // TODO: unused?
            return simplifies(&Rc::new(GenRegex::IfThenElse(
                Rc::new(Predicate::Equals(
                    Either::Right(Rc::clone(&string_index)),
                    Either::Left(Rc::clone(deriv_char)),
                )),
                empty_string(),
                empty_set(),
            )));
        }
    }
}

fn nullable(gre: &Rc<GenRegex>) -> Rc<GenRegex> {
    match gre.as_ref() {
        GenRegex::EmptySet => return Rc::clone(gre),
        GenRegex::CharExpression(cExpr) => match cExpr.as_ref() {
            CharExpression::CharVar(_name) => {
                // TODO: Unused?
                return Rc::new(GenRegex::EmptySet);
            }
            CharExpression::Literal(value) => {
                if value.is_empty() {
                    return Rc::clone(gre);
                } else {
                    return Rc::new(GenRegex::EmptySet);
                }
            }
        },
        GenRegex::StringSlice(string_var, index) => {
            return Rc::new(GenRegex::IfThenElse(
                Rc::new(Predicate::EqualLength(Rc::clone(string_var), *index)),
                Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
                    String::from(""),
                )))),
                Rc::new(GenRegex::EmptySet),
            ))
        }
        GenRegex::StringVar(string_var) => {
            return Rc::new(GenRegex::IfThenElse(
                Rc::new(Predicate::EqualLength(Rc::clone(string_var), 0)),
                Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
                    String::from(""),
                )))),
                Rc::new(GenRegex::EmptySet),
            ))
        }
        GenRegex::StringIndex(_string_index) => {
            return Rc::new(GenRegex::EmptySet);
        }
        GenRegex::Union(side1, side2) => {
            return Rc::new(GenRegex::Union(
                Rc::clone(&nullable(&Rc::clone(side1))),
                Rc::clone(&nullable(&Rc::clone(side2))),
            ))
        }
        GenRegex::Intersect(side1, side2) => {
            return Rc::new(GenRegex::Intersect(
                Rc::clone(&nullable(&Rc::clone(side1))),
                Rc::clone(&nullable(&Rc::clone(side2))),
            ))
        }
        GenRegex::Concatenation(side1, side2) => {
            return Rc::new(GenRegex::Concatenation(
                Rc::clone(&nullable(&Rc::clone(side1))),
                Rc::clone(&nullable(&Rc::clone(side2))),
            ))
        }
        GenRegex::Complement(side1) => {
            return Rc::new(GenRegex::Intersect(
                Rc::new(GenRegex::Complement(Rc::clone(&nullable(&Rc::clone(
                    side1,
                ))))),
                Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
                    String::from(""),
                )))),
            ))
        }
        GenRegex::Kleene(_) => {
            return Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
                String::from(""),
            ))))
        }
        GenRegex::IfThenElse(pred, side1, side2) => {
            return Rc::new(GenRegex::IfThenElse(
                Rc::clone(pred),
                Rc::clone(&nullable(&Rc::clone(side1))),
                Rc::clone(&nullable(&Rc::clone(side2))),
            ))
        }
    }
}

fn nullableProjectionHelper(expr: &Rc<GenRegex>) -> Rc<Predicate> {
    match expr.as_ref() {
        GenRegex::EmptySet => Rc::new(Predicate::False),

        GenRegex::CharExpression(cExpr) => match cExpr.as_ref() {
            CharExpression::CharVar(_name) => return Rc::new(Predicate::False),
            CharExpression::Literal(_value) => return Rc::new(Predicate::True),
        },

        GenRegex::IfThenElse(pred, true_expr, false_expr) => {
            let true_proj = nullableProjectionHelper(true_expr);
            let false_proj = nullableProjectionHelper(false_expr);
            println!("{}", print_predicate(&true_proj));

            match (true_proj.as_ref(), false_proj.as_ref()) {
                (Predicate::False, Predicate::False) => Rc::new(Predicate::False),
                (Predicate::False, _) => Rc::new(Predicate::And(vec![
                    Rc::new(Predicate::Not(Rc::clone(pred))),
                    Rc::clone(&false_proj),
                ])),
                (_, Predicate::False) => {
                    Rc::new(Predicate::And(vec![Rc::clone(pred), Rc::clone(&true_proj)]))
                }
                _ => Rc::new(Predicate::Or(vec![
                    Rc::new(Predicate::And(vec![Rc::clone(pred), Rc::clone(&true_proj)])),
                    Rc::new(Predicate::And(vec![
                        Rc::new(Predicate::Not(Rc::clone(pred))),
                        Rc::clone(&false_proj),
                    ])),
                ])),
            }
        }

        GenRegex::Union(left, right) => {
            let left_proj = nullableProjectionHelper(left);
            let right_proj = nullableProjectionHelper(right);

            match (left_proj.as_ref(), right_proj.as_ref()) {
                (Predicate::False, _) => Rc::clone(&right_proj),
                (_, Predicate::False) => Rc::clone(&left_proj),
                _ => Rc::new(Predicate::Or(vec![left_proj, right_proj])),
            }
        }

        GenRegex::Intersect(left, right) | GenRegex::Concatenation(left, right) => {
            let left_proj = nullableProjectionHelper(left);
            let right_proj = nullableProjectionHelper(right);

            match (left_proj.as_ref(), right_proj.as_ref()) {
                (Predicate::False, _) | (_, Predicate::False) => Rc::new(Predicate::False),
                _ => Rc::new(Predicate::And(vec![left_proj, right_proj])),
            }
        }

        GenRegex::Complement(inner) => Rc::new(Predicate::Not(nullableProjectionHelper(inner))),

        _ => Rc::new(Predicate::False),
    }
}
fn nullableProjection(gre: &Rc<GenRegex>) -> Rc<Predicate> {
    let nullableGre = &nullable(gre);
    let mut nullablePredicates = nullableProjectionHelper(nullableGre);
    nullablePredicates = evaluateComplete(&nullablePredicates);
    println!("{}", print_predicate(&nullablePredicates));
    return nullablePredicates;
}

fn matching(gre: &Rc<GenRegex>, proposed: String) -> bool {
    println!("{}", proposed);
    let expr = &simplifies(gre);
    if proposed.is_empty() {
        return !matches!(nullableProjection(expr).as_ref(), Predicate::False);
    }
    let literal = Rc::new(CharExpression::Literal(String::from(&proposed[0..1])));
    return matching(&derivative(expr, &literal), String::from(&proposed[1..]));
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
            match (c1.as_ref(), c2.as_ref()) {
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
            Rc::new(Predicate::And(vec![Rc::clone(pred1), Rc::clone(pred2)])),
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
        Rc::new(Predicate::And(vec![
            Rc::clone(pred2),
            Rc::new(Predicate::Not(Rc::clone(pred1))),
        ]))
    } else {
        Rc::new(Predicate::And(vec![
            Rc::clone(pred1),
            Rc::new(Predicate::Not(Rc::clone(pred2))),
        ]))
    };

    Rc::new(GenRegex::IfThenElse(
        Rc::new(Predicate::And(vec![Rc::clone(pred1), Rc::clone(pred2)])),
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
        Rc::new(Predicate::And(vec![Rc::clone(pred1), Rc::clone(pred2)])),
        Rc::new(GenRegex::Intersect(Rc::clone(true1), Rc::clone(true2))),
        Rc::new(GenRegex::IfThenElse(
            Rc::new(Predicate::And(vec![
                Rc::clone(pred1),
                Rc::new(Predicate::Not(Rc::clone(pred2))),
            ])),
            Rc::new(GenRegex::Intersect(Rc::clone(false2), Rc::clone(true1))),
            Rc::new(GenRegex::IfThenElse(
                Rc::new(Predicate::And(vec![
                    Rc::clone(pred2),
                    Rc::new(Predicate::Not(Rc::clone(pred1))),
                ])),
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
            match (c1.as_ref(), c2.as_ref()) {
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

    match (&*simplified_true, &*simplified_false) {
        (GenRegex::EmptySet, GenRegex::EmptySet) => {
            return simplified_true;
        }
        _ => {}
    };

    match pred.as_ref() {
        Predicate::Equals(left, right) => {
            if let (Either::Left(c1), Either::Left(c2)) = (left.as_ref(), right.as_ref()) {
                if let (CharExpression::Literal(val1), CharExpression::Literal(val2)) =
                    (c1.as_ref(), c2.as_ref())
                {
                    return if val1 == val2 {
                        simplified_true
                    } else {
                        simplified_false
                    };
                }
            }
        }
        _ => {}
    }

    Rc::new(GenRegex::IfThenElse(
        Rc::clone(pred),
        simplified_true,
        simplified_false,
    ))
}

fn main() {
    let string_var = Rc::new(StringVar {
        name: String::from("w0"),
    });

    //let char_expr = CharExpression::StringIndex(string_var, 0);

    // TODO: Unused code?
    let _predicate = Predicate::Equals(
        Either::Left(Rc::new(CharExpression::Literal(String::from("a")))), // First argument wrapped in Either::Left
        Either::Right(Rc::new(StringIndex {
            var: Rc::clone(&string_var),
            index: 0,
        })),
    );
    let _intersect = &Rc::new(GenRegex::Intersect(
        Rc::new(GenRegex::StringVar(Rc::clone(&string_var))),
        Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
            String::from("a"),
        )))),
    ));
    //let complement = &Rc::new(GenRegex::Complement(Rc::new(GenRegex::StringIndex(Rc::new(StringIndex{var: string_var, index: 3})))));
    let stringVarGre = &Rc::new(GenRegex::StringVar(Rc::clone(&string_var)));
    let gre2 = &Rc::new(GenRegex::Concatenation(
        Rc::clone(stringVarGre),
        Rc::clone(stringVarGre),
    ));
    let gre3 = &Rc::new(GenRegex::Concatenation(
        Rc::clone(gre2),
        Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
            String::from("a"),
        )))),
    ));
    let gre4 = &Rc::new(GenRegex::Concatenation(Rc::clone(gre3), Rc::clone(gre2)));

    //let complex_predicate = Rc::new(Predicate::And(vec![Rc::new(predicate), Rc::new(Predicate::False)]));
    //let gre = &Rc::new(GenRegex::IfThenElse(complex_predicate.clone(), Rc::new(GenRegex::EmptySet), Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::CharVar(String::from("c")))))));
    //println!("{}", print_predicate(&complex_predicate));
    //println!("{}", print_gre(&Rc::clone(gre)));
    //let deriv = &Rc::new(derivative(&Rc::clone(gre), &Rc::new(CharExpression::Literal(String::from("b")))));
    //println!("{}", print_predicate(&nullableProjection(&Rc::clone(deriv))));
    let matcher = matching(&Rc::clone(gre4), String::from("abcabcabcabc"));
    println!("{}", matcher);
    println!("Hello World!");
}
