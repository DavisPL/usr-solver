//!
//! Substitution terms, used for Antimirov derivatives
//!

use crate::types::expr::{CharExpression, CharVar, StringVar};
use crate::types::regex::GenRegex;

use std::cmp::{max, min};
use std::collections::{BTreeMap, HashSet};
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
    pub fn new(deriv_expression: Rc<GenRegex>, subs: SimpleSub) -> Self {
        Self {
            deriv_expression,
            subs,
        }
    }
    pub fn new_epsilon() -> Self {
        Self {
            deriv_expression: GenRegex::epsilon(),
            subs: SimpleSub::empty(),
        }
    }

    pub fn add_range(&mut self, key: CharVar, start: char, end: char) {
        self.subs.add_range(key, start, end);
    }
    pub fn get_expr(&self) -> &Rc<GenRegex> {
        &self.deriv_expression
    }
    pub fn get_subs(&self) -> &SimpleSub {
        &self.subs
    }
    pub fn get_ranges(&self) -> &BTreeMap<CharVar, RangeConstr> {
        self.subs.get_ranges()
    }

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
        let start = max(self.start, other.start);
        let end = min(self.end, other.end);
        if start < end {
            Some(Self { start, end })
        } else {
            None
        }
    }
}

fn merge_range_constraints(
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
}

impl AnySub {
    pub fn get_str_map(&self) -> &BTreeMap<StringVar, Vec<SubExpr>> {
        &self.string_to
    }
    pub fn get_char_map(&self) -> &BTreeMap<CharVar, Vec<CharExpression>> {
        &self.char_to
    }
    pub fn take_ranges(&mut self) -> Option<BTreeMap<CharVar, RangeConstr>> {
        self.range_constraints.take()
    }
}

/// Represents a simple sub (in a normalized form)
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SimpleSub {
    string_to: BTreeMap<StringVar, SubExpr>,
    char_to: BTreeMap<CharVar, CharExpression>,
    range_constraints: BTreeMap<CharVar, RangeConstr>,
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
    ) -> Self {
        SimpleSub {
            string_to,
            char_to,
            range_constraints,
        }
    }
    pub fn empty() -> Self {
        // Sub with empty HashMaps
        SimpleSub {
            string_to: BTreeMap::new(),
            char_to: BTreeMap::new(),
            range_constraints: BTreeMap::new(),
        }
    }

    /*
        Union operation
    */
    pub fn union(self, other: SimpleSub) -> AnySub {
        let mut combined_string_to: BTreeMap<StringVar, Vec<SubExpr>> = BTreeMap::new();
        let mut combined_char_to: BTreeMap<CharVar, Vec<CharExpression>> = BTreeMap::new();

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

        let range_constrs =
            merge_range_constraints(&self.range_constraints, &other.range_constraints);

        AnySub {
            string_to: combined_string_to,
            char_to: combined_char_to,
            range_constraints: range_constrs,
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
