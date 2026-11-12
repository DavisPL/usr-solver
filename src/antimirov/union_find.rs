//!
//! Union-find functions
//!

use crate::antimirov::subs::{RangeConstr, SubExpr};
use crate::types::expr::{CharExpression, CharVar, StringVar};

use super::subs::{AnySub, SimpleSub};

pub use disjoint_sets::UnionFind;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;
use std::unreachable;

#[derive(Debug, Clone)]
pub struct SubMerger {
    union_find: UnionFind<usize>,
    expr_to_id: HashMap<Rc<CharExpression>, usize>,
    id_to_expr: HashMap<usize, Rc<CharExpression>>,
    canonical_map: HashMap<usize, usize>,
}

impl SubMerger {
    pub fn new_for_sub(substitutions: &AnySub) -> Self {
        Self::new_with_capacity(substitutions.count_union_elems() + 2)
    }

    fn new_with_capacity(cap: usize) -> Self {
        let expr_to_id: HashMap<Rc<CharExpression>, usize> = HashMap::new();
        let id_to_expr: HashMap<usize, Rc<CharExpression>> = HashMap::new();
        let canonical_map: HashMap<usize, usize> = HashMap::new();
        let union_find: UnionFind<usize> = UnionFind::new(cap);
        Self {
            union_find,
            expr_to_id,
            id_to_expr,
            canonical_map,
        }
    }

    /*
        RESTRUCTURED CODE -- WIP REFACTOR
    */

    // PHASE 1: Merge String Maps
    pub fn merge_string_maps(
        &mut self,
        str_map: &mut BTreeMap<StringVar, Vec<SubExpr>>,
    ) -> Option<()> {
        for eq_exprs in str_map.values_mut() {
            let mut ind = 0;
            while eq_exprs.len() > 1 {
                let mut length_flag = false;
                let mut union_set: HashSet<Rc<CharExpression>> = HashSet::new();
                let mut i = 0;
                while i < eq_exprs.len() {
                    let curr_sub_expr = &eq_exprs[i];
                    if ind < curr_sub_expr.head_length() {
                        let temp = &curr_sub_expr[ind];
                        union_set.insert(Rc::new(temp.clone()));
                        i += 1;
                    } else if curr_sub_expr.get_tail() && eq_exprs.len() > 1 {
                        eq_exprs.remove(i);
                    } else {
                        for (j, item_j) in eq_exprs.iter().enumerate() {
                            if i != j {
                                if ind < item_j.head_length() {
                                    return None;
                                } else {
                                    continue;
                                }
                            }
                        }
                        //str_eq_class.insert(var.clone(), vec![curr_sub_expr.clone()]);
                        let new_vec = vec![curr_sub_expr.clone()];

                        // Move the ownership of `new_vec` to `eq_exprs`
                        *eq_exprs = new_vec;
                        //eq_exprs = &mut vec![curr_sub_expr.clone()];
                        length_flag = true;
                        break;
                    }
                }
                if length_flag {
                    break;
                }
                ind += 1;
                if !self.union_over_set(&union_set) {
                    return None;
                }
            }
        }
        Some(())
    }

    // PHASE 2: Merge Char Maps
    pub fn merge_char_maps(
        &mut self,
        char_eq_class: &BTreeMap<CharVar, Vec<CharExpression>>,
    ) -> Option<HashSet<Rc<CharVar>>> {
        let mut char_set = HashSet::new();
        for (var, eq_exprs) in char_eq_class {
            let mut temp_set: HashSet<Rc<CharVar>> = eq_exprs
                .iter()
                .filter_map(|expr| {
                    if let CharExpression::CharVar(ref var) = *expr {
                        Some(Rc::new(var.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            let mut union_set: HashSet<_> = eq_exprs
                .iter()
                .map(|expr| Rc::new((expr).clone())) // Dereference `expr` (&&CharExpression) and clone
                .collect();
            union_set.insert(Rc::new(CharExpression::CharVar(var.clone())));
            temp_set.insert(Rc::new(var.clone()));

            if !self.union_over_set(&union_set) {
                return None;
            }
            char_set = char_set.union(&temp_set).cloned().collect();
        }
        Some(char_set)
    }

    // PHASE 3: Set Char Maps
    pub fn set_char_maps(&mut self, char_set: &HashSet<Rc<CharVar>>, result: &mut SimpleSub) {
        for var in char_set {
            let char_expression = Rc::new(CharExpression::CharVar(var.as_ref().clone()));
            let canonical_repr = self.find_expr(char_expression);
            result.set_char_var(var.as_ref().clone(), (*canonical_repr).clone());
        }
    }

    pub fn get_canonical_rep(&self, id: usize) -> Option<&usize> {
        self.canonical_map.get(&id)
    }

    // PHASE 4: Set Str Maps
    pub fn set_str_maps(
        &self,
        str_map: &mut BTreeMap<StringVar, Vec<SubExpr>>,
        result: &mut SimpleSub,
    ) {
        for (var, eq_exprs) in str_map {
            let sub_expr_vector = eq_exprs[0].get_mut_head();
            for item in sub_expr_vector.iter_mut() {
                if let CharExpression::CharVar(c_var) = item {
                    let substitution_value = result.get_char_var(c_var);
                    match substitution_value {
                        Some(v) => {
                            // The key was found, and `v` is the value, so update the vector element
                            *item = v.clone();
                            //println!("Updated value at index {}: {:?}", i, v);
                        }
                        None => {
                            // The key was not found, so do nothing
                            //println!("No value found for key at index {}", i);
                        }
                    }
                }
            }
            result.set_str_var(var.clone(), eq_exprs[0].clone());
        }
    }

    // PHASE 5: Set Not Constraints
    pub fn set_not_constraints(&mut self, subs: &AnySub, result: &mut SimpleSub) -> Option<()> {
        let mut combined_not = BTreeMap::new();
        for (c, not_constraint_set) in subs.get_not_constraints() {
            let modified_not = self.find_set(&not_constraint_set);
            let key = CharExpression::CharVar(c.clone());
            if !self.expr_to_id.contains_key(&Rc::new(key.clone())) {
                self.expr_to_id
                    .insert(Rc::new(key.clone()), self.expr_to_id.len() + 1);
                self.id_to_expr
                    .insert(self.expr_to_id.len(), Rc::new(key.clone()));
            }
            let id_var = self.expr_to_id[&key.clone()];
            let new_c = self.id_to_expr[&self.union_find.find(id_var).clone()].clone();
            if modified_not.contains(&new_c) {
                return None;
            }
            match &*new_c {
                CharExpression::CharVar(new_var) => {
                    combined_not.insert(new_var.clone(), modified_not)
                }
                CharExpression::Literal(_) => None,
            };
        }

        // Include not constraints
        result.set_not_constraints(combined_not);
        Some(())
    }

    // PHASE 6: Set Range Constraints
    pub fn set_range_constraints(
        &mut self,
        range_constrs: BTreeMap<CharVar, RangeConstr>,
        result: &mut SimpleSub,
    ) {
        // Similar to handling not, just updated the char vars in range constraints using Find
        // Commented out due to index out of bounds error on Union Find
        let mut updated_range = BTreeMap::new();
        for (c, ranges) in range_constrs {
            let key = CharExpression::CharVar(c.clone());
            if !self.expr_to_id.contains_key(&Rc::new(key.clone())) {
                self.expr_to_id
                    .insert(Rc::new(key.clone()), self.expr_to_id.len() + 1);
                self.id_to_expr
                    .insert(self.expr_to_id.len(), Rc::new(key.clone()));
            }
            let id_var = self.expr_to_id[&key.clone()];
            let Some(new_var) = self.id_to_expr.get(&self.union_find.find(id_var).clone()) else {
                updated_range.insert(c, ranges);
                continue;
            };
            if let CharExpression::CharVar(name) = &**new_var {
                updated_range.insert(name.clone(), ranges);
            } else {
                updated_range.insert(c, ranges);
            }
        }
        result.set_ranges(updated_range);
    }

    /// OVERALL FUNCTION: does Phases 1-5
    pub fn merge(&mut self, substitutions: AnySub) -> Option<SimpleSub> {
        // Take range constraints
        // If merge was unsuccessful, return None
        let mut substitutions = substitutions;
        let range_constrs = substitutions.take_ranges()?;

        let mut str_eq_class = substitutions.get_str_map().clone();
        let char_eq_class = substitutions.get_char_map().clone();

        self.merge_string_maps(&mut str_eq_class)?;

        let char_set = self.merge_char_maps(&char_eq_class)?;

        let mut combined_expr: SimpleSub = SimpleSub::empty();

        self.set_char_maps(&char_set, &mut combined_expr);
        self.set_str_maps(&mut str_eq_class, &mut combined_expr);

        // Update not constraints using Find. Check for invalid not constraints. Put not constraints into combined_expr
        self.set_not_constraints(&substitutions, &mut combined_expr);

        // Include range constraints
        self.set_range_constraints(range_constrs, &mut combined_expr);

        Some(combined_expr)
    }

    // Helper function #1
    fn union_over_set(&mut self, union_set: &HashSet<Rc<CharExpression>>) -> bool {
        let mut prev: Option<Rc<CharExpression>> = None;

        for element in union_set {
            if let Some(prev_exists) = prev {
                if !self.union_exprs(prev_exists, element.clone()) {
                    return false;
                }
            }
            prev = Some(element.clone())
        }
        true
    }

    // Helper function #2
    pub fn find_set(&mut self, queries: &BTreeSet<CharExpression>) -> BTreeSet<CharExpression> {
        let mut ret_set = BTreeSet::new();
        for query in queries {
            let query = Rc::new(query.clone());
            let canonical_repr = self.find_expr(query);
            ret_set.insert((*canonical_repr).clone());
        }
        ret_set
    }

    // Helper function #3
    // Assigns ids to expr_1 and expr_2 if needed
    // Unions expr_1 and expr_2 together in union_find
    // Updates canonical_map accordingly
    // Returns true if successful, false otherwise
    fn union_exprs(&mut self, expr_1: Rc<CharExpression>, expr_2: Rc<CharExpression>) -> bool {
        let id_1 = self.get_id_from_expr(expr_1.clone());
        let id_2 = self.get_id_from_expr(expr_2.clone());
        //Stores old canonical representives in case of literals
        let old_canonical_expr_1 = self.find_expr(expr_1.clone());
        let old_canonical_expr_2 = self.find_expr(expr_2.clone());
        //Removes canonical mapping from old UF representitives
        self.canonical_map.remove(&self.union_find.find(id_1));
        self.canonical_map.remove(&self.union_find.find(id_2));
        self.union_find.union(id_1, id_2);
        let representative_id = self.union_find.find(id_1);
        //Assigns canonical representative for new UF representitive
        let old_canonical_id_1 = self.get_id_from_expr(old_canonical_expr_1.clone());
        let old_canonical_id_2 = self.get_id_from_expr(old_canonical_expr_2.clone());
        match (old_canonical_expr_1.as_ref(), old_canonical_expr_2.as_ref()) {
            (CharExpression::CharVar(_), CharExpression::CharVar(_)) => self
                .canonical_map
                .insert(representative_id, representative_id),
            (CharExpression::CharVar(_), CharExpression::Literal(_)) => self
                .canonical_map
                .insert(representative_id, old_canonical_id_2),
            (CharExpression::Literal(_), CharExpression::CharVar(_)) => self
                .canonical_map
                .insert(representative_id, old_canonical_id_1),
            (CharExpression::Literal(lit_1), CharExpression::Literal(lit_2)) => {
                if lit_1 != lit_2 {
                    return false;
                } else {
                    self.canonical_map
                        .insert(representative_id, old_canonical_id_1)
                }
            }
        };
        true
    }

    // Helper function #4
    // Finds canonical representative of expr
    fn find_expr(&mut self, expr: Rc<CharExpression>) -> Rc<CharExpression> {
        let expr_id = self.get_id_from_expr(expr);
        let canonical_id = self.get_canonical_id(expr_id);
        match self.get_expr_from_id(canonical_id) {
            Some(canonical_expr) => canonical_expr.clone(),
            None => unreachable!(),
        }
    }

    // Helper function #5
    // Find and returns id of expr from expr_to_id
    // Assigns id if needed
    fn get_id_from_expr(&mut self, expr: Rc<CharExpression>) -> usize {
        let id: usize;
        if self.expr_to_id.contains_key(&expr) {
            id = self.expr_to_id[&expr];
        } else {
            id = self.expr_to_id.len() + 1;
            self.expr_to_id.insert(expr.clone(), id);
            self.id_to_expr.insert(id, expr.clone());
        }
        id
    }

    // Helper function #6
    fn get_expr_from_id(&mut self, id: usize) -> Option<&Rc<CharExpression>> {
        self.id_to_expr.get(&id)
    }

    //Helper function #7
    //Returns canonical id of an input id
    //Does so by getting the UF representstive, then getting the value of the UF representive in canonical map
    //Assigns canonical mapping if id not in canonical map
    fn get_canonical_id(&mut self, id: usize) -> usize {
        let repr_id = self.union_find.find(id);
        match self.canonical_map.get(&repr_id) {
            Some(canonical_id) => *canonical_id,
            None => {
                self.canonical_map.insert(repr_id, repr_id);
                repr_id
            }
        }
    }
}
