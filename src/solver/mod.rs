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

mod antimirov;
mod brzozowski;
mod determinizing;

pub use antimirov::AntimirovSolver;
pub use brzozowski::BrzozowskiSolver;
pub use determinizing::DeterminizingSolver;

pub fn lookup_solver_name(name: &str) -> &str {
    match name {
        "a" | "antimirov" => "Antimirov",
        "b" | "brzozowski" => "Brzozowski",
        "d" | "determinizing" => "Determinizing Antimirov",
        _ => panic!("Unknown solver: {}", name),
    }
}

pub fn solver_by_name(name: &str) -> Box<dyn Solver> {
    match name {
        "Antimirov" => Box::new(AntimirovSolver::new()),
        "Brzozowski" => Box::new(BrzozowskiSolver::new()),
        "Determinizing Antimirov" => Box::new(DeterminizingSolver::new()),
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

pub fn determinized_satisfiable(gre: &Rc<GenRegex>) -> bool {
    DeterminizingSolver::new().satisfiable(gre)
}

/*
    Helpers for test purposes
*/

/// Run ALL solvers, for test purposes
pub const NUM_SOLVERS: usize = 2;
pub fn satisfiable_all(gre: &Rc<GenRegex>) -> Vec<bool> {
    let result = vec![
        antimirov_satisfiable(gre),
        brzozowski_satisfiable(gre),
        // Disabled - deprecated
        // ab_satisfiable(gre),
        // Disabled as cannot handle string variables on its own
        // determinized_satisfiable(gre),
    ];
    debug_assert_eq!(result.len(), NUM_SOLVERS);
    result
}

/// Run only default solver, for test purposes
pub fn satisfiable_default(gre: &Rc<GenRegex>) -> bool {
    // ab_satisfiable(gre)
    antimirov_satisfiable(gre)
}
