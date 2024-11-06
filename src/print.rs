// Better to fix and remove, allowing for now
#![allow(non_snake_case)]
/*
This file should be done by implementing
the Display trait.

https://doc.rust-lang.org/std/fmt/trait.Display.html

Example:

use fmt::{self, Display};

impl Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // paste in your print logic below here
        // instead of format!( you would then use write! to write the result to the formatter.
    }
}

Using Display means you would be able to print both to a string, or to a file, by using the
`{}` syntax (instead of `{:?}` for Debug) and get the printing style you want by default.
*/
// Remove after converting to Display

use crate::classes::{CharExpression, GenRegex, Predicate, StringIndex, StringVar};
use std::fmt::{self, Display};
use std::rc::Rc;

/*
    Display implementations
*/

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

/*
    Helper print functions

    NB:
    This is to preserve backward compatibility with the previous implementation;
    these directly call the Display implementations above.

    These can probably be removed if you want to just directly print using, e.g.

        format!("{}", genregex)

    if you want to get the string representation of the object, directly equivalent to the below, or

        print!("{}", genregex)

    if you want to print it to stdout.
*/

pub fn print_gre(genregex: &Rc<GenRegex>) -> String {
    // Use Display on GenRegex
    format!("{}", genregex)
}

// Unused

// pub fn print_predicate(pred: &Rc<Predicate>) -> String {
//     // Use Display on Predicate
//     format!("{}", pred)
// }

// pub fn print_char_expression(char_expr: &Rc<CharExpression>) -> String {
//     // Use Display on CharExpression
//     format!("{}", char_expr)
// }

// pub fn print_string_var(string_var: &Rc<StringVar>) -> String {
//     // Use Display on StringVar
//     format!("{}", string_var)
// }
