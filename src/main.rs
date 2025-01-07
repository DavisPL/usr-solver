//!
//! Main entrypoint
//!

// Better to fix and remove, allowing for now
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
mod antimirov;
mod brzozowski;
mod classes;
mod predicate_evaluation;
mod print;
mod smt;

use brzozowski::matching;
use antimirov::derivative;
use antimirov::satisfiable;
use brzozowski::nullable;
use brzozowski::nullableProjection;
use classes::{CharExpression, GenRegex, Predicate, StringIndex, StringVar, MaybeCharExpression};
use std::rc::Rc;

fn main() {
    let string_var = Rc::new(StringVar {
        name: String::from("w0"),
    });
    let char_var = &Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::CharVar(
        classes::CharVar { name: String::from("c1") }
    ))));
    let literal_a = &Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(String::from("a")))));
    let literal_b = &Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(String::from("b")))));

    //let char_expr = CharExpression::StringIndex(string_var, 0);
    let gre1 = &Rc::new(GenRegex::StringVar(string_var.clone()));
    let gre2 = &Rc::new(GenRegex::StringVar(string_var.clone()));
    let gre = &Rc::new(GenRegex::Concatenation(Rc::clone(gre1), Rc::clone(literal_a)));
    let gre3 = &Rc::new(GenRegex::Concatenation(Rc::clone(literal_a), Rc::clone(gre2)));
    let finalgre = &Rc::new(GenRegex::Intersect(Rc::clone(gre2), Rc::clone(gre1)));


    //let complex_predicate = Rc::new(Predicate::And(vec![Rc::new(predicate), Rc::new(Predicate::False)]));
    //let gre = &Rc::new(GenRegex::IfThenElse(complex_predicate.clone(), Rc::new(GenRegex::EmptySet), Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::CharVar(String::from("c")))))));
    //println!("{}", print_predicate(&complex_predicate));
    //println!("{}", &Rc::clone(gre4));
    let mut ind = 0;
    //println!("{}", derivative(&Rc::clone(finalgre), ind, HashSet::new()));
    let deriv = derivative(&Rc::clone(finalgre), &Rc::new(CharExpression::CharVar(classes::CharVar { name: String::from("c1") })));
    //let deriv2 = derivative(&Rc::clone(&deriv), &Rc::new(CharExpression::CharVar(classes::CharVar { name: String::from("c1") })));
//    println!("{}", deriv);
//    println!("{}", nullableProjection(&deriv));
    //println!("{}", satisfiable(&Rc::clone(finalgre), ind, HashSet::new()));
    for elem in deriv{
        println!("{}", elem);
    }
}
