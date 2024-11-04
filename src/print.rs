//mod classes;

use crate::classes::{CharExpression, GenRegex, Predicate, StringIndex, StringVar};
use either::Either;
use std::rc::Rc;

pub fn print_predicate(pred: &Rc<Predicate>) -> String {
    match pred.as_ref() {
        Predicate::And(kids) => {
            let parts: Vec<String> = kids.iter().map(|child| print_predicate(child)).collect();
            format!("({})", parts.join(" AND "))
        }
        Predicate::Or(kids) => {
            let parts: Vec<String> = kids.iter().map(|child| print_predicate(child)).collect();
            format!("({})", parts.join(" OR "))
        }
        Predicate::Not(pred1) => {
            format!("NOT({})", print_predicate(pred1))
        }
        Predicate::True => "TRUE".to_string(),
        Predicate::False => "FALSE".to_string(),
        Predicate::Equals(var, var2) => {
            format!("{} == {}", print_equals_arg(var), print_equals_arg(var2))
        }
        Predicate::EqualLength(var, inte) => {
            format!("|{}| == {}", print_string_var(var), inte)
        }
    }
}
pub fn print_equals_arg(equals_arg: &Either<Rc<CharExpression>, Rc<StringIndex>>) -> String {
    match equals_arg.as_ref() {
        Either::Left(char_expr) => print_char_expression(char_expr),
        Either::Right(strInd) => {
            let StringIndex { var, index } = strInd.as_ref();
            format!("{}[{}]", print_string_var(var), index)
        }
    }
}
pub fn print_char_expression(char_expr: &Rc<CharExpression>) -> String {
    match char_expr.as_ref() {
        CharExpression::CharVar(name) => {
            format!("char({})", name)
        }
        CharExpression::Literal(name) => {
            if name.is_empty() {
                "\"\"".to_string()
            } else {
                format!("{}", name)
            }
        }
    }
}

pub fn print_string_var(string_var: &Rc<StringVar>) -> String {
    format!("STR({})", string_var.name)
}

pub fn print_gre(genregex: &Rc<GenRegex>) -> String {
    match genregex.as_ref() {
        GenRegex::StringVar(var) => {
            format!("{}", print_string_var(var))
        }
        GenRegex::EmptySet => "EMPTY".to_string(),
        GenRegex::CharExpression(char_expr) => {
            format!("{}", print_char_expression(char_expr))
        }
        GenRegex::Union(gre1, gre2) => {
            format!("({}) OR ({})", print_gre(gre1), print_gre(gre2))
        }
        GenRegex::Intersect(gre1, gre2) => {
            format!("({}) AND ({})", print_gre(gre1), print_gre(gre2))
        }
        GenRegex::Concatenation(gre1, gre2) => {
            format!("({}) \\cdot ({})", print_gre(gre1), print_gre(gre2))
        }
        GenRegex::Kleene(gre1) => {
            format!("({})*", print_gre(gre1))
        }
        GenRegex::Complement(gre1) => {
            format!("({})^c", print_gre(gre1))
        }
        GenRegex::IfThenElse(pred, gre1, gre2) => {
            format!(
                "IF({}, {}, {})",
                print_predicate(pred),
                print_gre(gre1),
                print_gre(gre2)
            )
        }
        GenRegex::StringIndex(string_index) => {
            format!(
                "{}[{}]",
                print_string_var(&string_index.var),
                string_index.index
            )
        }
        GenRegex::StringSlice(var, index) => {
            format!("{}[{}:]", print_string_var(var), index)
        }
    }
}
