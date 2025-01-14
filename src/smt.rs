//!
//! Parsing for SMTLib files
//!

use super::classes::{CharExpression, CharVar, GenRegex, StringVar};

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

impl SmtParseError {
    fn unrecog(expr: &Value) -> SmtParseError {
        SmtParseError::Unrecognized(format!("Found unexpected S-expression: {:?}", expr))
    }
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
    fresh_var_ind: usize,
}

impl SmtParser {
    pub fn new() -> Self {
        Self {
            found_assert: false,
            found_check_sat: false,
            var_names: HashSet::new(),
            regex_result: None,
            fresh_var_ind: 0,
        }
    }

    pub fn parse_s_expr(&mut self, v: &Value) -> Result<GenRegex, SmtParseError> {
        // println!("called parse_s_expr with value: {:?}", v);
        // Parse list of items at the top level recursively
        if let Some((head, tail)) = v.as_pair() {
            // Process head
            self.parse_head(head)?;
            // Recurse on tail
            self.parse_s_expr(tail)
        } else if v.is_null() {
            if !self.found_assert {
                return Err(SmtParseError::MissingAssertion());
            }
            if !self.found_check_sat {
                return Err(SmtParseError::MissingCheckSat());
            }
            let result = self.regex_result.take();
            Ok(result.expect("Regex result should have been set by parser earlier!"))
        } else {
            Err(SmtParseError::unrecog(v))
        }
    }

    fn parse_head(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // 3 cases here: (declare-const), (assert), (check-sat)
        if let Some((head, tail)) = v.as_pair() {
            match head.as_symbol().ok_or(SmtParseError::unrecog(head))? {
                "declare-const" => self.parse_declare_const(tail),
                "assert" => self.parse_assert(tail),
                "check-sat" => self.parse_check_sat(tail),
                _ => Err(SmtParseError::Unsupported(format!(
                    "Unsupported SMTLib command: {}",
                    head
                ))),
            }
        } else {
            Err(SmtParseError::unrecog(v))
        }
    }

    fn parse_declare_const(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // Add variable name to self.var_names
        let (var_name, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (var_type, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        match var_type.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "String" => {
                self.var_names.insert(var_name.to_string());
                Ok(())
            }
            _ => Err(SmtParseError::unrecog(v)),
        }
    }

    fn parse_assert(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // Set flag
        self.found_assert = true;
        // Parse the arg, don't know if assert can take multiple args
        let (assert_arg, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        self.parse_assert_arg(assert_arg)?;
        Ok(())
    }

    fn parse_assert_arg(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // Parse the command. Going to assume the command always is Cons
        let (cmd, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        match cmd.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "str.in_re" => self.parse_str_in_re(tail),
            "and" => todo!(),
            _ => Err(SmtParseError::Unsupported(format!(
                "Unsupported SMTLib command: {}",
                cmd
            ))),
        }
    }

    fn parse_check_sat(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // Set flag
        self.found_check_sat = true;
        if !v.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        Ok(())
    }

    fn parse_regex(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        //Handles (str.to_re), (re.++), (re.inter), (re.union), (re.all), (re.*), (re.range)
        let (re_type, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        match re_type.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "str.to_re" => self.parse_str_to_re(tail),
            "re.++" => self.parse_re_concat(tail),
            "re.*" => todo!(),
            "re.inter" => todo!(),
            "re.all" => todo!(),
            "re.range" => todo!(),
            _ => Err(SmtParseError::unrecog(re_type)),
        }
    }

    fn parse_re_union(&mut self, v: &Value) -> Result<GenRegex, SmtParseError> {
        if let Value::Cons(c) = v {
            let (head, tail) = c.as_pair();
            let head_parsed = self.parse_regex(head)?;
            //let head_unwrapped = head_parsed.unwrap();
            let tail_parsed = self.parse_regex(tail)?;
            //let tail_unwrapped = tail_parsed.unwrap();
            let union_term = GenRegex::Union(head_parsed, tail_parsed);
            return Ok(union_term);
        }
        println!("unioning {}", v);
        todo!();
    }

    fn parse_re_inter(&mut self, v: &Value) -> Result<GenRegex, SmtParseError> {
        if let Value::Cons(c) = v {
            let (head, tail) = c.as_pair();
            let head_parsed = self.parse_regex(head)?;
            //let head_unwrapped = head_parsed.unwrap();
            let tail_parsed = self.parse_regex(tail)?;
            //let tail_unwrapped = tail_parsed.unwrap();
            let union_term = GenRegex::Intersect(head_parsed, tail_parsed);
            return Ok(union_term);
        }
        println!("unioning {}", v);
        todo!();
    }

    fn parse_re_all(&mut self, v: &Value) -> Result<GenRegex, SmtParseError> {
        let new_name = "c".to_owned() + &self.fresh_var_ind.to_string();
        let all_term =
            GenRegex::CharExpression(CharExpression::CharVar(CharVar { name: new_name }));
        self.fresh_var_ind += 1;
        Ok(all_term)
    }

    fn parse_str_in_re(&mut self, v: &Value) -> Result<(), SmtParseError> {
        //Syntax (str.in_re x R)
        let (str_var, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        //Check str_var is in var_names
        let str_var = str_var.as_symbol().ok_or(SmtParseError::unrecog(str_var))?;
        if self.var_names.contains(str_var) {
            //Construct str_var \cap R and update regex_result
            let str_var = GenRegex::create_gre_str_var(str_var);
            let regex = self.parse_regex(regex)?;
            self.regex_result = Some(GenRegex::Intersect(str_var, regex));
            Ok(())
        } else {
            Err(SmtParseError::Unrecognized(format!(
                "String variable not declared/found: {}",
                str_var
            )))
        }
    }

    fn parse_re_concat(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        //Syntax (re.++ R R)
        let (regex1, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex2, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        Ok(GenRegex::concat(
            &self.parse_regex(regex1)?,
            &self.parse_regex(regex2)?,
        ))
    }

    fn parse_str_to_re(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        //(str.to_re "String")
        let (str, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        Ok(GenRegex::str_to_re(
            str.as_str().ok_or(SmtParseError::unrecog(v))?,
        ))
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
