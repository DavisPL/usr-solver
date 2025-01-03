//!
//! Main entrypoint
//!

// Better to fix and remove, allowing for now
#![allow(non_snake_case)]
#![allow(dead_code)]

mod antimirov;
mod brzozowski;
mod classes;
mod predicate_evaluation;
mod print;
mod smt;

use antimirov::matching;
use antimirov::derivative;
use classes::{CharExpression, GenRegex, Predicate, StringIndex, StringVar, MaybeCharExpression};
use std::rc::Rc;

fn main() {
    let string_var = Rc::new(StringVar {
        name: String::from("w0"),
    });
    let char_var = &Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::CharVar(
        classes::CharVar { name: String::from("c1") }
    ))));

    //let char_expr = CharExpression::StringIndex(string_var, 0);
    let gre = &Rc::new(GenRegex::StringVar(string_var));


    //let complex_predicate = Rc::new(Predicate::And(vec![Rc::new(predicate), Rc::new(Predicate::False)]));
    //let gre = &Rc::new(GenRegex::IfThenElse(complex_predicate.clone(), Rc::new(GenRegex::EmptySet), Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::CharVar(String::from("c")))))));
    //println!("{}", print_predicate(&complex_predicate));
    //println!("{}", &Rc::clone(gre4));
    let deriv = derivative(&Rc::clone(gre), &Rc::new(CharExpression::Literal(String::from("b"))));
    for elem in deriv{
        println!("{}", elem);
    }
}
