/*
    Satisfiability checker using Brzozowski derivatives
*/

use super::Solver;

use crate::brzozowski::deriv::satisfiable as brzozowski_satisfiable;
use crate::types::regex::GenRegex;

use std::rc::Rc;

#[derive(Debug, Default)]
pub struct BrzozowskiSolver {}

impl BrzozowskiSolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl Solver for BrzozowskiSolver {
    fn satisfiable(&mut self, gre: &Rc<GenRegex>) -> bool {
        brzozowski_satisfiable(gre)
    }
}
