/*
    Satisfiability checker using Antimirov derivatives

    This solver internally uses Brzozowski in the complement case,
    so it is a hybrid approach.
*/

use super::Solver;

use crate::antimirov::deriv::{derivative, nullable};
use crate::antimirov::subs::{
    merge_binary, merge_range_constraints, merge_sets, sub_in, RangeConstr, SimpleSub, SubExpr,
};
use crate::types::expr::{CharExpression, CharVar};
use crate::types::regex::GenRegex;

use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::{BTreeMap, BinaryHeap, HashSet};
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct AntimirovSolver {}

// Stores a regex and at what depth of derivative it was found.
struct DerivativeDepth {
    gre: Rc<GenRegex>,
    // TODO: Maybe combine not_sub and range constraints?
    not_sub: SimpleSub,
    range_constraints: BTreeMap<CharVar, RangeConstr>,
    depth: i32,
}
impl Ord for DerivativeDepth {
    fn cmp(&self, other: &Self) -> Ordering {
        other.gre.cmp(&self.gre)
    }
}

impl PartialOrd for DerivativeDepth {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for DerivativeDepth {
    fn eq(&self, other: &Self) -> bool {
        self.gre == other.gre
    }
}

impl Eq for DerivativeDepth {}

impl AntimirovSolver {
    pub fn new() -> Self {
        Self {}
    }
}

impl Solver for AntimirovSolver {
    fn satisfiable(&mut self, expr: &Rc<GenRegex>) -> bool {
        // TODO: track not constraints along with the derivative regexes

        let mut sat_stack = BinaryHeap::new();
        sat_stack.push(DerivativeDepth {
            gre: expr.clone(),
            not_sub: SimpleSub::empty(),
            range_constraints: BTreeMap::new(),
            depth: 0,
        });
        //TODO: Modify visited to compare not substitutions, not just the USR
        let mut visited: HashSet<Rc<GenRegex>> = HashSet::new();
        let mut count = 0;
        while let Some(layer) = sat_stack.pop() {
            println!("Looking at: {}", layer.gre);
            // if count>=2{
            //     return true;
            // }
            // count+=1;
            if !merge_sets(
                &HashSet::from([layer.not_sub.clone()]),
                &nullable(&layer.gre),
            )
            .is_empty()
            {
                println!("Nullable");
                return true;
            } else if visited.contains(&layer.gre) {
                println!("Pruned");
                continue;
            } else {
                visited.insert(layer.gre.clone());

                println!("Visited count: {}", visited.len());
                println!("Stack size: {}", sat_stack.len());

                let deriv = derivative(&layer.gre, &self.get_fresh_var(layer.depth));
                //println!("deriv: {:?}", deriv);
                'deriv_loop: for ele in deriv {
                    println!("Pushing expr: {}", ele.get_expr());
                    println!("Subs: {:?}", ele.get_subs());
                    // Check range
                    let range = ele.get_ranges();

                    let sub = ele.get_subs();
                    let char_map = sub.get_char_map();
                    let not_constr = sub.get_not_constraints();
                    for (var, range) in range {
                        // Checks if there are conflicts btwn range and charvar mappings
                        let in_char_map = char_map.get(var);
                        if let Some(found) = in_char_map {
                            if let CharExpression::Literal(lit) = found {
                                if lit > range.get_end() || lit < range.get_start() {
                                    //println!("death");
                                    continue 'deriv_loop;
                                }
                            }
                        }
                        // TODO Caleb
                        eprintln!("TODO: handle range constraint {} on {}", range, var);
                        // For now, ignore and continue
                    }
                    println!("Subs: {:?}", ele.get_subs());
                    println!("Not subs: {:?}", layer.not_sub);
                    let Some(mut f) = merge_binary(ele.get_subs(), &layer.not_sub) else {
                        // println!("death2");
                        continue;
                    };
                    *f.get_str_map_mut() = BTreeMap::new();
                    let new_re = sub_in(ele.get_expr(), &f);
                    // Potential code for fixing optimized range constraints, commented out due to Union Find index out of bounds issues
                    //println!("Before Range Update: {:?}",layer.range_constraints);
                    let mut updated_range = BTreeMap::new();
                    for (var, ranges) in &layer.range_constraints {
                        if let Some(c) = char_map.get(&var) {
                            if let CharExpression::CharVar(name) = c {
                                updated_range.insert(name.clone(), ranges.clone());
                            }
                        } else {
                            updated_range.insert(var.clone(), ranges.clone());
                        }
                    }
                    //println!("Updated Range: {:?}",updated_range);
                    let Some(r) = merge_range_constraints(ele.get_ranges(), &updated_range) else {
                        print!("Pruned");
                        continue;
                    };
                    //println!("After range merge: {:?}",r);
                    println! {"Not constraint pushed: {:?}",f};
                    sat_stack.push(DerivativeDepth {
                        gre: new_re.clone(),
                        not_sub: SimpleSub::new(
                            BTreeMap::new(),
                            BTreeMap::new(),
                            BTreeMap::new(),
                            f.get_not_constraints().clone(),
                        ),
                        range_constraints: r,
                        depth: layer.depth + 1,
                    });
                }
            }
        }
        false
    }
}

impl AntimirovSolver {
    fn get_fresh_var(&mut self, id: i32) -> Rc<CharExpression> {
        let var_name = format!("f.{}", id);
        Rc::new(CharExpression::CharVar(CharVar { name: var_name }))
    }
}
