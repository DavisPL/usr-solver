//!
//! Type definitions -
//! Main GenRegex class and subclasses
//!

use either::Either;
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(PartialEq, Eq, Hash, Clone)]
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

impl PartialOrd for GenRegex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum Predicate {
    And(Vec<Rc<Predicate>>),
    Or(Vec<Rc<Predicate>>),
    Not(Rc<Predicate>),
    True,
    False,
    Equals(
        Either<Rc<CharExpression>, Rc<StringIndex>>,
        Either<Rc<CharExpression>, Rc<StringIndex>>,
    ),
    EqualLength(Rc<StringVar>, i32),
}

#[derive(PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum CharExpression {
    CharVar(String),
    Literal(String),
}

/*#[derive(PartialEq, Eq, Hash, Clone)] // Deriving PartialEq, Eq, and Hash
pub enum StringObject{
    StringSlice(Rc<StringVar>, i32),
    StringVar(Rc<StringVar>)
}*/

#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct StringVar {
    pub name: String,
}

#[derive(PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct StringIndex {
    pub var: Rc<StringVar>,
    pub index: i32,
}
