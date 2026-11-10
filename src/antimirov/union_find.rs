//!
//! Union-find functions
//!

use crate::types::expr::{CharExpression, CharVar};

use super::subs::{AnySub, SimpleSub};

pub use disjoint_sets::UnionFind;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct SubMerger {
    union_find: UnionFind<usize>,
    union_set: HashSet<Rc<CharExpression>>,
    expr_to_id: HashMap<Rc<CharExpression>, usize>,
    id_to_expr: HashMap<usize, Rc<CharExpression>>,
    canonical_map: HashMap<Rc<CharExpression>, Rc<CharExpression>>,
}

impl SubMerger {
    pub fn new_for_sub(substitutions: &AnySub) -> Self {
        Self::new_with_capacity(substitutions.count_union_elems() + 1)
    }

    fn new_with_capacity(cap: usize) -> Self {
        let union_set: HashSet<Rc<CharExpression>> = HashSet::new();
        let expr_to_id: HashMap<Rc<CharExpression>, usize> = HashMap::new();
        let id_to_expr: HashMap<usize, Rc<CharExpression>> = HashMap::new();
        let canonical_map: HashMap<Rc<CharExpression>, Rc<CharExpression>> = HashMap::new();
        let union_find: UnionFind<usize> = UnionFind::new(cap);
        Self {
            union_find,
            union_set,
            expr_to_id,
            id_to_expr,
            canonical_map,
        }
    }

    fn union_over_set(&mut self) -> bool {
        let mut prev: Option<Rc<CharExpression>> = None;

        for element in &self.union_set {
            if matches!(element.as_ref(), CharExpression::Literal(_)) {
                self.canonical_map.insert(element.clone(), element.clone());
            }

            if let Some(prev_exists) = prev {
                let prev_id: usize;
                let curr_id: usize;
                if self.expr_to_id.contains_key(&prev_exists) {
                    prev_id = self.expr_to_id[&prev_exists];
                } else {
                    prev_id = self.expr_to_id.len() + 1;
                    self.expr_to_id.insert(prev_exists.clone(), prev_id);
                    self.id_to_expr.insert(prev_id, prev_exists.clone());
                }
                if self.expr_to_id.contains_key(element.as_ref()) {
                    curr_id = self.expr_to_id[element.as_ref()];
                } else {
                    curr_id = self.expr_to_id.len() + 1;
                    self.expr_to_id.insert(element.clone(), curr_id);
                    //self.expr_to_id[element.as_ref()] = curr_id;
                    self.id_to_expr.insert(curr_id, element.clone());
                } // By this point in the code we should have the ID for the 2 elements we are unioning
                if self.canonical_map.contains_key(&prev_exists)
                    && self.canonical_map.contains_key(element.as_ref())
                    && self.canonical_map[&prev_exists] != self.canonical_map[element.as_ref()]
                {
                    return false;
                }
                self.union_find.union(prev_id, curr_id); //ERROR HERE
                if self.canonical_map.contains_key(element.as_ref()) {
                    self.canonical_map
                        .insert(prev_exists, self.canonical_map[element.as_ref()].clone());
                } else if self.canonical_map.contains_key(&prev_exists) {
                    self.canonical_map
                        .insert(element.clone(), self.canonical_map[&prev_exists].clone());
                }
            }
            prev = Some(element.clone())
        }
        true
    }

    pub fn merge(&mut self, mut substitutions: AnySub) -> Option<SimpleSub> {
        // Take range constraints
        // If merge was unsuccessful, return None
        let range_constrs = substitutions.take_ranges()?;

        let mut str_eq_class = substitutions.get_str_map().clone();
        let char_eq_class = substitutions.get_char_map().clone();

        for eq_exprs in str_eq_class.values_mut() {
            let mut ind = 0;
            while eq_exprs.len() > 1 {
                // vvvvv Does not access &self
                let mut length_flag = false;
                // let mut union_set: HashSet<Rc<CharExpression>> = HashSet::new();
                self.union_set = HashSet::<Rc<CharExpression>>::new();
                let mut i = 0;
                while i < eq_exprs.len() {
                    let curr_sub_expr = &eq_exprs[i];
                    if ind < curr_sub_expr.head_length() {
                        let temp = &curr_sub_expr[ind];
                        self.union_set.insert(Rc::new(temp.clone()));
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
                // ^^^^^ Does not access &self

                if !self.union_over_set() {
                    return None;
                }
            }
        }

        let mut combined_expr: SimpleSub = SimpleSub::empty();
        let mut char_set = HashSet::new();
        for (var, eq_exprs) in &char_eq_class {
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
            let mut u_set: HashSet<_> = eq_exprs
                .iter()
                .map(|expr| Rc::new((expr).clone())) // Dereference `expr` (&&CharExpression) and clone
                .collect();
            u_set.insert(Rc::new(CharExpression::CharVar(var.clone())));
            temp_set.insert(Rc::new(var.clone()));

            if !self.union_over_set() {
                return None;
            }
            char_set = char_set.union(&temp_set).cloned().collect();
        }

        for var in char_set {
            let deref = var.as_ref();
            let char_expression = Rc::new(CharExpression::CharVar(var.as_ref().clone()));
            let id_var = self.expr_to_id[&char_expression];
            let found_expr = self.id_to_expr[&self.union_find.find(id_var)].clone();
            match self.canonical_map.get(&char_expression) {
                Some(value) => {
                    combined_expr.set_char_var(var.as_ref().clone(), value.as_ref().clone())
                }
                None => {
                    if CharExpression::CharVar(deref.clone()) != *found_expr {
                        combined_expr.set_char_var(deref.clone(), found_expr.as_ref().clone());
                    }
                }
            }
        }

        for (var, mut eq_exprs) in str_eq_class {
            let sub_expr_vector = eq_exprs[0].get_mut_head();
            for item in sub_expr_vector.iter_mut() {
                if let CharExpression::CharVar(c_var) = item {
                    let substitution_value = combined_expr.get_char_var(c_var);
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
            combined_expr.set_str_var(var.clone(), eq_exprs[0].clone());
        }

        // Update not constraints using Find. Check for invalid not constraints. Put not constraints into combined_expr
        let mut combined_not = BTreeMap::new();
        for (c, not_constraint_set) in substitutions.get_not_constraints() {
            let modified_not = find_set(
                not_constraint_set,
                &self.union_find,
                &self.expr_to_id,
                &self.id_to_expr,
            );
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
        combined_expr.set_not_constraints(combined_not);

        // Include range constraints
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
        combined_expr.set_ranges(updated_range);

        Some(combined_expr)
    }
}

fn find_set(
    queries: &BTreeSet<CharExpression>,
    union_find: &UnionFind,
    expr_to_id: &HashMap<Rc<CharExpression>, usize>,
    id_to_expr: &HashMap<usize, Rc<CharExpression>>,
) -> BTreeSet<CharExpression> {
    let mut ret_set = BTreeSet::new();
    for query in queries {
        if !expr_to_id.contains_key(query) {
            ret_set.insert(query.clone());
        } else {
            let id_var = expr_to_id[query];
            ret_set.insert((*id_to_expr[&union_find.find(id_var)]).clone());
        }
    }
    ret_set
}

/// TODO: remove/integrate
pub fn union_over_set_TEMP(
    union_find: &mut UnionFind<usize>,
    union_set: &HashSet<Rc<CharExpression>>,
    expr_to_id: &mut HashMap<Rc<CharExpression>, usize>,
    id_to_expr: &mut HashMap<usize, Rc<CharExpression>>,
    canonical_map: &mut HashMap<Rc<CharExpression>, Rc<CharExpression>>,
) -> bool {
    let mut prev: std::option::Option<Rc<CharExpression>> = None;

    for element in union_set {
        if matches!(element.as_ref(), CharExpression::Literal(_)) {
            canonical_map.insert(element.clone(), element.clone());
        }

        if let Some(prev_exists) = prev {
            let prev_id: usize;
            let curr_id: usize;
            if expr_to_id.contains_key(&prev_exists) {
                prev_id = expr_to_id[&prev_exists];
            } else {
                prev_id = expr_to_id.len() + 1;
                expr_to_id.insert(prev_exists.clone(), prev_id);
                id_to_expr.insert(prev_id, prev_exists.clone());
            }
            if expr_to_id.contains_key(element.as_ref()) {
                curr_id = expr_to_id[element.as_ref()];
            } else {
                curr_id = expr_to_id.len() + 1;
                expr_to_id.insert(element.clone(), curr_id);
                //expr_to_id[element.as_ref()] = curr_id;
                id_to_expr.insert(curr_id, element.clone());
            } // By this point in the code we should have the ID for the 2 elements we are unioning
            if canonical_map.contains_key(&prev_exists)
                && canonical_map.contains_key(element.as_ref())
                && canonical_map[&prev_exists] != canonical_map[element.as_ref()]
            {
                return false;
            }
            union_find.union(prev_id, curr_id); //ERROR HERE
            if canonical_map.contains_key(element.as_ref()) {
                canonical_map.insert(prev_exists, canonical_map[element.as_ref()].clone());
            } else if canonical_map.contains_key(&prev_exists) {
                canonical_map.insert(element.clone(), canonical_map[&prev_exists].clone());
            }
        }
        prev = Some(element.clone())
    }
    true
}
