//!
//! AB Solver
//! Antimirov-Brzoziwski Solver (for lack of a better name) that chooses
//! Antimirov for cases with no ITE, complement, or string indexes, and Brzozowski otherwise.
//!

use super::antimirov::AntimirovSolver;
use super::brzozowski::BrzozowskiSolver;
use super::Solver;

use std::rc::Rc;

#[derive(Debug, Default)]
pub struct ABSolver {
    solver_a: AntimirovSolver,
    solver_b: BrzozowskiSolver,
}

impl ABSolver {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Solver for ABSolver {
    fn satisfiable(&mut self, gre: &Rc<crate::types::regex::GenRegex>) -> bool {
        if gre.contains_ite_complement_or_str_index() {
            self.solver_b.satisfiable(gre)
        } else {
            self.solver_a.satisfiable(gre)
        }
    }
}
