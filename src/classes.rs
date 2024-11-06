//! Object definitions
#![allow(dead_code)]

use either::Either;
use std::rc::Rc;

// TODO: variants `Kleene`, `Complement`, and `StringIndex` are never constructed
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

#[derive(PartialEq, Eq, Hash, Clone)]
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

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum CharExpression {
    CharVar(String),
    Literal(String),
}

/*#[derive(PartialEq, Eq, Hash, Clone)] // Deriving PartialEq, Eq, and Hash
pub enum StringObject{
    StringSlice(Rc<StringVar>, i32),
    StringVar(Rc<StringVar>)
}*/

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct StringVar {
    pub name: String,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct StringIndex {
    pub var: Rc<StringVar>,
    pub index: i32,
}
