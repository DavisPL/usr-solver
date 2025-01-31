//!
//! Main entrypoint
//!

// TODO: fix and remove
#![allow(dead_code)]

mod antimirov;
mod antimirov_sat;
mod brzozowski;
mod classes;
mod predicate_evaluation;
mod print;
mod smt;

use antimirov_sat::SatChecker;
use clap::Parser;
use std::rc::Rc;

use smt::SmtParser;

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

    let v = smt::parse_smtlib_file(&args.filename).expect("Invalid File path");
    let mut parser = SmtParser::new();
    let re = parser.parse_s_expr(&v).expect("Invalid S-expr");
    let result: bool = if parser.use_brzozowski() {
        println!("Using Brzozowski");
        brzozowski::satisfiable(&Rc::new(re))
    } else {
        println!("Using Antimirov");
        //antimirov::satisfiable(&Rc::new(re))
        let mut sat_check = SatChecker::new();
        sat_check.satisfiable(&Rc::new(re))
    };
    if result {
        println!("sat");
    } else {
        println!("unsat");
    }
}
