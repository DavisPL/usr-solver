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

fn to_char(s: &str) -> char {
    let hex = &s[3..7];
    let val = u32::from_str_radix(hex, 16).unwrap();
    char::try_from(val).unwrap()
}

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
        println!("var_name {}", var_name);
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
        let result = Rc::try_unwrap(self.parse_assert_arg(assert_arg)?);
        if let Ok(r) = result {
            self.regex_result = Some(r);
            Ok(())
        } else {
            Err(SmtParseError::unrecog(assert_arg))
        }
    }

    fn parse_assert_arg(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Parse the command. Going to assume the command always is Cons
        let (cmd, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        match cmd.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "str.in_re" => self.parse_str_in_re(tail),
            "and" => self.parse_and(tail),
            "=" => self.parse_equals(tail),
            _ => Err(SmtParseError::Unsupported(format!(
                "Unsupported SMTLib command: {}",
                cmd
            ))),
        }
    }

    fn parse_equals(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        let (regex1, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex2, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        let parsed1 = self.parse_regex(regex1)?;
        println!("hi {}, {}", parsed1, regex2);
        let parsed2 = self.parse_regex(regex2)?;
        println!("parsed2 {}", parsed2);
        Ok(GenRegex::union(
                &GenRegex::intersect(
                    &parsed1,
                    &GenRegex::complement(&parsed2)
                ),
                &GenRegex::intersect(
                    &GenRegex::complement(&parsed1),
                    &parsed2
                )
        ))
        //todo!();
    }

    fn parse_str_in_re(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        //Syntax (str.in_re x R)
        let (str_var, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        //Check str_var is in var_names
        let str_var = str_var.as_symbol().ok_or(SmtParseError::unrecog(str_var))?;
        if self.var_names.contains(str_var) {
            //Construct str_var \cap R and return
            let str_var = GenRegex::create_gre_str_var(str_var);
            let regex = self.parse_regex(regex)?;
            Ok(GenRegex::intersect(&str_var, &regex))
        } else {
            Err(SmtParseError::Unrecognized(format!(
                "String variable not declared/found: {}",
                str_var
            )))
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
        //Handles base case regex
        if let Some(re_type) = v.as_symbol() {
            return match re_type {
                "re.all" => self.parse_re_all(),
                "re.none" => self.parse_re_none(),
                _ => Err(SmtParseError::unrecog(v)),
            };
        }
        //Handles recursive case
        
        let (re_type, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        //Handles regex func  Ex ((_ re.loop n1 n2) Regex)
        if re_type.is_cons() {
            //Parse the command
            todo!()
        }
        //Handles recursive regex
        match re_type.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "str.to_re" => self.parse_str_to_re(tail),
            "re.++" => self.parse_re_concat(tail),
            "re.union" => self.parse_re_union(tail),
            "re.*" => self.parse_re_star(tail),
            "re.inter" => self.parse_re_inter(tail),
            "re.range" => self.parse_re_range(tail),
            "re.loop" => todo!(),
            _ => Err(SmtParseError::unrecog(re_type)),
        }
    }

    /*
        fn parse_re_union(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
            if let Value::Cons(c) = v {
                let (head, tail) = c.as_pair();
                let head_parsed = self.parse_regex(head)?;
                //let head_unwrapped = head_parsed.unwrap();
                let tail_parsed = self.parse_regex(tail)?;
                //let tail_unwrapped = tail_parsed.unwrap();
                let union_term = GenRegex::union(&head_parsed, &tail_parsed);
                return Ok(union_term);
            }
            Err(SmtParseError::unrecog(v))
        }

        fn parse_re_inter(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
            if let Value::Cons(c) = v {
                let (head, tail) = c.as_pair();
                let head_parsed = self.parse_regex(head)?;
                let tail_parsed = self.parse_regex(tail)?;
                let union_term = GenRegex::intersect(&head_parsed, &tail_parsed);
                return Ok(union_term);
            }
            Err(SmtParseError::unrecog(v))
        }
    */

    fn parse_re_union(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        //Syntax (re.union R R)
        let (regex1, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex2, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        Ok(GenRegex::union(
            &self.parse_regex(regex1)?,
            &self.parse_regex(regex2)?,
        ))
    }

    fn parse_re_inter(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        //Syntax (re.inter R R)
        let (regex1, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex2, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        println!("{}, {}, and tail {}", regex1, regex2, tail);
        if !tail.is_null() {
            println!("failed");
            return Err(SmtParseError::unrecog(v));
        }
        Ok(GenRegex::intersect(
            &self.parse_regex(regex1)?,
            &self.parse_regex(regex2)?,
        ))
    }

    fn parse_re_star(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        //Syntax (re.* R)
        //Returns R*
        let (regex, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        Ok(GenRegex::star(&self.parse_regex(regex)?))
    }

    fn parse_re_all(&self) -> Result<Rc<GenRegex>, SmtParseError> {
        Ok(GenRegex::star(&GenRegex::create_sigma()))
    }
    fn parse_re_none(&self) -> Result<Rc<GenRegex>, SmtParseError> {
        Ok(GenRegex::empty_set())
    }

    fn parse_and(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        //Syntax (and cmd cmd)
        if let Value::Cons(c) = v {
            let (head, tail) = c.as_pair();
            let regex1 = self.parse_assert_arg(head)?;
            if let Value::Cons(c) = tail {
                let (head, tail) = c.as_pair();
                if !tail.is_null() {
                    return Err(SmtParseError::unrecog(v));
                }
                let regex2 = self.parse_assert_arg(head)?;
                return Ok(GenRegex::concat(&regex1, &regex2));
            }
        }
        Err(SmtParseError::unrecog(v))
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

    fn parse_re_range(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax (re.range char1 char2)
        let (char1, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (char2, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        let char1 = char1.as_str().ok_or(SmtParseError::unrecog(v))?;
        let char2 = char2.as_str().ok_or(SmtParseError::unrecog(v))?;
        if char1.len() != 1 || char2.len() != 1 {
            return Err(SmtParseError::unrecog(v));
        }
        if let (Some(char1), Some(char2)) = (char1.chars().next(), char2.chars().next()) {
            return Ok(GenRegex::re_range(&char1, &char2));
        }
        Err(SmtParseError::unrecog(v))
    }

    fn parse_re_loop(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        unimplemented!()
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
    use std::vec;

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

    #[test]
    fn test_simple_2() {
        let smt_result = parse_smtlib_file("benchmarks/simple2.smt2");
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
        let expected_str_var = GenRegex::StringVar(StringVar {
            name: "x".to_string(),
        });
        let expected_intersection_1 = GenRegex::Intersect(
            Rc::new(expected_str_var.clone()),
            Rc::new(GenRegex::CharExpression(CharExpression::Literal(
                "a".to_string(),
            ))),
        );
        let expected_intersection_2 = GenRegex::Intersect(
            Rc::new(expected_str_var),
            Rc::new(GenRegex::CharExpression(CharExpression::Literal(
                "b".to_string(),
            ))),
        );

        let expected = GenRegex::Concatenation(
            Rc::new(expected_intersection_1),
            Rc::new(expected_intersection_2),
        );

        assert_eq!(gen_regex_unwrapped, expected);
        // TODO
        //unimplemented!()
    }

    #[test]
    fn test_simple_3() {
        let smt_result = parse_smtlib_file("benchmarks/simple3.smt2");
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
        let expected_str_var_x = GenRegex::StringVar(StringVar {
            name: "x".to_string(),
        });
        let expected_str_var_y = GenRegex::StringVar(StringVar {
            name: "y".to_string(),
        });
        let expected_intersection_1 = GenRegex::Intersect(
            Rc::new(expected_str_var_x),
            Rc::new(GenRegex::CharExpression(CharExpression::Literal(
                "a".to_string(),
            ))),
        );
        let expected_intersection_2 = GenRegex::Intersect(
            Rc::new(expected_str_var_y),
            Rc::new(GenRegex::CharExpression(CharExpression::Literal(
                "b".to_string(),
            ))),
        );

        let expected = GenRegex::Concatenation(
            Rc::new(expected_intersection_1),
            Rc::new(expected_intersection_2),
        );

        assert_eq!(gen_regex_unwrapped, expected);
    }

    #[test]
    fn test_range() {
        let smt_result = parse_smtlib_file("benchmarks/range1.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();

        let expected = GenRegex::Intersect(
            GenRegex::create_gre_str_var("x"),
            GenRegex::re_range(&'0', &'9'),
        );

        assert_eq!(gen_regex_unwrapped, expected);
    }

    #[test]
    fn test_re_all() {
        let smt_result = parse_smtlib_file("benchmarks/re_all.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();

        let union = GenRegex::union(
            &GenRegex::create_gre_char_lit("a"),
            &GenRegex::create_gre_char_lit("b"),
        );
        let regex = GenRegex::concat(&GenRegex::star(&GenRegex::create_sigma()), &union);
        let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

        assert_eq!(gen_regex_unwrapped, expected);
    }

    #[test]
    fn test_date() {
        let smt_result = parse_smtlib_file("benchmarks/date.smt2");
        println!("Parsed s-expression: {:?}\n", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();

        let dot_star = GenRegex::star(&GenRegex::create_sigma());
        let mut days_of_the_week: Vec<Rc<GenRegex>> = Vec::new();
        days_of_the_week.push(GenRegex::str_to_re("monday"));
        days_of_the_week.push(GenRegex::str_to_re("tuesday"));
        days_of_the_week.push(GenRegex::str_to_re("wednesday"));
        days_of_the_week.push(GenRegex::str_to_re("thursday"));
        days_of_the_week.push(GenRegex::str_to_re("friday"));
        days_of_the_week.push(GenRegex::str_to_re("saturday"));
        days_of_the_week.push(GenRegex::str_to_re("sunday"));
        let mut union_days = days_of_the_week[0].clone();
        for v in &days_of_the_week[1..] {
            union_days = GenRegex::union(&union_days, v);
        }
        let mut months: Vec<Rc<GenRegex>> = Vec::new();
        months.push(GenRegex::str_to_re("january"));
        months.push(GenRegex::str_to_re("february"));
        months.push(GenRegex::str_to_re("march"));
        months.push(GenRegex::str_to_re("april"));
        months.push(GenRegex::str_to_re("may"));
        months.push(GenRegex::str_to_re("june"));
        months.push(GenRegex::str_to_re("july"));
        months.push(GenRegex::str_to_re("august"));
        months.push(GenRegex::str_to_re("september"));
        months.push(GenRegex::str_to_re("october"));
        months.push(GenRegex::str_to_re("november"));
        months.push(GenRegex::str_to_re("december"));
        let mut union_months = months[0].clone();
        for v in &months[1..] {
            union_months = GenRegex::union(&union_months, v);
        }
        let first = GenRegex::concat(
            &GenRegex::concat(&dot_star.clone(), &union_days),
            &dot_star.clone(),
        );
        let second = GenRegex::concat(
            &GenRegex::concat(&dot_star.clone(), &union_months),
            &dot_star.clone(),
        );
        let third = GenRegex::star(&GenRegex::re_range(&'!', &'~'));
        let regex = GenRegex::intersect(&&GenRegex::intersect(&first, &second), &third);
        let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

        assert_eq!(gen_regex_unwrapped, expected);
    }
    #[test]
    fn test_date_2() {
        fn create_case_insensitive(word: &str) -> Rc<GenRegex> {
            //init first character of word
            let first_char = word.chars().next().unwrap();
            let mut curr_regex = GenRegex::str_to_re(&first_char.to_uppercase().to_string());
            let lower = GenRegex::str_to_re(&first_char.to_lowercase().to_string());
            curr_regex = GenRegex::union(&curr_regex, &lower);

            //iterate over word and add union of upper and lowercase versions
            for c in word[1..].chars() {
                let lower = GenRegex::str_to_re(&c.to_lowercase().to_string());
                let upper = GenRegex::str_to_re(&c.to_uppercase().to_string());
                let char_union = GenRegex::union(&upper, &lower);

                curr_regex = GenRegex::concat(&curr_regex, &char_union);
            }

            curr_regex
        }
        let smt_result = parse_smtlib_file("benchmarks/date2.smt2");
        println!("Parsed s-expression: {:?}\n", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();

        let dot_star = GenRegex::star(&GenRegex::create_sigma());
        let mut days_of_the_week: Vec<Rc<GenRegex>> = Vec::new();
        days_of_the_week.push(create_case_insensitive("monday"));
        days_of_the_week.push(create_case_insensitive("tuesday"));
        days_of_the_week.push(create_case_insensitive("wednesday"));
        days_of_the_week.push(create_case_insensitive("thursday"));
        days_of_the_week.push(create_case_insensitive("friday"));
        days_of_the_week.push(create_case_insensitive("saturday"));
        days_of_the_week.push(create_case_insensitive("sunday"));
        let mut union_days = days_of_the_week[0].clone();
        for v in &days_of_the_week[1..] {
            union_days = GenRegex::union(&union_days, v);
        }
        let mut months: Vec<Rc<GenRegex>> = Vec::new();
        months.push(create_case_insensitive("january"));
        months.push(create_case_insensitive("february"));
        months.push(create_case_insensitive("march"));
        months.push(create_case_insensitive("april"));
        months.push(create_case_insensitive("may"));
        months.push(create_case_insensitive("june"));
        months.push(create_case_insensitive("july"));
        months.push(create_case_insensitive("august"));
        months.push(create_case_insensitive("september"));
        months.push(create_case_insensitive("october"));
        months.push(create_case_insensitive("november"));
        months.push(create_case_insensitive("december"));
        let mut union_months = months[0].clone();
        for v in &months[1..] {
            union_months = GenRegex::union(&union_months, v);
        }
        let first = GenRegex::concat(
            &GenRegex::concat(&dot_star.clone(), &union_days),
            &dot_star.clone(),
        );
        let second = GenRegex::concat(
            &GenRegex::concat(&dot_star.clone(), &union_months),
            &dot_star.clone(),
        );
        //let third = GenRegex::star(&GenRegex::re_range(&'!', &'~'));
        let regex = GenRegex::intersect(&first, &second);
        let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

        assert_eq!(gen_regex_unwrapped, expected);
    }
    #[ignore]
    #[test]
    fn test_passw_sat1() {
        let smt_result = parse_smtlib_file("benchmarks/passw_sat1.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
    }

    fn test_passw_unsat1() {
        let smt_result = parse_smtlib_file("benchmarks/passw_unsat1.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
    }

    #[ignore]
    #[test]
    fn test_passw_eq_sat1() {
        let smt_result = parse_smtlib_file("benchmarks/passw_eq_sat1.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();

        let union = GenRegex::union(
            &GenRegex::create_gre_char_lit("a"),
            &GenRegex::create_gre_char_lit("b"),
        );
        let regex = GenRegex::concat(&GenRegex::star(&GenRegex::create_sigma()), &union);
        let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

        assert_eq!(gen_regex_unwrapped, expected);
    }
    #[test]
    fn test_hex_code() {
        let smt_result = parse_smtlib_file("benchmarks/hexcode.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();

        let union = GenRegex::union(
            &GenRegex::create_gre_char_lit("a"),
            &GenRegex::create_gre_char_lit("b"),
        );
        let regex = GenRegex::concat(&GenRegex::star(&GenRegex::create_sigma()), &union);
        let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

        assert_eq!(gen_regex_unwrapped, expected);
    }
}
