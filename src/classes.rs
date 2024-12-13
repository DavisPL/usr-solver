//!
//! Type definitions -
//! Main GenRegex class and subclasses
//!

use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GenRegex {
    EmptySet,
    CharExpression(Rc<CharExpression>),
    StringVar(Rc<StringVar>),
    StringSlice(Rc<StringVar>, i32),
    Union(Rc<GenRegex>, Rc<GenRegex>),
    Intersect(Rc<GenRegex>, Rc<GenRegex>),
    Concatenation(Rc<GenRegex>, Rc<GenRegex>),
    Kleene(Rc<GenRegex>),
    Complement(Rc<GenRegex>),
    IfThenElse(Rc<Predicate>, Rc<GenRegex>, Rc<GenRegex>),
    StringIndex(Rc<StringIndex>),
}

pub enum MergeResult {
    Subs(Subs),
    Bottom
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Subs {
    EmptySub,
    Sub(Rc<Pair>)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Pair {
    Combined(Rc<Pair>, Rc<Pair>),
    StringTo(Rc<StringVar>, Rc<SubExpr>),
    CharTo(Rc<CharExpression>, Rc<CharExpression>)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum SubExpr {
    Combined(Rc<CharExpression>, Rc<SubExpr>),
    EmptyString,
    StringVar(Rc<StringVar>),
}

/*
    TODO fix: use the following for SubExpr and Subs
*/

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubExpr2 {
    head: Vec<CharExpression>,
    tail_is_string_var: bool
}

// impl Into<GenRegex> for SubExpr2 {

// }

impl SubExpr2 {
    fn to_gen_regex(&self, tail_var: &StringVar) -> GenRegex {
        unimplemented!()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AnySub {
    string_to: HashMap<StringVar, Vec<SubExpr2>>
    char_to: HashMap<CharVar, Vec<CharExpression>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SimpleSub {
    string_to: HashMap<StringVar, SubExpr2>,
    char_to: HashMap<CharVar, CharExpression>,
}

impl SimpleSub {
    fn substitute_in_regex(&self, g: GenRegex) -> GenRegex {
        unimplemented!()
    }
}

use std::collections::HashMap;
use std::ops::Index;
use std::ops::IndexMut;

// l[3] -- 3rd elem of list
// class MyList
// m: MyList
// m[5] -- define what it means to get the 5th element of MyList

// https://doc.rust-lang.org/std/ops/trait.Index.html
impl Index<&StringVar> for SimpleSub {
    type Output = SubExpr2;

    fn index(&self, index: &StringVar) -> &Self::Output {
        unimplemented!()
    }
}

impl IndexMut<&StringVar> for SimpleSub {
    fn index_mut(&mut self, index: &StringVar) -> &mut Self::Output {
        unimplemented!()
    }
}

impl Index<usize> for SubExpr2 {
    type Output = Option<CharExpression>;

    fn index(&self, index: usize) -> &Self::Output {
        unimplemented!()
    }
}

// f: SimpleSub
// w: String var (w1, w2, w3)
// f[w] <- get the subexpr

// f1 + f2 <- merge two simple subs
// f1 - f2 <- sub subtractions
// impl Add<SimpleSub> for SimpleSub {
//     //
// }

// merge_subs()

// Option<SimpleSub> - simple sub or \bottom

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum Predicate {
    And(Vec<Rc<Predicate>>),
    Or(Vec<Rc<Predicate>>),
    Not(Rc<Predicate>),
    True,
    False,
    Equals(
        Rc<MaybeCharExpression>,
        Rc<MaybeCharExpression>
    ),
    EqualLength(Rc<StringVar>, i32),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum MaybeCharExpression {
    CharExpression(Rc<CharExpression>),
    StringIndex(Rc<StringIndex>)
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum CharExpression {
    CharVar(String), // change to CharVar
    Literal(String),
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
    pub var: Rc<StringVar>,
    pub index: i32,
}

impl PartialOrd for GenRegex {
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
}
