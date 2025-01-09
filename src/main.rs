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
    let empty = &Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(String::from("")))));
    let literal_b = &Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(String::from("b")))));
    let test1 = &Rc::new(GenRegex::Union(Rc::clone(literal_a), Rc::clone(literal_b)));
    let test2 = &Rc::new(GenRegex::Kleene(Rc::clone(literal_a)));
    let test3 = &Rc::new(GenRegex::Concatenation(Rc::clone(test2), Rc::clone(literal_b)));
    let a_b = &Rc::new(GenRegex::Concatenation(Rc::clone(literal_a), Rc::clone(literal_b)));
    let test4 = &Rc::new(GenRegex::Kleene(Rc::clone(a_b)));
    let test5 = &Rc::new(GenRegex::Kleene(Rc::clone(test1)));
    let astar = &Rc::new(GenRegex::Kleene(Rc::clone(literal_a)));
    let test6 = &Rc::new(GenRegex::Complement(Rc::clone(test1)));
    let a_and_b = &Rc::new(GenRegex::Intersect(Rc::clone(literal_a), Rc::clone(literal_b)));
    let test7 = &Rc::new(GenRegex::Complement(Rc::clone(a_and_b)));
    let test8 = &Rc::new(GenRegex::Complement(Rc::clone(empty)));
    let test9 = &Rc::new(GenRegex::Complement(Rc::clone(astar)));


    let gre1 = &Rc::new(GenRegex::StringVar(string_var.clone()));

    let a_w = &Rc::new(GenRegex::Concatenation(Rc::clone(literal_a), Rc::clone(gre1)));
    let w_a = &Rc::new(GenRegex::Concatenation(Rc::clone(gre1), Rc::clone(literal_a)));
    let new_test = &Rc::new(GenRegex::Intersect(Rc::clone(a_w), Rc::clone(gre1)));
    /*let literal_c = &Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(String::from("c")))));

    //let char_expr = CharExpression::StringIndex(string_var, 0);
    let gre2 = &Rc::new(GenRegex::StringVar(string_var.clone()));
    let emptyset = &Rc::new(GenRegex::EmptySet);
    let gre = &Rc::new(GenRegex::Concatenation(Rc::clone(gre1), Rc::clone(literal_a)));
    let gre3 = &Rc::new(GenRegex::Concatenation(Rc::clone(literal_b), Rc::clone(gre2)));
    let union = &Rc::new(GenRegex::Union(Rc::clone(gre), Rc::clone(gre3)));
    let intersect = &Rc::new(GenRegex::Intersect(Rc::clone(gre1), Rc::clone(gre2)));
    let finalgre = &Rc::new(GenRegex::Intersect(Rc::clone(union), Rc::clone(literal_c)));
    let complement = &Rc::new(GenRegex::Complement(Rc::clone(emptyset)));
    let complement2 = &Rc::new(GenRegex::Complement(Rc::clone(complement)));*/


    //let complex_predicate = Rc::new(Predicate::And(vec![Rc::new(predicate), Rc::new(Predicate::False)]));
    //let gre = &Rc::new(GenRegex::IfThenElse(complex_predicate.clone(), Rc::new(GenRegex::EmptySet), Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::CharVar(String::from("c")))))));
    //println!("{}", print_predicate(&complex_predicate));
    //println!("{}", &Rc::clone(gre4));
    //println!("{}", derivative(&Rc::clone(finalgre), ind, HashSet::new()));
//    let deriv = derivative(&Rc::clone(intersect), &Rc::new(CharExpression::CharVar(classes::CharVar { name: String::from("c1") })));
    //let deriv2 = derivative(&Rc::clone(&deriv), &Rc::new(CharExpression::CharVar(classes::CharVar { name: String::from("c1") })));
//    println!("{}", deriv);
//    println!("{}", nullableProjection(&deriv));
   /* println!("{} {}", test1, satisfiable(&Rc::clone(test1)));
    println!("{} {}", test2, satisfiable(&Rc::clone(test2)));
    println!("{} {}", test3, satisfiable(&Rc::clone(test3)));
    //println!("{} {}", test4, satisfiable(&Rc::clone(test4)));*/
    println!("{} {}", new_test, satisfiable(&Rc::clone(new_test)));
    /*println!("{} {}", test6, satisfiable(&Rc::clone(test6)));
    println!("{} {}", test7, satisfiable(&Rc::clone(test7)));
    println!("{} {}", test8, satisfiable(&Rc::clone(test8)));
    println!("{} {}", test9, satisfiable(&Rc::clone(test9)));*/
    //println!("{} {}", intersect, satisfiable(&Rc::clone(intersect), ind, HashSet::new()));
    //for elem in deriv{
        //println!("{}", elem);
    //}
}
