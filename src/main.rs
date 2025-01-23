//!
//! Main entrypoint
//!

// TODO: fix and remove, allowing for now
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]

use antimirov::satisfiable;
use clap::Parser;
use smt::SmtParser;
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
use classes::CharVar;
use classes::{CharExpression, GenRegex, MaybeCharExpression, Predicate, StringIndex, StringVar};
use std::rc::Rc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// SMT input file to run on
    filename: String,
}

fn main() {
    // Will take 1 arg .smt2 file
    // Print out true or false based on asserts in smt2 file
    let args = Args::parse();

    let v=smt::parse_smtlib_file(&args.filename).expect("Invalid File path");
    let mut parser=SmtParser::new();
    let re=parser.parse_s_expr(&v).expect("Invalid S-expr");
    let result:bool;
    if parser.brzozowski_flag==true{
        result=brzozowski::satisfiable(&Rc::new(re));
    }
    else{
        result=antimirov::satisfiable(&Rc::new(re));
    }
    if result{
        println!("sat");
    }
    else {
        println!("unsat");
    }
}
