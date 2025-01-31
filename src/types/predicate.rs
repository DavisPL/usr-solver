//!
//! Predicates
//!

use super::expr::{MaybeCharExpression, StringVar};

use std::rc::Rc;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum Predicate {
    And(Rc<Predicate>, Rc<Predicate>),
    Or(Rc<Predicate>, Rc<Predicate>),
    Not(Rc<Predicate>),
    True,
    False,
    Equals(Rc<MaybeCharExpression>, Rc<MaybeCharExpression>),
    EqualLength(Rc<StringVar>, i32),
    // TODO: rename to LessThanEq, GreaterThanEq
    LessThan(Rc<MaybeCharExpression>, char), //Includes Equal to
    GreaterThan(Rc<MaybeCharExpression>, char), //Includes Equal to
}

/*
    Pretty printing
*/

use std::fmt::{self, Display};

impl Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Predicate::And(left, right) => {
                //let parts: Vec<String> = kids.iter().map(|p| format!("{}", p)).collect();
                write!(f, "({} ^ {})", left, right)
            }
            Predicate::Or(left, right) => {
                //let parts: Vec<String> = kids.iter().map(|p| format!("{}", p)).collect();
                write!(f, "({} v {})", left, right)
                //write!(f, "({})", parts.join(" OR "))
            }
            Predicate::Not(pred1) => {
                write!(f, "!({})", pred1)
            }
            Predicate::True => write!(f, "TRUE"),
            Predicate::False => write!(f, "FALSE"),
            Predicate::Equals(var1, var2) => {
                write!(f, "{} == {}", var1, var2)
            }
            Predicate::EqualLength(var, inte) => {
                write!(f, "|{}| == {}", var, inte)
            }
            Predicate::LessThan(var, val) => {
                write!(f, "{} <= {}", var, val)
            }
            Predicate::GreaterThan(var, val) => {
                write!(f, "{} >= {}", var, val)
            }
        }
    }
}
