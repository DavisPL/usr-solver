mod classes;

use classes::{GenRegex, Predicate, CharExpression, StringObject, StringVar};
use std::rc::Rc;

fn print_predicate(pred: &Predicate) -> String{
    match pred {
        Predicate::And(pred1, pred2) =>{
            format!("({}) AND ({})", print_predicate(pred1), print_predicate(pred2))
        }
        Predicate::Or(pred1, pred2) =>{
            format!("({}) OR ({})", print_predicate(pred1), print_predicate(pred2))
        }
        Predicate::Not(pred1) =>{
            format!("NOT({})", print_predicate(pred1)) }
        Predicate::True =>{
            "TRUE".to_string()
        }
        Predicate::False =>{
            "FALSE".to_string()
        }
        Predicate::Equals(var, var2) =>{
            format!("({}) == ({})", print_char_expression(var), print_char_expression(var2))
        }
        Predicate::EqualLength(var, inte) =>{
            format!("|{}| == ({})", print_string_object(var), inte)
        }
    }

}
fn print_char_expression(charExpr: &CharExpression) -> String{
    match charExpr{
        CharExpression::CharVar(name) =>{
            format!("char({})", name)
        }
        CharExpression::Literal(name) =>{
            format!("{}", name)
        }
        CharExpression::StringIndex(name, index) =>{
            format!("{}[{}]", print_string_var(name), index)
        }
    }
}
pub fn print_string_object(str_obj: &StringObject) -> String {
    match str_obj {
        StringObject::StringSlice(var, index) => {
            format!("{}[{}:]", print_string_var(var), index)
        }
        StringObject::StringVar(name) => format!("STR({})", name),
    }
}

// Function to print StringVar
pub fn print_string_var(string_var: &StringVar) -> String {
    format!("STR({})", string_var.name)
}

pub fn print_gre(genregex: &GenRegex) -> String{
    match genregex{
        GenRegex::EmptySet =>{
            "EMPTY".to_string()
        }
        GenRegex::CharExpression(charExpr) =>{
            format!("{}", print_char_expression(charExpr))
        }
        GenRegex::StringObject(stringObj) =>{
            format!("{}", print_string_object(stringObj))
        }
        GenRegex::Union(gre1, gre2) =>{
            format!("({}) OR ({})", print_gre(gre1), print_gre(gre2))
        }
        GenRegex::Intersect(gre1, gre2) =>{
            format!("({}) AND ({})", print_gre(gre1), print_gre(gre2))
        }
        GenRegex::Concatenation(gre1, gre2) =>{
            format!("({}) \\cdot ({})", print_gre(gre1), print_gre(gre2))
        }
        GenRegex::Kleene(gre1) =>{
            format!("({})*", print_gre(gre1))
        }
        GenRegex::Complement(gre1) =>{
            format!("({})^c", print_gre(gre1))
        }
        GenRegex::IfThenElse(pred, gre1, gre2) =>{
            format!("IF({}, {}, {})", print_predicate(pred), print_gre(gre1), print_gre(gre2))
        }
    }

}
fn main() {
    let string_var = StringVar {
        name: String::from("example_var"),
    };

    let char_expr = CharExpression::StringIndex(string_var, 0);

    let predicate = Predicate::Equals(
        CharExpression::Literal(String::from("")),
        char_expr,
    );

    let complex_predicate = Rc::new(Predicate::And(Rc::new(predicate), Rc::new(Predicate::True)));
    let gre = GenRegex::IfThenElse(complex_predicate.clone(), Rc::new(GenRegex::CharExpression(CharExpression::CharVar(String::from("c")))), Rc::new(GenRegex::EmptySet));
    println!("{}", print_predicate(&complex_predicate));
    println!("{}", print_gre(&gre));
    println!("Hello World!");

    // Now you can use `complex_predicate` as needed
}

