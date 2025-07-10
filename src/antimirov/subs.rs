//!
//! Substitution terms, used for Antimirov derivatives
//!

// TODO: Fix and remove
#![allow(dead_code)]

use crate::types::expr::{CharExpression, CharVar, MaybeCharExpression, StringIndex, StringVar};
use crate::types::predicate::Predicate;
use crate::types::regex::GenRegex;

use std::cmp::{max, min};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::ops::Index;
use std::rc::Rc;

/*
    Antimirov derivative terms
*/

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct AntimirovElement {
    deriv_expression: Rc<GenRegex>,
    subs: SimpleSub,
}

impl AntimirovElement {
    /*
        Main constructors
    */

    pub fn new(deriv_expression: Rc<GenRegex>, subs: SimpleSub) -> Self {
        Self {
            deriv_expression,
            subs,
        }
    }

    pub fn new_emptysub(deriv_expression: Rc<GenRegex>) -> Self {
        Self {
            deriv_expression,
            subs: SimpleSub::empty(),
        }
    }

    pub fn new_epsilon() -> Self {
        Self::new_emptysub(GenRegex::epsilon())
    }

    pub fn new_empty() -> Self {
        Self::new_emptysub(GenRegex::empty_set())
    }

    /*
        Range expressions
    */

    pub fn add_range(&mut self, key: CharVar, start: char, end: char) {
        self.subs.add_range(key, start, end);
    }

    pub fn new_epsilon_range(key: CharVar, start: char, end: char) -> Self {
        let mut result = Self::new_epsilon();
        result.add_range(key, start, end);
        result
    }

    pub fn new_empty_range(key: CharVar, start: char, end: char) -> Self {
        let mut result = Self::new_empty();
        result.add_range(key, start, end);
        result
    }

    /*
        Merge

        This operation comes up repeatedly (using merge on substitutions),
        so should be useful to have a generic wrapper for it.
    */

    pub fn merge_using<F>(
        left: &AntimirovElement,
        right: &AntimirovElement,
        merge_fun: F,
    ) -> Option<AntimirovElement>
    where
        F: Fn(&Rc<GenRegex>, &Rc<GenRegex>) -> Option<Rc<GenRegex>>,
    {
        // Calculate subs
        let l_subs = left.get_subs();
        let r_subs = right.get_subs();
        let subs = merge_binary(l_subs, r_subs)?;
        let l_sub_diff = sub_difference_from_merge(&subs, l_subs);
        let r_sub_diff = sub_difference_from_merge(&subs, r_subs);

        // Calculate left and right expressions
        let l_expr = sub_in(left.get_expr(), &l_sub_diff);
        let r_expr = sub_in(right.get_expr(), &r_sub_diff);

        // Apply merge function
        let merged = merge_fun(&l_expr, &r_expr)?;
        let result = AntimirovElement::new(merged, subs);
        Some(result)
    }

    // Safe version which enforces that the merge should return Some(..) iff left.subs and right.subs
    // are consistent (have nonempty overlap)
    // This version errors on on failure to calculate sub difference
    pub fn merge_using_safe<F>(
        left: &AntimirovElement,
        right: &AntimirovElement,
        merge_fun: F,
    ) -> Option<AntimirovElement>
    where
        F: Fn(&Rc<GenRegex>, &Rc<GenRegex>) -> Rc<GenRegex>,
    {
        // Calculate subs
        let l_subs = left.get_subs();
        let r_subs = right.get_subs();
        let subs = merge_binary(l_subs, r_subs)?;
        let l_sub_diff = sub_difference_from_merge(&subs, l_subs);
        let r_sub_diff = sub_difference_from_merge(&subs, r_subs);

        // Calculate left and right expressions
        let l_expr = sub_in(left.get_expr(), &l_sub_diff);
        let r_expr = sub_in(right.get_expr(), &r_sub_diff);

        // Apply merge function
        let merged = merge_fun(&l_expr, &r_expr);
        let result = AntimirovElement::new(merged, subs);
        Some(result)
    }

    /*
        Getters
    */

    pub fn get_expr(&self) -> &Rc<GenRegex> {
        &self.deriv_expression
    }
    pub fn get_subs(&self) -> &SimpleSub {
        &self.subs
    }
    pub fn get_ranges(&self) -> &BTreeMap<CharVar, RangeConstr> {
        self.subs.get_ranges()
    }

    /*
        Setters
    */

    pub fn map_expr<F>(self, map_fun: F) -> Self
    where
        F: Fn(Rc<GenRegex>) -> Rc<GenRegex>,
    {
        let Self {
            deriv_expression,
            subs,
        } = self;
        let deriv_expression = map_fun(deriv_expression);
        Self {
            deriv_expression,
            subs,
        }
    }

    /*
        Consumers
    */

    pub fn into_set(self) -> HashSet<Self> {
        HashSet::from([self])
    }
}

/*
    Range constraints
*/

/// Optimization to represent ranges as constraints on subs
// example: d([a-z], x) = {(epsilon, x->a), (epsilon, x->b) ... (epsilon, x->z)}
// Store only: x, [a, z], {(epsilon, {})}.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct RangeConstr {
    start: char,
    end: char,
}
impl RangeConstr {
    pub fn new(start: char, end: char) -> Self {
        RangeConstr { start, end }
    }
    pub fn get_start(&self) -> &char {
        &self.start
    }
    pub fn get_end(&self) -> &char {
        &self.end
    }
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        // println!("INTERSECT: {} and {}", self, other);
        let start = max(self.start, other.start);
        let end = min(self.end, other.end);
        if start <= end {
            // println!("INTERSECTED RESULT: [{}, {}]", start, end);
            Some(Self { start, end })
        } else {
            // println!("INTERSECTED RESULT: None");
            None
        }
    }
}

pub fn merge_range_constraints(
    constraints1: &BTreeMap<CharVar, RangeConstr>,
    constraints2: &BTreeMap<CharVar, RangeConstr>,
) -> Option<BTreeMap<CharVar, RangeConstr>> {
    let mut constraints = constraints1.clone();
    for (key, val) in constraints2 {
        if let Some(other_val) = constraints.get(key) {
            if let Some(merged_range) = val.intersect(other_val) {
                constraints.insert(key.clone(), merged_range);
            } else {
                return None;
            }
        } else {
            constraints.insert(key.clone(), val.clone());
        }
    }
    Some(constraints)
}

/*
    Substitution expressions and substitution classes
*/

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SubExpr {
    head: Vec<CharExpression>,
    tail_is_string_var: bool,
}

impl Index<usize> for SubExpr {
    type Output = CharExpression;

    fn index(&self, index: usize) -> &Self::Output {
        &self.head[index]
    }
}

impl SubExpr {
    pub fn to_gen_regex(&self, tail_var: &StringVar) -> Rc<GenRegex> {
        let head = Self::to_gen_regex_helper(self.get_head());
        if self.get_tail() {
            Rc::new(GenRegex::Concatenation(
                head,
                Rc::new(GenRegex::StringVar(tail_var.clone())),
            ))
        } else {
            head
        }
    }
    fn to_gen_regex_helper(head: &[CharExpression]) -> Rc<GenRegex> {
        let split = head.split_first();
        match split {
            Some((first, rest)) => {
                let ret_val = Rc::new(GenRegex::CharExpression(first.clone()));
                if rest.to_vec().len() == 1 {
                    ret_val
                } else {
                    Rc::new(GenRegex::Concatenation(
                        ret_val,
                        Self::to_gen_regex_helper(rest),
                    ))
                }
            }
            None => GenRegex::epsilon(),
        }
    }
    pub fn get_head(&self) -> &Vec<CharExpression> {
        &self.head
    }
    pub fn get_mut_head(&mut self) -> &mut Vec<CharExpression> {
        &mut self.head
    }
    pub fn get_tail(&self) -> bool {
        self.tail_is_string_var
    }

    pub fn head_length(&self) -> usize {
        self.head.len()
    }
    pub fn set_head(&mut self, new_head: Vec<CharExpression>) {
        self.head = new_head;
    }
    pub fn is_empty(&self) -> bool {
        self.head.len() == 0
    }
    pub fn empty() -> Self {
        SubExpr {
            head: Vec::new(),
            tail_is_string_var: false,
        }
    }
    pub fn new(head: Vec<CharExpression>, tail_is_string_var: bool) -> Self {
        SubExpr {
            head,
            tail_is_string_var,
        }
    }
}

/// Represents any sub, not necessarily simple (e.g. x -> y, y -> x)
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct AnySub {
    string_to: BTreeMap<StringVar, Vec<SubExpr>>,
    char_to: BTreeMap<CharVar, Vec<CharExpression>>,
    range_constraints: Option<BTreeMap<CharVar, RangeConstr>>,
    not_constraints: BTreeMap<CharVar, BTreeSet<CharExpression>>,
}

impl AnySub {
    pub fn get_str_map(&self) -> &BTreeMap<StringVar, Vec<SubExpr>> {
        &self.string_to
    }
    pub fn get_char_map(&self) -> &BTreeMap<CharVar, Vec<CharExpression>> {
        &self.char_to
    }
    pub fn get_ranges(&self) -> &Option<BTreeMap<CharVar, RangeConstr>> {
        &self.range_constraints
    }
    pub fn take_ranges(&mut self) -> Option<BTreeMap<CharVar, RangeConstr>> {
        self.range_constraints.take()
    }
    pub fn get_not_constraints(&self) -> &BTreeMap<CharVar, BTreeSet<CharExpression>> {
        &self.not_constraints
    }
}

/// Represents a simple sub (in a normalized form)
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SimpleSub {
    string_to: BTreeMap<StringVar, SubExpr>,
    char_to: BTreeMap<CharVar, CharExpression>,
    range_constraints: BTreeMap<CharVar, RangeConstr>,
    not_constraints: BTreeMap<CharVar, BTreeSet<CharExpression>>,
}

impl Index<&StringVar> for SimpleSub {
    type Output = SubExpr;

    fn index(&self, _index: &StringVar) -> &Self::Output {
        unimplemented!()
    }
}

impl SimpleSub {
    /*
        Constructors
    */
    pub fn new(
        string_to: BTreeMap<StringVar, SubExpr>,
        char_to: BTreeMap<CharVar, CharExpression>,
        range_constraints: BTreeMap<CharVar, RangeConstr>,
        not_constraints: BTreeMap<CharVar, BTreeSet<CharExpression>>,
    ) -> Self {
        SimpleSub {
            string_to,
            char_to,
            range_constraints,
            not_constraints,
        }
    }
    pub fn empty() -> Self {
        // Sub with empty HashMaps
        SimpleSub {
            string_to: BTreeMap::new(),
            char_to: BTreeMap::new(),
            range_constraints: BTreeMap::new(),
            not_constraints: BTreeMap::new(),
        }
    }

    /*
        Union operation
    */
    pub fn union(self, other: SimpleSub) -> AnySub {
        let mut combined_string_to: BTreeMap<StringVar, Vec<SubExpr>> = BTreeMap::new();
        let mut combined_char_to: BTreeMap<CharVar, Vec<CharExpression>> = BTreeMap::new();
        let mut combined_not_constraints: BTreeMap<CharVar, BTreeSet<CharExpression>> =
            BTreeMap::new();

        for (key, value) in self.string_to {
            combined_string_to.entry(key).or_default().push(value);
        }
        for (key, value) in other.string_to {
            combined_string_to.entry(key).or_default().push(value);
        }

        for (key, value) in self.char_to {
            combined_char_to.entry(key).or_default().push(value);
        }
        for (key, value) in other.char_to {
            combined_char_to.entry(key).or_default().push(value);
        }

        for (key, value) in self.not_constraints {
            combined_not_constraints
                .entry(key)
                .or_default()
                .extend(value);
        }
        for (key, value) in other.not_constraints {
            combined_not_constraints
                .entry(key)
                .or_default()
                .extend(value);
        }

        let range_constrs =
            merge_range_constraints(&self.range_constraints, &other.range_constraints);

        AnySub {
            string_to: combined_string_to,
            char_to: combined_char_to,
            range_constraints: range_constrs,
            not_constraints: combined_not_constraints,
        }
    }

    /*
        Getters and Setters
    */
    pub fn get_str_map(&self) -> &BTreeMap<StringVar, SubExpr> {
        &self.string_to
    }
    pub fn get_str_map_mut(&mut self) -> &mut BTreeMap<StringVar, SubExpr> {
        &mut self.string_to
    }
    pub fn remove_str_map(&mut self, key: &StringVar) -> Option<SubExpr> {
        self.string_to.remove(key)
    }
    pub fn get_str_var(&self, key: &StringVar) -> Option<&SubExpr> {
        self.string_to.get(key)
    }
    pub fn set_str_var(&mut self, key: StringVar, value: SubExpr) {
        self.string_to.insert(key, value);
    }

    pub fn get_char_map(&self) -> &BTreeMap<CharVar, CharExpression> {
        &self.char_to
    }
    pub fn get_char_map_mut(&mut self) -> &mut BTreeMap<CharVar, CharExpression> {
        &mut self.char_to
    }
    pub fn remove_char_map(&mut self, key: &CharVar) -> Option<CharExpression> {
        self.char_to.remove(key)
    }
    pub fn get_char_var(&self, key: &CharVar) -> Option<&CharExpression> {
        self.char_to.get(key)
    }
    pub fn set_char_var(&mut self, key: CharVar, value: CharExpression) {
        self.char_to.insert(key, value);
    }

    pub fn get_ranges(&self) -> &BTreeMap<CharVar, RangeConstr> {
        &self.range_constraints
    }
    pub fn set_ranges(&mut self, ranges: BTreeMap<CharVar, RangeConstr>) {
        self.range_constraints = ranges;
    }
    pub fn add_range(&mut self, key: CharVar, start: char, end: char) {
        let value = RangeConstr::new(start, end);
        self.range_constraints.insert(key, value);
    }

    pub fn get_not_constraints(&self) -> &BTreeMap<CharVar, BTreeSet<CharExpression>> {
        &self.not_constraints
    }

    pub fn set_not_constraints(&mut self, not_constr: BTreeMap<CharVar, BTreeSet<CharExpression>>) {
        self.not_constraints = not_constr;
    }

    /*
        Consumers
    */
    pub fn into_set(self) -> HashSet<SimpleSub> {
        HashSet::from([self])
    }
}

/*
    Pretty printing
*/

use std::fmt::{self, Display};

impl Display for SubExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for val in self.get_head() {
            write!(f, "{}", val)?;
        }
        write!(f, "{}", self.get_tail())
    }
}
impl Display for AnySub {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ANYSUB string_to: {{ ")?;
        for (key, value) in self.get_str_map() {
            for v in value {
                write!(f, "{} => {}, ", key, v)?;
            }
        }
        writeln!(f, "}}")?;

        write!(f, "char_to: {{ ")?;
        for (key, value) in self.get_char_map() {
            for v in value {
                write!(f, "{} => {}, ", key, v)?;
            }
        }
        write!(f, "}}")?;

        Ok(())
    }
}

impl Display for SimpleSub {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "string_to: {{ ")?;
        for (key, value) in self.get_str_map() {
            write!(f, "{} => {}, ", key, value)?;
        }
        writeln!(f, "}}")?;

        write!(f, "char_to: {{ ")?;
        for (key, value) in self.get_char_map() {
            write!(f, "{} => {}, ", key, value)?;
        }
        write!(f, "}}")?;

        Ok(())
    }
}

impl Display for RangeConstr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}-{}]", self.get_start(), self.get_end())
    }
}

impl Display for AntimirovElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}", self.get_expr(), self.get_subs())?;
        for (var, range) in self.get_ranges() {
            write!(f, ", {}: {}", var, range)?;
        }
        write!(f, ")")
    }
}

/*
    Substitution operations: merge, difference, and sub_in
*/

use super::union_find::{count_union_elems, union_over_set, UnionFind};
use std::collections::HashMap;

fn merge(substitutions: AnySub) -> Option<SimpleSub> {
    //let mut union_set: HashSet<Rc<CharExpression>> = HashSet::new();
    let mut expr_to_id: HashMap<Rc<CharExpression>, usize> = HashMap::new();
    let mut id_to_expr: HashMap<usize, Rc<CharExpression>> = HashMap::new();
    let mut canonical_map: HashMap<Rc<CharExpression>, Rc<CharExpression>> = HashMap::new();
    let mut union_find: UnionFind<usize> = UnionFind::new(count_union_elems(&substitutions) + 1);

    // Take range constraints
    // If merge was unsuccessful, return None
    let mut substitutions = substitutions;
    let range_constrs = substitutions.take_ranges()?;
    let substitutions = substitutions;

    let mut str_eq_class = substitutions.get_str_map().clone();
    let char_eq_class = substitutions.get_char_map().clone();

    for eq_exprs in str_eq_class.values_mut() {
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
            if !union_over_set(
                &mut union_find,
                &union_set,
                &mut expr_to_id,
                &mut id_to_expr,
                &mut canonical_map,
            ) {
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

        if !union_over_set(
            &mut union_find,
            &u_set,
            &mut expr_to_id,
            &mut id_to_expr,
            &mut canonical_map,
        ) {
            return None;
        }
        char_set = char_set.union(&temp_set).cloned().collect();
    }
    for var in char_set {
        let deref = var.as_ref();
        let char_expression = Rc::new(CharExpression::CharVar(var.as_ref().clone()));
        let id_var = expr_to_id[&char_expression];
        let found_expr = id_to_expr[&union_find.find(id_var)].clone();
        match canonical_map.get(&char_expression) {
            Some(value) => combined_expr.set_char_var(var.as_ref().clone(), value.as_ref().clone()),
            None => {
                if CharExpression::CharVar(deref.clone()) != *found_expr {
                    combined_expr.set_char_var(deref.clone(), found_expr.as_ref().clone());
                }
            }
        }
    }

    for (var, mut eq_exprs) in str_eq_class {
        let sub_expr_vector = eq_exprs[0].get_mut_head();
        for (i, item) in sub_expr_vector.iter_mut().enumerate() {
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
    for (c, not_constraint_set) in substitutions.not_constraints {
        let modified_not = find_set(not_constraint_set, &union_find, &expr_to_id, &id_to_expr);
        let key = CharExpression::CharVar(c.clone());
        if !expr_to_id.contains_key(&Rc::new(key.clone())) {
            expr_to_id.insert(Rc::new(key.clone()), expr_to_id.len() + 1);
            id_to_expr.insert(expr_to_id.len(), Rc::new(key.clone()));
        }
        let id_var = expr_to_id[&key.clone()];
        let new_c = id_to_expr[&union_find.find(id_var).clone()].clone();
        if modified_not.contains(&new_c) {
            return None;
        }
        // Note: We insert c instead of new_c here which means that not all keys are fully resolved. Can be revisted later.
        match &*new_c {
            CharExpression::CharVar(new_var) => combined_not.insert(new_var.clone(), modified_not),
            CharExpression::Literal(_) => todo!(),
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
        if !expr_to_id.contains_key(&Rc::new(key.clone())) {
            expr_to_id.insert(Rc::new(key.clone()), expr_to_id.len() + 1);
            id_to_expr.insert(expr_to_id.len(), Rc::new(key.clone()));
        }
        let id_var = expr_to_id[&key.clone()];
        let Some(new_var) = id_to_expr.get(&union_find.find(id_var).clone()) else {
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

pub fn find_set(
    queries: BTreeSet<CharExpression>,
    union_find: &UnionFind,
    expr_to_id: &HashMap<Rc<CharExpression>, usize>,
    id_to_expr: &HashMap<usize, Rc<CharExpression>>,
) -> BTreeSet<CharExpression> {
    let mut ret_set = BTreeSet::new();
    for query in queries {
        if !expr_to_id.contains_key(&query) {
            ret_set.insert(query);
        } else {
            let id_var = expr_to_id[&query];
            ret_set.insert((*id_to_expr[&union_find.find(id_var)]).clone());
        }
    }
    ret_set
}

pub fn merge_binary(sub1: &SimpleSub, sub2: &SimpleSub) -> Option<SimpleSub> {
    let union_lr: AnySub = sub1.clone().union(sub2.clone());
    merge(union_lr)
}

pub fn merge_sets(subs1: &HashSet<SimpleSub>, subs2: &HashSet<SimpleSub>) -> HashSet<SimpleSub> {
    let mut result = HashSet::new();
    for sub1 in subs1 {
        for sub2 in subs2 {
            if let Some(ret) = merge_binary(sub1, sub2) {
                result.insert(ret);
            }
        }
    }
    result
}

pub fn union_sets(subs1: HashSet<SimpleSub>, subs2: HashSet<SimpleSub>) -> HashSet<SimpleSub> {
    let mut result = subs1;
    result.extend(subs2);
    result
}

/// PRECONDITION: merged is the merge of sub and sub2
/// If the precondition holds, this should never panic
pub fn sub_difference_from_merge(merged: &SimpleSub, sub: &SimpleSub) -> SimpleSub {
    let mut retsub = merged.clone();
    for char_var in sub.get_char_map().keys() {
        retsub.remove_char_map(char_var);
    }
    for (string_var, sub_expr1) in merged.get_str_map() {
        if let Some(sub_expr2) = sub.get_str_var(string_var) {
            retsub.remove_str_map(string_var);
            if let Some(mut sub) = sub_expr_match(sub_expr1, sub_expr2, string_var) {
                retsub.get_char_map_mut().append(sub.get_char_map_mut());
                retsub.get_str_map_mut().append(sub.get_str_map_mut());
            } else {
                panic!(
                    "sub_difference_from_merge failed: was it called with the right precondition?"
                );
            }
        }
    }
    retsub
}

// Note: no longer used atm in favor of sub_difference_from_merge
// fn sub_difference(sub1: &SimpleSub, sub2: &SimpleSub) -> Option<SimpleSub> {
//     if let Some(result) = merge_binary(sub1, sub2) {
//         sub_difference_from_merge(&result, sub2)
//     } else {
//         None
//     }
// }

fn sub_expr_match(
    sub_expr1: &SubExpr,
    sub_expr2: &SubExpr,
    str_var: &StringVar,
) -> Option<SimpleSub> {
    let mut retval = SimpleSub::empty();
    if sub_expr1.is_empty() && sub_expr2.is_empty() {
        return Some(retval);
    } else if sub_expr1.head_length() == 0 && sub_expr1.get_tail() {
        retval.set_str_var(str_var.clone(), sub_expr2.clone());
        return Some(retval);
    } else if sub_expr2.head_length() == 0 && sub_expr2.get_tail() {
        retval.set_str_var(str_var.clone(), sub_expr1.clone());
        return Some(retval);
    } else if sub_expr1.is_empty() || sub_expr2.is_empty() {
        return None;
    }
    let trunc_sub_expr1 = SubExpr::new(sub_expr1.get_head()[1..].to_vec(), sub_expr1.get_tail());
    let trunc_sub_expr2 = SubExpr::new(sub_expr2.get_head()[1..].to_vec(), sub_expr2.get_tail());
    match sub_expr_match(&trunc_sub_expr1, &trunc_sub_expr2, str_var) {
        Some(val) => retval = val,
        None => return None,
    }
    let head1 = &sub_expr1.get_head()[0];
    let head2 = &sub_expr2.get_head()[0];
    if let CharExpression::CharVar(key) = head1 {
        retval.set_char_var(key.clone(), head2.clone());
    } else if let CharExpression::CharVar(key) = head2 {
        retval.set_char_var(key.clone(), head1.clone());
    }
    Some(retval)
}

pub fn sub_in(expr: &Rc<GenRegex>, substitution: &SimpleSub) -> Rc<GenRegex> {
    if substitution.get_str_map().is_empty() && substitution.get_char_map().is_empty() {
        return expr.clone(); // Returns a clone of expr.
    }
    match expr.as_ref() {
        GenRegex::EmptySet => Rc::clone(expr),
        GenRegex::Epsilon => Rc::clone(expr),
        GenRegex::Sigma => Rc::clone(expr),
        GenRegex::SigmaStar => Rc::clone(expr),
        GenRegex::Range(_, _) => Rc::clone(expr),
        GenRegex::CharExpression(char_expr) => match char_expr {
            CharExpression::CharVar(char_var) => match substitution.get_char_var(char_var) {
                Some(value) => Rc::new(GenRegex::CharExpression(value.clone())),
                None => expr.clone(),
            },
            CharExpression::Literal(_) => expr.clone(),
        },
        GenRegex::StringVar(string_var) => match substitution.get_str_var(string_var) {
            Some(value) => value.to_gen_regex(string_var),
            None => expr.clone(),
        },
        GenRegex::StringIndex(string_index) => match substitution.get_str_var(&string_index.var) {
            Some(value) => {
                let index = string_index.index as usize;
                let length = value.get_head().len();
                if index < length {
                    Rc::new(GenRegex::CharExpression(value.get_head()[index].clone()))
                } else if value.get_tail() {
                    Rc::new(GenRegex::StringIndex(StringIndex {
                        var: string_index.var.clone(),
                        index: ((index - length + 1) as i32),
                    }))
                } else {
                    Rc::new(GenRegex::EmptySet)
                }
            }
            None => expr.clone(),
        },
        GenRegex::Union(gen_regex1, gen_regex2) => Rc::new(GenRegex::Union(
            sub_in(gen_regex1, substitution),
            sub_in(gen_regex2, substitution),
        )),
        GenRegex::Intersect(gen_regex1, gen_regex2) => Rc::new(GenRegex::Intersect(
            sub_in(gen_regex1, substitution),
            sub_in(gen_regex2, substitution),
        )),
        GenRegex::Concatenation(gen_regex1, gen_regex2) => GenRegex::make_concatenation(
            sub_in(gen_regex1, substitution),
            sub_in(gen_regex2, substitution),
        ),
        GenRegex::Kleene(gen_regex) => Rc::new(GenRegex::Kleene(sub_in(gen_regex, substitution))),
        GenRegex::Complement(gen_regex) => {
            Rc::new(GenRegex::Complement(sub_in(gen_regex, substitution)))
        }
        GenRegex::IfThenElse(predicate, gen_regex1, gen_regex2) => {
            // TODO 6: Optional
            eprintln!("TODO: Antimirov derivative does not currently fully support IfThenElse for substitutions");
            unimplemented!()
        }
        GenRegex::StringSlice(string_var, _) => {
            // TODO 7: Optional
            eprintln!("TODO: Antimirov derivative does not currently fully support StringSlice for substitutions");
            unimplemented!()
        }
    }
}

pub fn sub_in_predicate(pred: &Rc<Predicate>, sub: &SimpleSub) -> Rc<Predicate> {
    match pred.as_ref() {
        Predicate::True => Rc::clone(pred),
        Predicate::False => Rc::clone(pred),
        Predicate::Not(p) => Rc::new(Predicate::Not(sub_in_predicate(p, sub))),
        Predicate::And(p1, p2) => Rc::new(Predicate::And(
            sub_in_predicate(p1, sub),
            sub_in_predicate(p2, sub),
        )),
        Predicate::Or(p1, p2) => Rc::new(Predicate::Or(
            sub_in_predicate(p1, sub),
            sub_in_predicate(p2, sub),
        )),
        Predicate::Equals(expr1, expr2) => {
            let new_expr1 = sub_in_maybe_char_expr(expr1, sub);
            let new_expr2 = sub_in_maybe_char_expr(expr2, sub);
            Rc::new(Predicate::Equals(new_expr1, new_expr2))
        }
        Predicate::LessThan(expr, c) => {
            let new_expr = sub_in_maybe_char_expr(expr, sub);
            Rc::new(Predicate::LessThan(new_expr, *c))
        }
        Predicate::GreaterThan(expr, c) => {
            let new_expr = sub_in_maybe_char_expr(expr, sub);
            Rc::new(Predicate::LessThan(new_expr, *c))
        }
        Predicate::EqualLength(var, len) => {
            // TODO
            unimplemented!()
            // let new_var = sub_in_string_var(var, sub);
            // Rc::new(Predicate::EqualLength(new_var, *len))
        }
    }
}

fn sub_in_maybe_char_expr(expr: &MaybeCharExpression, sub: &SimpleSub) -> Rc<MaybeCharExpression> {
    match expr {
        MaybeCharExpression::CharExpression(c_expr) => {
            let new_expr = sub_in_char_expr(c_expr, sub);
            Rc::new(MaybeCharExpression::CharExpression(new_expr))
        }
        MaybeCharExpression::StringIndex(_string_index) => {
            // TODO 8: Optional
            eprintln!("TODO: Antimirov derivative does not currently fully support String Index for substitutions");
            unimplemented!()
        }
    }
}

fn sub_in_char_expr(expr: &CharExpression, sub: &SimpleSub) -> CharExpression {
    match expr {
        CharExpression::CharVar(var) => sub.get_char_var(var).unwrap_or(expr).clone(),
        CharExpression::Literal(_) => expr.clone(),
    }
}
