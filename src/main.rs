//! Main entrypoint

// Better to fix and remove, allowing for now
#![allow(non_snake_case)]
mod brzozowski;
mod antimirov;
mod classes;
mod predicate_evaluation;
mod print;

use antimirov::{matching};
use classes::{CharExpression, GenRegex, Predicate, StringIndex, StringVar};
use either::Either;
use print::print_gre;
use std::rc::Rc;

// TODO: remove unused imports
// use predicateEvaluation::{convertToDNF, evaluateComplete, flatten_and_predicates};
// These should use Display instead
// use print::{
//     print_char_expression, print_equals_arg, print_gre, print_predicate, print_string_var,
// };
//use print::print_predicate;

// This is Brzozowski derivative, right?


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
    println!("{}", print_gre(&Rc::clone(gre4)));
    //let deriv = &Rc::new(derivative(&Rc::clone(gre), &Rc::new(CharExpression::Literal(String::from("b")))));
    //println!("{}", print_predicate(&nullableProjection(&Rc::clone(deriv))));
    let matcher = matching(&Rc::clone(gre4), String::from("catscatsacatscats"));
    println!("{}", matcher);
    //println!("Hello World!");
}
