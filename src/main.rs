//!
//! Binary entrypoint
//!

#![allow(clippy::uninlined_format_args)]

use gen_regex_impl::smt::parse::{parse_smtlib_file, SmtParser};
use gen_regex_impl::solver::{self, Solver};

use clap::Parser;
use std::rc::Rc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// SMT input file to run on
    filename: String,

    /// Solver to use: for current options, see src::solver/mod.rs.
    #[clap(long, short, default_value = "a")]
    solver_name: String,
}
//use std::ffi::CString;
//use z3::Context;
//use z3::Solver as OtherSolver;
//use z3::*;

fn main() {
    /*let cfg = z3::Config::new();
    let context = Context::new(&cfg);

    // Step 2: Create a Solver in the context
    let solver = OtherSolver::new(&context);

    //(assert (= (str.len x) 4))
    //(assert (= (str.len x) 5))
    //(assert (and (str.< (str.at s0 2) "b") (not (str.< (str.at s0 2) "b"))))

    let smtlib2_input = r#"
        (declare-fun s () String)
        (assert (and
        (= (< (char_at 2 s) "b") true)
        false))
    "#;
    solver.from_string(smtlib2_input); // Parse the string and add constraints
    match solver.check() {
        SatResult::Sat => println!("Satisfiable"),
        SatResult::Unsat => println!("Unsatisfiable"),
        SatResult::Unknown => println!("Unknown result (couldn't determine satisfiability)"),
    }*/

    // Print the internal state (for demonstration purposes)
    // Will take 1 arg .smt2 file
    // Print out true or false based on asserts in smt2 file
    let args = Args::parse();

    let v = parse_smtlib_file(&args.filename).expect("Invalid File path");
    let mut parser = SmtParser::new();
    let re = Rc::new(parser.parse_s_expr(&v).expect("Invalid S-expr"));

    let solver_name = solver::lookup_solver_name(&args.solver_name);
    println!("Using solver: {}", solver_name);
    let mut solver: Box<dyn Solver> = solver::solver_by_name(solver_name);

    let result = solver.satisfiable(&re);
    if result {
        println!("sat");
    } else {
        println!("unsat");
    }
}
