//!
//! Solver interface to compare different algorithms
//!

/*
    Solver trait
*/

use crate::smt::parse::SmtParser;
use crate::types::regex::GenRegex;

use std::rc::Rc;

pub trait Solver {
    fn satisfiable(&mut self, gre: &Rc<GenRegex>) -> bool;

    // Get a hint from the parser, if needed
    fn parser_hint(&mut self, _parser: SmtParser) {
        // Default: do nothing
    }
}

/*
    Modules implementing the solver
*/

mod ab_solver;
mod antimirov;
mod brzozowski;

pub use ab_solver::ABSolver;
pub use antimirov::AntimirovSolver;
pub use brzozowski::BrzozowskiSolver;

pub fn solver_by_name(name: &str) -> Box<dyn Solver> {
    match name {
        "antimirov" => Box::new(AntimirovSolver::new()),
        "brzozowski" => Box::new(BrzozowskiSolver::new()),
        "ab" => Box::new(ABSolver::new()),
        _ => panic!("Unknown solver: {}", name),
    }
}
