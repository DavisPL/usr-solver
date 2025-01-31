//!
//! Type definitions -
//! Main GenRegex class and subclasses
//!

use std::cmp::{max, min};
use std::collections::{BTreeMap, HashSet};
use std::ops::Index;
use std::rc::Rc;

/*
    GenRegex
*/

// TODO: add a GenRegex::StringLiteral
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GenRegex {
    EmptySet,
    Epsilon,
    Sigma,
    Range(char, char),
    CharExpression(CharExpression),
    StringVar(StringVar),
    StringSlice(StringVar, i32),
    Union(Rc<GenRegex>, Rc<GenRegex>),
    Intersect(Rc<GenRegex>, Rc<GenRegex>),
    Concatenation(Rc<GenRegex>, Rc<GenRegex>),
    Kleene(Rc<GenRegex>),
    Complement(Rc<GenRegex>),
    IfThenElse(Rc<Predicate>, Rc<GenRegex>, Rc<GenRegex>),
    StringIndex(StringIndex),
}

impl GenRegex {
    pub fn create_sigma() -> Rc<GenRegex> {
        Rc::new(GenRegex::Sigma)
    }
    pub fn epsilon() -> Rc<GenRegex> {
        Rc::new(GenRegex::Epsilon)
    }
    pub fn create_gre_char_lit(lit: char) -> Rc<GenRegex> {
        let lit = CharExpression::Literal(lit);
        Rc::new(GenRegex::CharExpression(lit))
    }
    pub fn create_gre_char_var(var_name: &str) -> Rc<GenRegex> {
        let char_var = CharExpression::CharVar(CharVar {
            name: var_name.to_string(),
        });
        Rc::new(GenRegex::CharExpression(char_var))
    }
    pub fn create_gre_str_var(var_name: &str) -> Rc<GenRegex> {
        let str_var = StringVar {
            name: var_name.to_string(),
        };
        Rc::new(GenRegex::StringVar(str_var))
    }
    pub fn concat(gre1: &Rc<GenRegex>, gre2: &Rc<GenRegex>) -> Rc<GenRegex> {
        Rc::new(GenRegex::Concatenation(gre1.clone(), gre2.clone()))
    }
    pub fn intersect(gre1: &Rc<GenRegex>, gre2: &Rc<GenRegex>) -> Rc<GenRegex> {
        Rc::new(GenRegex::Intersect(gre1.clone(), gre2.clone()))
    }
    pub fn union(gre1: &Rc<GenRegex>, gre2: &Rc<GenRegex>) -> Rc<GenRegex> {
        Rc::new(GenRegex::Union(gre1.clone(), gre2.clone()))
    }
    pub fn complement(gre: &Rc<GenRegex>) -> Rc<GenRegex> {
        Rc::new(GenRegex::Complement(gre.clone()))
    }
    pub fn union_many(args: &[Rc<GenRegex>]) -> Rc<GenRegex> {
        if args.is_empty() {
            GenRegex::empty_set()
        } else if args.len() == 1 {
            args[0].clone()
        } else {
            GenRegex::union(&args[0].clone(), &GenRegex::union_many(&args[1..]))
        }
    }

    pub fn intersect_many(args: &[Rc<GenRegex>]) -> Rc<GenRegex> {
        if args.is_empty() {
            GenRegex::empty_set()
        } else if args.len() == 1 {
            args[0].clone()
        } else {
            GenRegex::intersect(&args[0].clone(), &GenRegex::intersect_many(&args[1..]))
        }
    }

    pub fn concat_many(args: &[Rc<GenRegex>]) -> Rc<GenRegex> {
        if args.is_empty() {
            GenRegex::empty_set()
        } else if args.len() == 1 {
            args[0].clone()
        } else {
            GenRegex::concat(&args[0].clone(), &GenRegex::concat_many(&args[1..]))
        }
    }

    pub fn star(gre: &Rc<GenRegex>) -> Rc<GenRegex> {
        Rc::new(GenRegex::Kleene(gre.clone()))
    }
    pub fn empty_set() -> Rc<GenRegex> {
        Rc::new(GenRegex::EmptySet)
    }
    pub fn str_to_re(str: &str) -> Rc<GenRegex> {
        //Needs something better than chars() for more support
        let mut char_list = str.chars().rev();
        let Some(c) = char_list.next() else {
            return GenRegex::epsilon();
        };
        let mut retval = GenRegex::create_gre_char_lit(c);
        for c in char_list {
            retval = GenRegex::concat(&GenRegex::create_gre_char_lit(c), &retval);
        }
        retval
    }
    pub fn re_range(start: char, end: char) -> Rc<GenRegex> {
        // WITH RANGE OPTIMIZATION:
        Rc::new(GenRegex::Range(start, end))
        // OLD IMPL:
        // let mut retval = GenRegex::create_gre_char_lit(end);
        // for c in (start..=end).rev().skip(1) {
        //     retval = GenRegex::union(&GenRegex::create_gre_char_lit(c), &retval)
        // }
        // retval
    }
    pub fn caret(n: u64, gre: &Rc<GenRegex>) -> Rc<GenRegex> {
        if n == 0 {
            return GenRegex::epsilon();
        }
        let mut retval = gre.clone();
        for _ in 1..n {
            retval = GenRegex::concat(&gre.clone(), &retval);
        }
        retval
    }
    pub fn re_loop(n1: u64, n2: u64, gre: &Rc<GenRegex>) -> Rc<GenRegex> {
        if n1 > n2 {
            return GenRegex::empty_set();
        }
        let mut retval = GenRegex::caret(n2, gre);
        for i in (n1..n2).rev() {
            retval = GenRegex::union(&GenRegex::caret(i, gre), &retval);
        }
        retval
    }
}

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
        /*if index < self.head.len() {
            &Some(self.head[index])
        } else {
            // Return a reference to None when the index is out of bounds.
            &None
        }*/
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
    Predicates and characters
*/

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum Predicate {
    And(Rc<Predicate>, Rc<Predicate>),
    Or(Rc<Predicate>, Rc<Predicate>),
    Not(Rc<Predicate>),
    True,
    False,
    Equals(Rc<MaybeCharExpression>, Rc<MaybeCharExpression>),
    EqualLength(Rc<StringVar>, i32),
    LessThan(Rc<MaybeCharExpression>, char), //Includes Equal to
    GreaterThan(Rc<MaybeCharExpression>, char), //Includes Equal to
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum MaybeCharExpression {
    CharExpression(CharExpression),
    StringIndex(StringIndex),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum CharExpression {
    CharVar(CharVar), // change to CharVar
    Literal(char),    // should be a char
}

/*#[derive(Debug, PartialEq, Eq, Hash, Clone)] // Deriving PartialEq, Eq, and Hash
pub enum StringObject{
    StringSlice(Rc<StringVar>, i32),
    StringVar(Rc<StringVar>)
}*/

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct StringVar {
    pub name: String,
}

// TODO
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct CharVar {
    pub name: String,
}

// pub struct StringVar {
//     name: String,
// }
// impl StringVar {
//     pub fn new(s: String) -> Self {
//         Self(name)
//     }
//     pub fn from_integer(i: usize) -> Self {
//         Self(format!("w{}", i))
//     }
//     pub fn get_name(&self) -> &str {
//         &self.name
//     }
// }

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct StringIndex {
    pub var: StringVar,
    pub index: i32,
}

/*impl PartialOrd for GenRegex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GenRegex {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (GenRegex::EmptySet, GenRegex::EmptySet) => Ordering::Equal,
            (GenRegex::EmptySet, _) => Ordering::Less,
            (_, GenRegex::EmptySet) => Ordering::Greater,
            (GenRegex::CharExpression(a), GenRegex::CharExpression(b)) => a.cmp(b),
            (GenRegex::CharExpression(_), _) => Ordering::Less,
            (_, GenRegex::CharExpression(_)) => Ordering::Greater,
            (GenRegex::StringVar(a), GenRegex::StringVar(b)) => a.cmp(b),
            (GenRegex::StringVar(_), _) => Ordering::Less,
            (_, GenRegex::StringVar(_)) => Ordering::Greater,
            (GenRegex::StringSlice(a, i), GenRegex::StringSlice(b, j)) => {
                let cmp = a.cmp(b);
                if cmp == Ordering::Equal {
                    i.cmp(j)
                } else {
                    cmp
                }
            }
            (GenRegex::StringSlice(_, _), _) => Ordering::Less,
            (_, GenRegex::StringSlice(_, _)) => Ordering::Greater,
            (GenRegex::Union(a, b), GenRegex::Union(c, d)) => {
                let cmp = a.cmp(c);
                if cmp == Ordering::Equal {
                    b.cmp(d)
                } else {
                    cmp
                }
            }
            (GenRegex::Union(_, _), _) => Ordering::Less,
            (_, GenRegex::Union(_, _)) => Ordering::Greater,
            (GenRegex::Intersect(a, b), GenRegex::Intersect(c, d)) => {
                let cmp = a.cmp(c);
                if cmp == Ordering::Equal {
                    b.cmp(d)
                } else {
                    cmp
                }
            }
            (GenRegex::Intersect(_, _), _) => Ordering::Less,
            (_, GenRegex::Intersect(_, _)) => Ordering::Greater,
            (GenRegex::Concatenation(a, b), GenRegex::Concatenation(c, d)) => {
                let cmp = a.cmp(c);
                if cmp == Ordering::Equal {
                    b.cmp(d)
                } else {
                    cmp
                }
            }
            (GenRegex::Concatenation(_, _), _) => Ordering::Less,
            (_, GenRegex::Concatenation(_, _)) => Ordering::Greater,
            (GenRegex::Kleene(a), GenRegex::Kleene(b)) => a.cmp(b),
            (GenRegex::Kleene(_), _) => Ordering::Less,
            (_, GenRegex::Kleene(_)) => Ordering::Greater,
            (GenRegex::Complement(a), GenRegex::Complement(b)) => a.cmp(b),
            (GenRegex::Complement(_), _) => Ordering::Less,
            (_, GenRegex::Complement(_)) => Ordering::Greater,
            (GenRegex::IfThenElse(a, b, c), GenRegex::IfThenElse(d, e, f)) => {
                let cmp = a.cmp(d);
                if cmp == Ordering::Equal {
                    let cmp2 = b.cmp(e);
                    if cmp2 == Ordering::Equal {
                        c.cmp(f)
                    } else {
                        cmp2
                    }
                } else {
                    cmp
                }
            }
            (GenRegex::IfThenElse(_, _, _), _) => Ordering::Less,
            (_, GenRegex::IfThenElse(_, _, _)) => Ordering::Greater,
            (GenRegex::StringIndex(a), GenRegex::StringIndex(b)) => a.cmp(b),
        }
    }
}*/
