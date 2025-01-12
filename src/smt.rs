//!
//! Parsing for SMTLib files
//!

use super::classes::{CharExpression, GenRegex, StringVar};

use lexpr::{self, value, Value};

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

/*
    Error type
*/

#[derive(Debug)]
pub enum SmtParseError {
    FileError(String),               // File not found
    SexprError(lexpr::parse::Error), // Error parsing S-expression
    MissingAssertion(),              // Missing (assert) statement in SMTLib file
    MissingCheckSat(),               // Missing (check-sat) statement in SMTLib file
    Unsupported(String),             // Unsupported SMTLib feature
    Unrecognized(String),            // Unrecognized SMTLib feature
    Unimplemented(String),           // Unimplemented SMTLib feature
}

impl From<lexpr::parse::Error> for SmtParseError {
    fn from(e: lexpr::parse::Error) -> Self {
        SmtParseError::SexprError(e)
    }
}

impl From<std::io::Error> for SmtParseError {
    fn from(e: std::io::Error) -> Self {
        SmtParseError::FileError(e.to_string())
    }
}

impl fmt::Display for SmtParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SmtParseError::FileError(s) => write!(f, "File error: {}", s),
            SmtParseError::SexprError(e) => write!(f, "S-expression error: {}", e),
            SmtParseError::MissingAssertion() => write!(f, "Expected (assert) statement"),
            SmtParseError::MissingCheckSat() => write!(f, "Expected (check-sat) statement"),
            SmtParseError::Unsupported(s) => write!(f, "Unsupported SMTLib feature: {}", s),
            SmtParseError::Unrecognized(s) => write!(f, "Unrecognized SMTLib feature: {}", s),
            SmtParseError::Unimplemented(s) => write!(f, "Unimplemented SMTLib feature: {}", s),
        }
    }
}

impl Error for SmtParseError {}

/*
    S expression parsing functions

    These are private so that the implementation can be changed later
*/

fn parse_smtlib_string(smt_string: &str) -> Result<Value, SmtParseError> {
    let v = lexpr::from_str(smt_string)?;
    Ok(v)
}

fn parse_smtlib_file(file_path: &str) -> Result<Value, SmtParseError> {
    // Read in the file
    let smt_string = std::fs::read_to_string(file_path)?;

    // Add an opening and closoing paren
    let smt_string = format!("(\n{}\n)", smt_string);

    // Parse S-expression
    let v = lexpr::from_str(&smt_string)?;

    // Return
    Ok(v)
}

/*
    Main parsing interface
*/

pub struct SmtParser {
    found_assert: bool,
    found_check_sat: bool,
    var_names: HashSet<String>,
    regex_result: Option<GenRegex>,
}

impl SmtParser {
    pub fn new() -> Self {
        Self {
            found_assert: false,
            found_check_sat: false,
            var_names: HashSet::new(),
            regex_result: None,
        }
    }

    pub fn parse_s_expr(&mut self, v: &Value) -> Result<GenRegex, SmtParseError> {
        // println!("called parse_s_expr with value: {:?}", v);

        // Parse list of items at the top level recursively
        if let Value::Cons(c) = v {
            let (head, tail) = c.as_pair();
            // Process head
            self.parse_head(head)?;
            // Recurse on tail
            self.parse_s_expr(tail)
        } else if let Value::Null = v {
            if !self.found_assert {
                return Err(SmtParseError::MissingAssertion());
            }
            if !self.found_check_sat {
                return Err(SmtParseError::MissingCheckSat());
            }
            let result = self.regex_result.take();
            Ok(result.expect("Regex result should have been set by parser earlier!"))
        } else {
            Err(SmtParseError::Unrecognized(format!(
                "Found unexpected S-expression: {:?}",
                v
            )))
        }
    }

    fn parse_head(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // 3 cases here: (declare-const), (assert), (check-sat)
        if let Value::Cons(c) = v {
            let (head, tail) = c.as_pair();

            if let Value::Symbol(s) = head {
                match s.as_ref() {
                    "declare-const" => self.parse_declare_const(tail),
                    "assert" => self.parse_assert(tail),
                    "check-sat" => self.parse_check_sat(tail),
                    _ => Err(SmtParseError::Unsupported(format!(
                        "Unsupported SMTLib command: {}",
                        s
                    ))),
                }
            } else {
                Err(SmtParseError::Unrecognized(format!(
                    "Unrecognized S-expression: {:?}",
                    head
                )))
            }
        } else {
            Err(SmtParseError::Unrecognized(format!(
                "Unrecognized S-expression: {:?}",
                v
            )))
        }
    }

    fn parse_declare_const(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // Add variable name to self.var_names
        if let Value::Cons(c1) = v {
            let (head1, tail1) = c1.as_pair();
            if let Value::Symbol(var_name) = head1 {
                if let Value::Cons(c2) = tail1 {
                    let (head2, tail2) = c2.as_pair();
                    if let Value::Symbol(var_type) = head2 {
                        let Value::Null = tail2 else { todo!() };
                        match var_type.as_ref() {
                            "String" => {
                                self.var_names.insert(var_name.to_string());
                                Ok(())
                            }
                            _ => Err(SmtParseError::Unrecognized(format!(
                                "Unrecognized S-expression: {:?}",
                                v
                            ))),
                        }
                    } else {
                        Err(SmtParseError::Unrecognized(format!(
                            "Unrecognized S-expression: {:?}",
                            v
                        )))
                    }
                } else {
                    Err(SmtParseError::Unrecognized(format!(
                        "Unrecognized S-expression: {:?}",
                        v
                    )))
                }
            } else {
                Err(SmtParseError::Unrecognized(format!(
                    "Unrecognized S-expression: {:?}",
                    v
                )))
            }
        } else {
            Err(SmtParseError::Unrecognized(format!(
                "Unrecognized S-expression: {:?}",
                v
            )))
        }
        // TODO
    }

    fn parse_assert(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // Set flag
        self.found_assert = true;
        // Parse the regex
        if let Value::Cons(c) = v {
            let (head, tail) = c.as_pair();
            if let Value::Symbol(s) = head {
                match s.as_ref() {
                    "str.in_re" => self.parse_str_in_re(tail),
                    _ => Err(SmtParseError::Unrecognized(format!(
                        "Unrecognized S-expression: {:?}",
                        v
                    ))),
                }
            } else {
                Err(SmtParseError::Unrecognized(format!(
                    "Unrecognized S-expression: {:?}",
                    v
                )))
            }
        } else {
            Err(SmtParseError::Unrecognized(format!(
                "Unrecognized S-expression: {:?}",
                v
            )))
        }
    }
    fn parse_regex(&mut self, v: &Value) -> Result<GenRegex, SmtParseError> {
        todo!();
    }
    fn parse_re_union(&mut self, v: &Value) -> Result<GenRegex, SmtParseError> {
        if let Value::Cons(c) = v {
            let (head, tail) = c.as_pair();
            let head_parsed = self.parse_regex(head)?;
            //let head_unwrapped = head_parsed.unwrap();
            let tail_parsed = self.parse_regex(tail)?;
            //let tail_unwrapped = tail_parsed.unwrap();
            let union_term = GenRegex::Union(Rc::new(head_parsed), Rc::new(tail_parsed));
            return Ok(union_term);
        }
        println!("unioning {}", v);
        todo!();
        
    }

    fn parse_str_in_re(&mut self, v: &Value) -> Result<(), SmtParseError> {
        //(str.in_re x R) update regex_result <- Some(x \cap R)
        if let Value::Cons(c) = v {
            let (head, tail) = c.as_pair();
            if let Value::Symbol(s) = head {
                if self.var_names.contains(s.as_ref()) {
                    //Create GenRegex::StringVar x
                    todo!()
                } else {
                    return Err(SmtParseError::Unrecognized(format!(
                        "Unrecognized S-expression: {:?}",
                        v
                    )));
                }
            }
        }
        todo!()
    }

    fn parse_check_sat(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // Set flag
        self.found_check_sat = true;
        Ok(())
    }

    pub fn parse_empty(&self) -> Result<GenRegex, SmtParseError> {
        eprintln!("TODO: Implement parse_empty");
        Err(SmtParseError::Unimplemented(
            "Empty S-expression".to_string(),
        ))
    }

    pub fn parse_cons(&self, _c: &lexpr::Cons) -> Result<GenRegex, SmtParseError> {
        eprintln!("TODO: Implement parse_empty");
        Err(SmtParseError::Unimplemented(
            "Cons S-expression".to_string(),
        ))

        // let mut iter = c.iter();
        // let head = iter.next().unwrap();
        // let tail = iter.next().unwrap();

        // match head {
        //     Value::Symbol(s) => match s.as_str() {
        //         "declare-const" => {
        //             self.parse_declare_const(tail)
        //         }
        //         "assert" => {
        //             self.parse_assert(tail)
        //         }
        //         "check-sat" => {
        //             self.parse_check_sat(tail)
        //         }
        //         _ => Err(Error::Unsupported(format!(
        //             "Unsupported SMTLib command: {}",
        //             s
        //         ))),
        //     },
        //     _ => Err(Error::Unrecognized("Unrecognized SMTLib command".to_string())),
        // }
    }

    pub fn parse_symbol(&self, _s: &str) -> Result<GenRegex, SmtParseError> {
        eprintln!("TODO: Implement parse_symbol");
        Err(SmtParseError::Unimplemented(
            "Symbol S-expression".to_string(),
        ))
    }
}

/*
    Entrypoint
*/

// TODO
// pub fn genregex_from_smtlib_string(smt_string: &str) -> Result<GenRegex, SmtParseError> {
//     let v = parse_smtlib_string(smt_string)?;
//     parse_genregex(&v)
// }

// pub fn genregex_from_smtlib_file(file_path: &str) -> Result<GenRegex, SmtParseError> {
//     let v = parse_smtlib_file(file_path)?;
//     parse_genregex(&v)
// }

/*
    Unit tests
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s_expr_test() {
        // Basic unit test for parsing SMTLib files
        // Note that we have to add the beginning '(' and ending ')' to the string
        // so that it makes a single S-expression.

        let smt_string = r#"
(
(set-logic QF_S)
;---
; .NET regular expressions restricted to 7-bit characters
; membership in intersection of
; .*(monday|tuesday|wednesday|thursday|friday|saturday|sunday).*
; .*(january|february|march|april|may|june|july|august|september|october|november|december).*
; [!-~]*
;---
(declare-const x String)
(assert (str.in_re x (re.inter (re.inter (re.++ (re.++ re.all (re.union (re.union (re.union (re.union (re.union (re.union (str.to_re "monday") (str.to_re "tuesday")) (str.to_re "wednesday")) (str.to_re "thursday")) (str.to_re "friday")) (str.to_re "saturday")) (str.to_re "sunday"))) re.all) (re.++ (re.++ re.all (re.union (re.union (re.union (re.union (re.union (re.union (re.union (re.union (re.union (re.union (re.union (str.to_re "january") (str.to_re "february")) (str.to_re "march")) (str.to_re "april")) (str.to_re "may")) (str.to_re "june")) (str.to_re "july")) (str.to_re "august")) (str.to_re "september")) (str.to_re "october")) (str.to_re "november")) (str.to_re "december"))) re.all)) (re.* (re.range "!" "~")))))
(check-sat)
;(get-model)
)
"#;

        println!("{}", smt_string);
        let v = lexpr::from_str(smt_string).unwrap();
        println!("{:?}", v);

        // Uncomment to view output
        // assert!(false);
    }

    #[test]
    fn test_simple_1() {
        // Load the file simple1.smt2
        // Parse as s-expression
        let smt_result = parse_smtlib_file("benchmarks/simple1.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();

        // Expected output
        let expected = GenRegex::Intersect(
            Rc::new(GenRegex::StringVar(StringVar {
                name: "x".to_string(),
            })),
            Rc::new(GenRegex::Concatenation(
                Rc::new(GenRegex::CharExpression(CharExpression::Literal(
                    "a".to_string(),
                ))),
                Rc::new(GenRegex::CharExpression(CharExpression::Literal(
                    "b".to_string(),
                ))),
            )),
        );

        assert_eq!(gen_regex_unwrapped, expected);
    }

    #[ignore]
    #[test]
    fn test_simple_2() {
        // TODO
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn test_simple_3() {
        // TODO
        unimplemented!()
    }

    #[ignore]
    #[test]
    fn test_date() {
        // TODO
        unimplemented!()
    }
}
