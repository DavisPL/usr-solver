//!
//! AB Solver
//! Antimirov-Brzoziwski Solver (for lack of a better name) that chooses
//! Antimirov for cases with no ITE, complement, or string indexes, and Brzozowski otherwise.
//!

use super::antimirov::AntimirovSolver;
use super::brzozowski::BrzozowskiSolver;
use super::Solver;

use crate::smt::parse::SmtParser;

use std::rc::Rc;

#[derive(Debug, Default)]
pub struct ABSolver {
    solver_a: AntimirovSolver,
    solver_b: BrzozowskiSolver,
    use_brzozowski: Option<bool>,
}

impl ABSolver {
    pub fn new() -> Self {
        ABSolver {
            solver_a: AntimirovSolver::new(),
            solver_b: BrzozowskiSolver::new(),
            use_brzozowski: None,
        }
    }
}

impl Solver for ABSolver {
    fn parser_hint(&mut self, parser: SmtParser) {
        self.use_brzozowski = Some(parser.use_brzozowski());
    }
    fn satisfiable(&mut self, gre: &Rc<crate::types::regex::GenRegex>) -> bool {
        match self.use_brzozowski {
            Some(true) => self.solver_b.satisfiable(gre),
            Some(false) => self.solver_a.satisfiable(gre),
            None => {
                panic!("Error: ABSolver called without solver hint; defaulting to Antimirov");
            }
        }
    }
}
