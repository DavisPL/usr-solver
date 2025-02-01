//!
//! Solver interface to compare different algorithms
//!

/*
    Solver trait
*/

use crate::types::regex::GenRegex;

use std::rc::Rc;

pub trait Solver {
    fn satisfiable(&mut self, gre: &Rc<GenRegex>) -> bool;
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

/*
    Convenience functions
*/

pub fn antimirov_satisfiable(gre: &Rc<GenRegex>) -> bool {
    AntimirovSolver::new().satisfiable(gre)
}

pub fn brzozowski_satisfiable(gre: &Rc<GenRegex>) -> bool {
    BrzozowskiSolver::new().satisfiable(gre)
}

pub fn ab_satisfiable(gre: &Rc<GenRegex>) -> bool {
    ABSolver::new().satisfiable(gre)
}

/// Default solver to use for unit tests and benchmarks
pub fn satisfiable(gre: &Rc<GenRegex>) -> bool {
    ab_satisfiable(gre)
}
