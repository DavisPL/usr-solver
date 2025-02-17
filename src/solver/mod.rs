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

pub fn lookup_solver_name(name: &str) -> &str {
    match name {
        "a" | "antimirov" => "Antimirov",
        "b" | "brzozowski" => "Brzozowski",
        "ab" => "AB Solver",
        _ => panic!("Unknown solver: {}", name),
    }
}

pub fn solver_by_name(name: &str) -> Box<dyn Solver> {
    match name {
        "Antimirov" => Box::new(AntimirovSolver::new()),
        "Brzozowski" => Box::new(BrzozowskiSolver::new()),
        "AB Solver" => Box::new(ABSolver::new()),
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

/// Run ALL solvers, for test purposes
pub const NUM_SOLVERS: usize = 3;
pub fn satisfiable_all(gre: &Rc<GenRegex>) -> Vec<bool> {
    let mut results = Vec::new();
    results.push(antimirov_satisfiable(gre));
    results.push(brzozowski_satisfiable(gre));
    results.push(ab_satisfiable(gre));
    results
}
