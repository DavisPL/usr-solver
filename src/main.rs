//!
//! Main entrypoint
//!

// Better to fix and remove, allowing for now
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
mod antimirov;
mod brzozowski;
mod classes;
mod predicate_evaluation;
mod print;
mod smt;

use brzozowski::derivative;
use brzozowski::matching;
use brzozowski::nullable;
use brzozowski::nullableProjection;
use brzozowski::satisfiable;
use classes::CharVar;
use classes::{CharExpression, GenRegex, MaybeCharExpression, Predicate, StringIndex, StringVar};
use std::rc::Rc;

#[allow(unused_variables, unused_mut)]
fn main() {
    println!("sat");
    // Will take 1 arg .smt2 file
    // Print out true or false based on asserts in smt2 file
}
