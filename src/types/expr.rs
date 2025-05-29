//!
//! Expression types
//!

/*
    String and character variables
*/

// TBD: Could be useful to add later.
// #[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
// pub enum StringExpression {
//     StringVar(StringVar),
//     Literal(String),
// }

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct StringVar {
    pub name: String,
}

// TODO
#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct CharVar {
    pub name: String,
}

/*
    Character expressions

    TODO: possibly merge MaybeCharExpression and CharExpression,
    or make MaybeCharExpression a String expression.
*/

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum MaybeCharExpression {
    CharExpression(CharExpression),
    StringIndex(StringIndex),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Ord, PartialOrd)]
pub enum CharExpression {
    CharVar(CharVar),
    Literal(char),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct StringIndex {
    pub var: StringVar,
    pub index: i32,
}

/*
    String expressions

    TODO
*/

/*
    Pretty printing
*/

use std::fmt::{self, Display};

impl Display for CharVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for CharExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CharExpression::CharVar(name) => {
                write!(f, "{}", name)
            }
            CharExpression::Literal(name) => {
                write!(f, "'{}'", name)
            }
        }
    }
}

impl Display for MaybeCharExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaybeCharExpression::CharExpression(name) => {
                write!(f, "{}", name)
            }
            MaybeCharExpression::StringIndex(name) => {
                write!(f, "{}", name)
            }
        }
    }
}

impl Display for StringVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Display for StringIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.var, self.index)
    }
}
