//!
//! Display implementations for pretty printing
//!

use crate::classes::{CharExpression, GenRegex, Predicate, StringIndex, StringVar, MaybeCharExpression, CharVar};
use std::fmt::{self, Display};


impl Display for CharVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "char({})", self.name)
    }
}

impl Display for CharExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CharExpression::CharVar(name) => {
                write!(f, "char({})", name)
            }
            CharExpression::Literal(name) => {
                if name.is_empty() {
                    write!(f, "\"\"")
                } else {
                    write!(f, "{}", name)
                }
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
        write!(f, "STR({})", self.name)
    }
}

impl Display for StringIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}]", self.var, self.index)
    }
}

impl Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Predicate::And(kids) => {
                let parts: Vec<String> = kids.iter().map(|p| format!("{}", p)).collect();
                write!(f, "({})", parts.join(" AND "))
            }
            Predicate::Or(kids) => {
                let parts: Vec<String> = kids.iter().map(|p| format!("{}", p)).collect();
                write!(f, "({})", parts.join(" OR "))
            }
            Predicate::Not(pred1) => {
                write!(f, "NOT({})", pred1)
            }
            Predicate::True => write!(f, "TRUE"),
            Predicate::False => write!(f, "FALSE"),
            Predicate::Equals(var1, var2) => {
                write!(f, "{} == {}", var1, var2)
            }
            Predicate::EqualLength(var, inte) => {
                write!(f, "|{}| == {}", var, inte)
            }
        }
    }
}

impl Display for GenRegex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenRegex::StringVar(var) => {
                // Use Display on var
                write!(f, "{}", var)
            }
            GenRegex::EmptySet => {
                write!(f, "EMPTY")
            }
            GenRegex::CharExpression(char_expr) => {
                // Use Display on char_expr
                write!(f, "{}", char_expr)
            }
            GenRegex::Union(gre1, gre2) => {
                write!(f, "({}) OR ({})", gre1, gre2)
            }
            GenRegex::Intersect(gre1, gre2) => {
                write!(f, "({}) AND ({})", gre1, gre2)
            }
            GenRegex::Concatenation(gre1, gre2) => {
                write!(f, "({}) \\cdot ({})", gre1, gre2)
            }
            GenRegex::Kleene(gre1) => {
                write!(f, "({})*", gre1)
            }
            GenRegex::Complement(gre1) => {
                write!(f, "({})^c", gre1)
            }
            GenRegex::IfThenElse(pred, gre1, gre2) => {
                write!(f, "IF({}, {}, {})", pred, gre1, gre2)
            }
            GenRegex::StringIndex(string_index) => {
                // Use Display on string_index
                write!(f, "{}", string_index)
            }
            GenRegex::StringSlice(var, index) => {
                write!(f, "{}[{}:]", var, index)
            }
        }
    }
}
