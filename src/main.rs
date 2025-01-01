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

use brzozowski::matching;
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

    // TODO: Unused code?
    let _predicate = Predicate::Equals(
        Rc::new(MaybeCharExpression::CharExpression(Rc::new(CharExpression::Literal(String::from("a"))))), 
        Rc::new(MaybeCharExpression::StringIndex(Rc::new(StringIndex {
            var: Rc::clone(&string_var),
            index: 0,
        }))),
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
        Rc::clone(stringVarGre),
        Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::Literal(
            String::from("b"),
        )))),
    ));
    let gre4 = &Rc::new(GenRegex::Concatenation(Rc::clone(gre3), Rc::clone(gre2)));
    let gre5 = &Rc::new(GenRegex::Intersect(
        Rc::clone(gre3),
        Rc::new(GenRegex::Kleene(Rc::clone(stringVarGre))),
    ));

    //let complex_predicate = Rc::new(Predicate::And(vec![Rc::new(predicate), Rc::new(Predicate::False)]));
    //let gre = &Rc::new(GenRegex::IfThenElse(complex_predicate.clone(), Rc::new(GenRegex::EmptySet), Rc::new(GenRegex::CharExpression(Rc::new(CharExpression::CharVar(String::from("c")))))));
    //println!("{}", print_predicate(&complex_predicate));
    println!("{}", &Rc::clone(gre4));
    //let deriv = &Rc::new(derivative(&Rc::clone(gre), &Rc::new(CharExpression::Literal(String::from("b")))));
    //println!("{}", print_predicate(&nullableProjection(&Rc::clone(deriv))));
    let matcher = matching(&Rc::clone(gre4), String::from("catcatbcatcatcatcat"));
    println!("{}", matcher);
    //println!("Hello World!");
}
