//!
//! Parsing for SMTLib files
//!

// TODO fix
#![allow(clippy::useless_format)]

use super::classes::GenRegex;

use regex::Regex;

use lexpr::{self, Value};

use std::collections::{HashMap, HashSet};
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
    BadCheckSat(),                   // Bad or missing (check-sat) statement in SMTLib file
    Unsupported(String),             // Unsupported SMTLib feature
    Unrecognized(String),            // Unrecognized SMTLib feature
    Unimplemented(String),           // Unimplemented SMTLib feature
    BadLiteral(String),              // Bad literal in SMTLib file
    Unexpected(String, String),      // Unexpected S-expression
}

impl SmtParseError {
    fn unrecog(expr: &Value) -> SmtParseError {
        SmtParseError::Unrecognized(expr.to_string())
    }
    fn bad_literal(expr: &Value) -> SmtParseError {
        SmtParseError::BadLiteral(expr.to_string())
    }
    fn unexpected(got: &Value, expected: &str) -> SmtParseError {
        SmtParseError::Unexpected(got.to_string(), expected.to_string())
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
            SmtParseError::BadCheckSat() => {
                write!(f, "Expected (check-sat) statement at end of file")
            }
            SmtParseError::Unsupported(s) => write!(f, "Unsupported SMTLib feature: {}", s),
            SmtParseError::Unrecognized(s) => write!(f, "Unrecognized S-expression: {}", s),
            SmtParseError::Unimplemented(s) => write!(f, "Unimplemented SMTLib feature: {}", s),
            SmtParseError::BadLiteral(s) => write!(f, "Bad literal in S-expression: {}", s),
            SmtParseError::Unexpected(got, expected) => {
                write!(
                    f,
                    "Unexpected S-expression: got {}, expected {}",
                    got, expected
                )
            }
        }
    }
}

impl Error for SmtParseError {}

/*
    S-expression parsing functions

    These are private so that the implementation can be changed later
*/

fn expect_pair(v: &Value) -> Result<(&Value, &Value), SmtParseError> {
    v.as_pair().ok_or(SmtParseError::unexpected(v, "pair"))
}

fn expect_null(v: &Value) -> Result<(), SmtParseError> {
    v.as_null().ok_or(SmtParseError::unexpected(v, "null"))
}

fn expect_symbol(v: &Value) -> Result<&str, SmtParseError> {
    v.as_symbol().ok_or(SmtParseError::unexpected(v, "symbol"))
}

fn hex_to_char(number: u64) -> Result<char, SmtParseError> {
    char::from_u32(number as u32).ok_or(SmtParseError::FileError(format!(
        "Invalid hex value: {}",
        number
    )))
}

fn parse_unicode_escape(text: &str) -> Result<String, SmtParseError> {
    fn replace_all<E>(
        re: &Regex,
        haystack: &str,
        replacement: impl Fn(&regex::Captures) -> Result<String, E>,
    ) -> Result<String, E> {
        let mut new = String::with_capacity(haystack.len());
        let mut last_match = 0;
        for caps in re.captures_iter(haystack) {
            let m = caps.get(0).unwrap();
            new.push_str(&haystack[last_match..m.start()]);
            new.push_str(&replacement(&caps)?);
            last_match = m.end();
        }
        new.push_str(&haystack[last_match..]);
        Ok(new)
    }
    // Regex pattern for unicode escapes \u{Hex}
    // Does not check invalid hex
    let unicode_escape_re = Regex::new(r"\\u\{([0-9A-Fa-f]+)\}").unwrap();

    replace_all(&unicode_escape_re, text, |caps: &regex::Captures| {
        // Unwrap is okay since regex check between 0-f for hex
        let hex_value = u32::from_str_radix(&caps[1], 16).unwrap();
        match char::from_u32(hex_value) {
            Some(v) => Ok(v.to_string()),
            // Error on invalid hex
            None => Err(SmtParseError::FileError(format!(
                "Bad hex in unicode escape {:?}",
                hex_value
            ))),
        }
    })
}

fn parse_smtlib_string(smt_string: &str) -> Result<Value, SmtParseError> {
    let v = lexpr::from_str(smt_string)?;
    Ok(v)
}

pub fn parse_smtlib_file(file_path: &str) -> Result<Value, SmtParseError> {
    // Read in the file
    let smt_string = std::fs::read_to_string(file_path)?;

    // Add an opening and closoing paren
    let smt_string = format!("(\n{}\n)", smt_string);
    let smt_string = parse_unicode_escape(&smt_string)?;

    // Parse S-expression
    let v = lexpr::from_str(&smt_string)?;

    // Return
    Ok(v)
}

/*
    Main parsing interface
*/

enum RegexToken {
    Var(String),
    Val(Rc<GenRegex>),
}
enum StringToken {
    Var(String),
    Val(String),
}

pub struct SmtParser {
    found_assert: bool,
    found_check_sat: bool,
    str_var_names: HashSet<String>,
    func_names: HashMap<String, String>,
    re_var_names: HashMap<String, Option<Rc<GenRegex>>>,
    let_var_regexes: HashMap<String, Rc<GenRegex>>,
    let_var_asserts: HashMap<String, Rc<GenRegex>>,
    regex_result: Option<Rc<GenRegex>>,
    brzozowski_flag: bool,
}

impl SmtParser {
    pub fn new() -> Self {
        Self {
            found_assert: false,
            found_check_sat: false,
            str_var_names: HashSet::new(),
            func_names: HashMap::new(),
            re_var_names: HashMap::new(),
            let_var_regexes: HashMap::new(),
            let_var_asserts: HashMap::new(),
            regex_result: None,
            brzozowski_flag: false,
        }
    }

    /*
        Parsing entrypoint and public API

        The main parse_s_expr takes input from lexpr::Value.
    */

    /// Parse list of items at the top level recursively
    pub fn parse_s_expr(&mut self, v: &Value) -> Result<GenRegex, SmtParseError> {
        // Note: this function is written recursively (may stack overflow)
        // We may want to rewrite to be iterative eventually.

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
                return Err(SmtParseError::BadCheckSat());
            }
            let result = self.regex_result.take();
            Ok(Rc::unwrap_or_clone(result.expect(
                "Regex result should have been set by parser earlier!",
            )))
        } else {
            Err(SmtParseError::unrecog(v))
        }
    }

    pub fn use_brzozowski(&self) -> bool {
        self.brzozowski_flag
    }

    /*
        Top-level parsing functions
    */

    fn parse_head(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // 4 cases here: (declare-const), (assert), (check-sat), (define-fun)
        if let Some((head, tail)) = v.as_pair() {
            match head.as_symbol().ok_or(SmtParseError::unrecog(head))? {
                "set-logic" => Ok(()),
                "declare-const" => self.parse_declare_const(tail),
                "assert" => self.parse_assert(tail),
                "check-sat" => self.parse_check_sat(tail),
                "define-fun" => self.parse_define_fun(tail),
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
        // Declare const should not occur after check-sat
        if self.found_check_sat {
            return Err(SmtParseError::BadCheckSat());
        }
        // Add variable name to self.var_names
        let (var_name, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (var_type, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        match var_type.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "String" => {
                self.str_var_names.insert(var_name.to_string());
                Ok(())
            }
            "RegLan" => {
                self.re_var_names.insert(var_name.to_string(), None);
                Ok(())
            }
            _ => Err(SmtParseError::unrecog(v)),
        }
    }

    fn parse_assert(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // Assert should not occur after check-sat
        if self.found_check_sat {
            return Err(SmtParseError::BadCheckSat());
        }
        // Set flag
        self.found_assert = true;
        // Parse the arg
        let (assert_arg, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        let result = self.parse_assert_arg(assert_arg)?;
        if let Some(r) = &self.regex_result {
            self.regex_result = Some(Rc::new(GenRegex::Concatenation(r.clone(), result)));
            Ok(())
        } else {
            self.regex_result = Some(result);
            Ok(())
        }
    }

    fn parse_check_sat(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // check-sat should occur only once
        if self.found_check_sat {
            return Err(SmtParseError::BadCheckSat());
        }
        // Set flag
        self.found_check_sat = true;
        expect_null(v)?;
        Ok(())
    }

    fn parse_define_fun(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // Syntax: (define-fun [fun name] () String [fun defn])
        let args = self.get_args(&v)?;
        if args.len() != 4 {
            return Err(SmtParseError::unrecog(v));
        }
        let (name, params, ret_type, defn) = (args[0], args[1], args[2], args[3]);
        //Ensure params and return type are valid
        match params {
            Value::Null => (),
            Value::Cons(_) => {
                return Err(SmtParseError::Unsupported(format!(
                    "Function parameters currently not supported."
                )))
            }
            _ => return Err(SmtParseError::unrecog(params)),
        };
        match ret_type
            .as_symbol()
            .ok_or(SmtParseError::unrecog(ret_type))?
        {
            "String" => (),
            "RegLan" => {
                return Err(SmtParseError::Unsupported(format!(
                    "Functions with RegLan output currently not supported."
                )))
            }
            _ => return Err(SmtParseError::unrecog(params)),
        };
        //Parses the function definition and inserts into HashMap
        let constructed_string = self.parse_str(defn)?;
        self.func_names.insert(
            name.as_symbol()
                .ok_or(SmtParseError::unrecog(name))?
                .to_string(),
            constructed_string,
        );
        Ok(())
    }
    /*
        Parsing functions which return a GenRegex representing a specific SMTLib assertion
    */

    fn parse_assert_arg(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        println!("called parse_assert_arg: {:?}", v);

        // Parse the command. Assume the command always is Cons or a single symbol

        // Let variable case
        // TBD: currently parse_assert_arg can be called for a single symbol,
        // in the let expression case.
        // this seems a bit odd though. Maybe some other function is calling it wrong.
        if let Some(name) = v.as_symbol() {
            if let Some(let_result) = self.let_var_asserts.get(name) {
                return Ok(let_result.clone());
            } else {
                return Err(SmtParseError::unrecog(v));
            }
        }

        // Command cons case
        let (cmd, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let cmd_str = expect_symbol(cmd)?;
        match cmd_str {
            "str.in_re" => self.parse_str_in_re(tail),
            "and" => self.parse_and(tail),
            "=" => self.parse_equals(tail),
            "let" => self.parse_let_assertion(tail),
            _ => {
                // Check for let variable case a second time
                // println!("cmd_str: {:?}", cmd_str);
                self.parse_assert_arg(cmd)
            }
        }
    }

    fn parse_equals(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        //Assumes RegLan on both sides of =
        //Todo: support String equality?
        let (regex1, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex2, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        let parsed1 = self.parse_reglan_type(regex1)?;
        let parsed2 = self.parse_reglan_type(regex2)?;
        //Initializes variables if its var=Regex
        //Asserts equality if Regex=Regex
        //Will return epsilin in case of initialization
        match (parsed1, parsed2) {
            (RegexToken::Var(_), RegexToken::Var(_)) => Err(SmtParseError::Unsupported(format!(
                "Equality of uninitialzied RegLan variables not supported."
            ))),
            (RegexToken::Var(name), RegexToken::Val(gen_regex)) => {
                let res = self.re_var_names.get(&name);
                if let Some(found) = res {
                    match found {
                        Some(_) => Err(SmtParseError::Unsupported(format!(
                            "Conflicting RegLan initilizations are caught here instead of solver."
                        ))),
                        None => {
                            self.re_var_names.insert(name, Some(gen_regex));
                            Ok(GenRegex::epsilon())
                        }
                    }
                } else {
                    Err(SmtParseError::Unrecognized(format!(
                        "RegLan variable not declared/found: {}",
                        name
                    )))
                }
            }
            (RegexToken::Val(gen_regex), RegexToken::Var(name)) => {
                let res = self.re_var_names.get(&name);
                if let Some(found) = res {
                    match found {
                        Some(_) => Err(SmtParseError::Unsupported(format!(
                            "Conflicting RegLan initilizations are caught here instead of solver."
                        ))),
                        None => {
                            self.re_var_names.insert(name, Some(gen_regex));
                            Ok(GenRegex::epsilon())
                        }
                    }
                } else {
                    Err(SmtParseError::Unrecognized(format!(
                        "RegLan variable not declared/found: {}",
                        name
                    )))
                }
            }
            (RegexToken::Val(gen_regex1), RegexToken::Val(gen_regex2)) => {
                self.brzozowski_flag = true;
                Ok(GenRegex::union(
                &GenRegex::intersect(&gen_regex1, &GenRegex::complement(&gen_regex2)),
                &GenRegex::intersect(&GenRegex::complement(&gen_regex1), &gen_regex2),
            ))},
        }
    }

    fn parse_and(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (and cmd cmd)
        if let Value::Cons(c) = v {
            let (head, tail) = c.as_pair();
            let regex1 = self.parse_assert_arg(head)?;
            if let Value::Cons(c) = tail {
                let (head, tail) = c.as_pair();
                expect_null(tail)?;
                let regex2 = self.parse_assert_arg(head)?;
                return Ok(GenRegex::concat(&regex1, &regex2));
            }
        }
        Err(SmtParseError::unrecog(v))
    }

    fn parse_str_in_re(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (str.in_re x R)
        let (string, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        //Chooses behavior based on string and regex tokens
        let str_tok = self.parse_string_type(string)?;
        let regex_tok = self.parse_reglan_type(regex)?;
        match (str_tok, regex_tok) {
            (StringToken::Var(var_name), RegexToken::Val(gen_regex)) => Ok(GenRegex::intersect(
                &GenRegex::create_gre_str_var(&var_name),
                &gen_regex,
            )),
            (StringToken::Val(string), RegexToken::Val(gen_regex)) => Ok(GenRegex::intersect(
                &GenRegex::str_to_re(&string),
                &gen_regex,
            )),
            _ => Err(SmtParseError::Unsupported(format!(
                "RegLan variable in str.in_re needs to be initialzied beforehand."
            ))),
        }
    }

    /*
        Parsing functions for let expressions
    */

    fn parse_let_assertion(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        println!("called let_assertion: {:?}", v);
        // Parse the let declaration part
        let expr_tail = self.parse_let_declaration(v)?;
        // Recurse on the tail expression
        println!("let_assertion expr_tail: {:?}", expr_tail);
        let (assert_arg, assert_tail) = expect_pair(expr_tail)?;
        expect_null(assert_tail)?;
        let result = self.parse_assert_arg(assert_arg);
        println!("let_assertion result: {:?}", result);
        result
    }

    fn parse_let_regex(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        println!("called let_regex: {:?}", v);
        // Parse the let declaration part
        let expr_tail = self.parse_let_declaration(v)?;
        // Recurse on the tail expression
        println!("let_regex expr_tail: {:?}", expr_tail);
        let (regex_arg, regex_tail) = expect_pair(expr_tail)?;
        expect_null(regex_tail)?;
        let result = self.parse_regex(regex_arg);
        println!("let_regex result: {:?}", result);
        result
    }

    fn parse_let_declaration<'b>(&mut self, v: &'b Value) -> Result<&'b Value, SmtParseError> {
        // Helper function which parses the let declaration, stores the variable as a hashmap entry,
        // and returns the tail expression.

        // Decompose the expression
        // Underscored parts should be null
        // ((let3 (let4 _tail4_)) _tail2_) tail1
        let (let1, tail1) = expect_pair(v)?;
        let (let2, tail2) = expect_pair(let1)?;
        let (let3, tail3) = expect_pair(let2)?;
        let (let4, tail4) = expect_pair(tail3)?;
        expect_null(tail2)?;
        expect_null(tail4)?;

        // Extract the important parts
        // (let ((symbol (regex))) expr)
        //        ^let3   ^let4    ^tail1
        let let_symbol = expect_symbol(let3)?;
        let let_result = let4;

        // Try parsing as either a regex or as an assertion, and insert into the hashmap
        if let Ok(let_regex) = self.parse_regex(let_result) {
            self.let_var_regexes
                .insert(let_symbol.to_string(), let_regex);
        } else if let Ok(let_assert) = self.parse_assert_arg(let_result) {
            self.let_var_asserts
                .insert(let_symbol.to_string(), let_assert);
        } else {
            return Err(SmtParseError::unrecog(let_result));
        }

        // Return the expression to be evaluated
        Ok(tail1)
    }

    /*
        Parsing functions with output RegLan
    */

    //parse_reglan_type must be used in all places that take reglan as input
    fn parse_reglan_type(&mut self, v: &Value) -> Result<RegexToken, SmtParseError> {
        //If is a variable returns var name if uninitialized and initialized value o.w.
        //If not variable parses the regex
        if let Some(name) = v.as_symbol() {
            let res = self.re_var_names.get(name);
            match res {
                Some(found) => match found {
                    Some(re) => Ok(RegexToken::Val(re.clone())),
                    None => Ok(RegexToken::Var(name.to_string())),
                },
                None => Ok(RegexToken::Val(self.parse_regex(v)?)),
            }
        } else {
            Ok(RegexToken::Val(self.parse_regex(v)?))
        }
    }

    fn parse_regex(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        println!("called parse_regex: {:?}", v);
        // Handles base case regex
        if let Some(re_type) = v.as_symbol() {
            return match re_type {
                "re.all" => self.parse_re_all(),
                "re.none" => self.parse_re_none(),
                "re.allchar" => self.parse_re_allchar(),
                _ => {
                    // Check for let variable
                    if let Some(let_result) = self.let_var_regexes.get(re_type) {
                        Ok(let_result.clone())
                    } else {
                        Err(SmtParseError::unrecog(v))
                    }
                }
            };
        }

        // Handles recursive case
        let (re_type, args) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;

        // re_func case
        if let Some((head, tail)) = re_type.as_pair() {
            return match head.as_symbol().ok_or(SmtParseError::unrecog(v))? {
                "_" => self.parse_re_func(tail, args),
                _ => Err(SmtParseError::unrecog(v)),
            };
        }

        // All other cases
        let result = match re_type.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "let" => self.parse_let_regex(args),
            "str.to_re" => self.parse_str_to_re(args),
            "re.++" => self.parse_re_concat(args),
            "re.union" => self.parse_re_union(args),
            "re.diff" => self.parse_re_diff(args),
            "re.*" => self.parse_re_star(args),
            "re.inter" => self.parse_re_inter(args),
            "re.range" => self.parse_re_range(args),
            "re.comp" => self.parse_re_comp(args),
            "re.+" => self.parse_re_plus(args),
            "re.opt" => self.parse_re_opt(args),
            _ => Err(SmtParseError::unrecog(re_type)),
        };
        println!("parse_regex result: {:?}", result);
        result
    }

    fn parse_re_union(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (re.union R1 R2 ...)
        let args = self.get_args(v)?;
        if args.len() < 2 {
            return Err(SmtParseError::unrecog(v));
        }
        let mut regex_args: Vec<Rc<GenRegex>> = Vec::new();
        for a in args {
            regex_args.push(self.parse_regex(a)?);
        }
        Ok(GenRegex::union_many(&regex_args))
    }

    fn parse_re_diff(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (re.diff R R)
        self.brzozowski_flag = true;
        let (regex1, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex2, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        Ok(GenRegex::intersect(
            &self.parse_regex(regex1)?,
            &GenRegex::complement(&self.parse_regex(regex2)?),
        ))
    }

    fn parse_re_inter(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (re.inter R1 R2 ...)
        let args = self.get_args(v)?;
        if args.len() < 2 {
            return Err(SmtParseError::unrecog(v));
        }
        let mut regex_args: Vec<Rc<GenRegex>> = Vec::new();
        for a in args {
            regex_args.push(self.parse_regex(a)?);
        }
        Ok(GenRegex::intersect_many(&regex_args))
    }

    fn parse_re_star(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (re.* R)
        // Returns R*
        let (regex, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        Ok(GenRegex::star(&self.parse_regex(regex)?))
    }

    fn parse_re_all(&self) -> Result<Rc<GenRegex>, SmtParseError> {
        Ok(GenRegex::star(&GenRegex::create_sigma()))
    }

    fn parse_re_none(&self) -> Result<Rc<GenRegex>, SmtParseError> {
        Ok(GenRegex::empty_set())
    }

    fn parse_re_concat(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (re.++ R1 R2 ...)
        let args = self.get_args(v)?;
        if args.len() < 2 {
            return Err(SmtParseError::unrecog(v));
        }
        let mut regex_args: Vec<Rc<GenRegex>> = Vec::new();
        for a in args {
            regex_args.push(self.parse_regex(a)?);
        }
        Ok(GenRegex::concat_many(&regex_args))
    }

    fn parse_str_to_re(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // (str.to_re "String")
        let (str, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        Ok(GenRegex::str_to_re(
            str.as_str().ok_or(SmtParseError::unrecog(v))?,
        ))
    }

    fn parse_re_range(&self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax (re.range char1 char2)
        let (char1, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (char2, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        println!("{}, 2{}, tail {}", char1, char2, tail);
        expect_null(tail)?;
        let char1 = self.parse_char_obj(char1)?.to_string();
        let char2 = self.parse_char_obj(char2)?.to_string();

        if char1.chars().count() != 1 || char2.chars().count() != 1 {
            return Err(SmtParseError::unrecog(v));
        }
        if let (Some(char1), Some(char2)) = (char1.chars().next(), char2.chars().next()) {
            return Ok(GenRegex::re_range(&char1, &char2));
        }
        Err(SmtParseError::unrecog(v))
    }

    fn parse_re_func(&mut self, func: &Value, args: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        println!("re_fun");
        let (re_func, func_params) = func.as_pair().ok_or(SmtParseError::unrecog(func))?;
        match re_func.as_symbol().ok_or(SmtParseError::unrecog(func))? {
            "re.loop" => {
                let (param1_val, tail) = func_params
                    .as_pair()
                    .ok_or(SmtParseError::unrecog(func_params))?;
                if tail.is_null() {
                    return self.parse_re_loop(param1_val, &Value::Null, args);
                }
                let (param2_val, tail) =
                    tail.as_pair().ok_or(SmtParseError::unrecog(func_params))?;
                expect_null(tail)?;
                let (regex, tail) = args.as_pair().ok_or(SmtParseError::unrecog(args))?;
                expect_null(tail)?;
                self.parse_re_loop(param1_val, param2_val, regex)
            }
            "char" => {
                println!("what the heckles"); // Lol
                                              // Should we call parse_char_obj here?
                todo!();
            }
            "re.^" => {
                let (param1_val, tail) = func_params
                    .as_pair()
                    .ok_or(SmtParseError::unrecog(func_params))?;
                expect_null(tail)?;
                let (regex, tail) = args.as_pair().ok_or(SmtParseError::unrecog(args))?;
                expect_null(tail)?;
                self.parse_re_caret(param1_val, regex)
            }
            _ => Err(SmtParseError::unrecog(func)),
        }
    }

    fn parse_re_caret(
        &mut self,
        param1: &Value,
        regex: &Value,
    ) -> Result<Rc<GenRegex>, SmtParseError> {
        let p1 = param1.as_number();
        let p1 = p1.ok_or(SmtParseError::unrecog(param1))?;
        let p1 = p1.as_u64().ok_or(SmtParseError::Unrecognized(
            "Integer for re.loop should be positive.".to_string(),
        ))?;
        let regex_base = self.parse_regex(regex)?;
        Ok(GenRegex::caret(p1, &regex_base))
    }

    fn parse_re_loop(
        &mut self,
        param1: &Value,
        param2: &Value,
        regex: &Value,
    ) -> Result<Rc<GenRegex>, SmtParseError> {
        let (p1, p2) = (param1.as_number(), param2.as_number());
        let p1 = p1.ok_or(SmtParseError::unrecog(param1))?;
        let p2 = p2.ok_or(SmtParseError::unrecog(param1))?;
        let p1 = p1.as_u64().ok_or(SmtParseError::Unrecognized(
            "Integer for re.loop should be positive.".to_string(),
        ))?;
        let p2 = p2.as_u64().ok_or(SmtParseError::Unrecognized(
            "Integer for re.loop should be positive.".to_string(),
        ))?;
        let regex_base = self.parse_regex(regex)?;
        Ok(GenRegex::re_loop(p1, p2, &regex_base))
    }

    pub fn parse_re_allchar(&self) -> Result<Rc<GenRegex>, SmtParseError> {
        Ok(GenRegex::create_sigma())
    }

    pub fn parse_re_comp(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        self.brzozowski_flag = true;
        let (regex, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        Ok(GenRegex::complement(&self.parse_regex(regex)?))
    }

    pub fn parse_re_opt(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        let (regex, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        Ok(GenRegex::union(
            &self.parse_regex(regex)?,
            &GenRegex::create_gre_char_lit(""),
        ))
    }

    pub fn parse_re_plus(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        let (regex, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        let regex = self.parse_regex(regex)?;
        Ok(GenRegex::concat(&regex, &GenRegex::star(&regex)))
    }

    /*
       Parsing functions with output String/Char
    */

    //parse_reglan_type must be used in all places that take reglan as input
    fn parse_string_type(&mut self, v: &Value) -> Result<StringToken, SmtParseError> {
        //If is a variable returns var name if uninitialized and initialized value o.w.
        //If not variable parses the regex
        if let Some(name) = v.as_symbol() {
            let res = self.func_names.get(name);
            if let Some(s) = res {
                Ok(StringToken::Val(s.to_string()))
            } else if self.str_var_names.contains(name) {
                Ok(StringToken::Var(name.to_string()))
            } else {
                Err(SmtParseError::BadLiteral(format!(
                    "{} is not found in declared variables or defined functions.",
                    name
                )))
            }
        } else {
            Err(SmtParseError::unrecog(v))
        }
    }

    fn parse_char_obj(&self, v: &Value) -> Result<char, SmtParseError> {
        // println!("char_obj: {:?}", v);
        if v.is_string() {
            if v.as_str().unwrap().chars().count() == 1 {
                let v_char = v.as_str().unwrap().chars().next().unwrap();
                Ok(v_char)
            } else {
                Err(SmtParseError::bad_literal(v))
            }
        } else if v.is_cons() {
            // Removes underscore
            // TODO: validate initial characters
            let (_underscore, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
            let (_, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
            let (hex, _tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
            let hex_val = hex.as_u64().ok_or(SmtParseError::bad_literal(hex))?;
            hex_to_char(hex_val)
        } else {
            Err(SmtParseError::bad_literal(v))
        }
    }

    fn parse_str_at(&self, v: &Value) -> Result<String, SmtParseError> {
        let (string, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (index, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        expect_null(tail)?;
        if index.is_number() {
            Ok(string
                .as_str()
                .ok_or(SmtParseError::unrecog(v))?
                .chars()
                .nth(index.as_u64().ok_or(SmtParseError::unrecog(v))? as usize)
                .ok_or(SmtParseError::unrecog(v))?
                .to_string())
        } else {
            Err(SmtParseError::unrecog(v))
        }
    }

    fn parse_str(&self, v: &Value) -> Result<String, SmtParseError> {
        // Handles String literals
        if let Some(s) = v.as_str() {
            return Ok(s.to_string());
        }
        let (str_type, args) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        // Handles recursive strings
        match str_type.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "str.at" => self.parse_str_at(args),
            "str.++" => self.parse_str_concat(args),
            _ => Err(SmtParseError::unrecog(str_type)),
        }
    }

    fn parse_str_concat(&self, v: &Value) -> Result<String, SmtParseError> {
        // Syntax: (str.++ String String)
        let args = self.get_args(v)?;
        if args.len() != 2 {
            return Err(SmtParseError::unrecog(v));
        }
        let (string1, string2) = (args[0], args[1]);
        let string1 = self.parse_str(string1)?;
        let string2 = self.parse_str(string2)?;
        Ok(format!("{}{}", string1, string2))
    }

    /*
       Helper Functions
    */

    fn get_args<'a>(&self, v: &'a Value) -> Result<Vec<&'a Value>, SmtParseError> {
        if !v.is_null() && !v.is_cons() {
            return Err(SmtParseError::unrecog(v));
        }
        let mut retval: Vec<&Value> = Vec::new();
        let mut curval = v;
        while !curval.is_null() {
            let (head, tail) = curval.as_pair().ok_or(SmtParseError::unrecog(curval))?;
            retval.push(head);
            curval = tail;
        }
        Ok(retval)
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

    use crate::antimirov::satisfiable;
    use crate::brzozowski;
    use crate::classes::{CharExpression, GenRegex, StringVar};

    // Helper function
    // TODO: Update some of the other tests to use this
    // Run the SMT2 file and assert that satisfiable() returns as expected
    fn assert_smt2_file_helper(filepath: &str, expected: bool) {
        // Read the file and parse as s-expression
        let smt_result = parse_smtlib_file(filepath);
        println!("Parsed s-expression: {:?}", smt_result);
        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);
        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();

        // Get result
        let result: bool = if parser.use_brzozowski() {
            brzozowski::satisfiable(&Rc::new(gen_regex_unwrapped))
        } else {
            satisfiable(&Rc::new(gen_regex_unwrapped))
        };
        assert_eq!(result, expected);
    }

    fn assert_satisfiable(filepath: &str) {
        assert_smt2_file_helper(filepath, true);
    }

    fn assert_unsatisfiable(filepath: &str) {
        assert_smt2_file_helper(filepath, false);
    }

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
        let smt_result = parse_smtlib_file("benchmarks/simple1_sat.smt2");
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

        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), true);
    }

    #[test]
    fn test_simple_2() {
        let smt_result = parse_smtlib_file("benchmarks/simple2_unsat.smt2");
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

        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), false);
    }

    #[test]
    fn test_simple_3() {
        let smt_result = parse_smtlib_file("benchmarks/simple3_sat.smt2");
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
        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), true);

        assert_eq!(gen_regex_unwrapped, expected);
    }

    #[test]
    fn test_range() {
        let smt_result = parse_smtlib_file("benchmarks/range1_sat.smt2");
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

        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), true);
    }

    #[test]
    fn test_re_all() {
        let smt_result = parse_smtlib_file("benchmarks/re_all_sat.smt2");
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

        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), true);
    }

    #[test]
    fn test_date() {
        let smt_result = parse_smtlib_file("benchmarks/date_sat.smt2");
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

        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), true);
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
        let smt_result = parse_smtlib_file("benchmarks/date2_sat.smt2");
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

        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), true);
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
        let gen_regex_unwrapped = gen_regex.unwrap();

        let dot_star = GenRegex::star(&GenRegex::create_sigma());
        let first = GenRegex::concat(
            &GenRegex::concat(&dot_star, &GenRegex::re_range(&'a', &'z')),
            &dot_star,
        );
        let second = GenRegex::concat(
            &GenRegex::concat(&dot_star, &GenRegex::re_range(&'A', &'Z')),
            &dot_star,
        );
        let third = GenRegex::concat(
            &GenRegex::concat(&dot_star, &GenRegex::re_range(&'0', &'9')),
            &dot_star,
        );
        let fourth = GenRegex::re_loop(0, 3, &GenRegex::re_range(&'!', &'~'));
        let regex = GenRegex::intersect(
            &GenRegex::intersect(&GenRegex::intersect(&first, &second), &third),
            &fourth,
        );
        let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

        assert_eq!(gen_regex_unwrapped, expected);
        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped)), true);
    }

    #[ignore]
    #[test]
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
        let gen_regex_unwrapped = gen_regex.unwrap();

        let dot_star = GenRegex::star(&GenRegex::create_sigma());
        let first = GenRegex::concat(
            &GenRegex::concat(&dot_star, &GenRegex::re_range(&'a', &'z')),
            &dot_star,
        );
        let second = GenRegex::concat(
            &GenRegex::concat(&dot_star, &GenRegex::re_range(&'A', &'Z')),
            &dot_star,
        );
        let third = GenRegex::concat(
            &GenRegex::concat(&dot_star, &GenRegex::re_range(&'0', &'9')),
            &dot_star,
        );
        let fourth = GenRegex::star(&GenRegex::re_range(&':', &'~'));
        let regex = GenRegex::intersect(
            &GenRegex::intersect(&GenRegex::intersect(&first, &second), &third),
            &fourth,
        );
        let expected = GenRegex::Intersect(GenRegex::create_gre_str_var("x"), regex);

        assert_eq!(gen_regex_unwrapped, expected);

        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), false);
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

        let dot_star = GenRegex::star(&GenRegex::create_sigma());
        let one = GenRegex::concat_many(&vec![
            dot_star.clone(),
            GenRegex::re_range(&'a', &'z'),
            dot_star.clone(),
        ]);
        let two = GenRegex::concat_many(&vec![
            dot_star.clone(),
            GenRegex::re_range(&'0', &'9'),
            dot_star.clone(),
        ]);
        let three = GenRegex::concat_many(&vec![
            dot_star.clone(),
            GenRegex::re_range(&'A', &'Z'),
            dot_star.clone(),
        ]);
        let four = GenRegex::re_loop(8, 20, &GenRegex::create_sigma());
        let five = GenRegex::star(&GenRegex::re_range(&'A', &'z'));
        let together = GenRegex::intersect_many(&vec![
            one.clone(),
            two.clone(),
            three.clone(),
            four.clone(),
            five.clone(),
        ]);
        let eq1 = GenRegex::intersect(&GenRegex::empty_set(), &GenRegex::complement(&together));
        let eq2 = GenRegex::intersect(&GenRegex::complement(&GenRegex::empty_set()), &together);
        let expected = GenRegex::Union(eq1, eq2);
        assert_eq!(gen_regex_unwrapped, expected);
        assert_eq!(brzozowski::satisfiable(&Rc::new(gen_regex_unwrapped)), true);
    }

    #[ignore]
    #[test]
    fn test_hex_code() {
        let smt_result = parse_smtlib_file("benchmarks/hexcode_sat.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();
        assert_eq!(
            brzozowski::satisfiable(&Rc::new(gen_regex_unwrapped.clone())),
            true
        );
    }

    #[test]
    fn test_simple_hex() {
        println!("A number{:?}", hex_to_char(0));
        let smt_result = parse_smtlib_file("benchmarks/simplehex_sat.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();
        let hex = hex_to_char(0x0).unwrap();
        let expected = GenRegex::Intersect(
            GenRegex::create_gre_str_var("x"),
            GenRegex::re_range(&hex, &'/'),
        );

        assert_eq!(gen_regex_unwrapped, expected);
        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), true);
    }

    #[test]
    fn unicode_hex_test() {
        assert_satisfiable("benchmarks/hex_syntax_test_sat.smt2");
    }

    #[ignore]
    #[test]
    fn intersect_test1() {
        assert_satisfiable("benchmarks/intersect_0_0_sat.smt2");
    }

    #[test]
    fn test_reglan_var() {
        assert_satisfiable("benchmarks/reglan_var_test_sat.smt2");
    }

    #[test]
    fn test_let_1() {
        assert_satisfiable("benchmarks/simple_let_sat_1.smt2");
    }

    #[test]
    fn test_let_2() {
        assert_satisfiable("benchmarks/simple_let_sat_2.smt2");
    }

    #[test]
    fn test_let_3() {
        assert_satisfiable("benchmarks/simple_let_sat_3.smt2");
    }

    #[ignore]
    #[test]
    fn test_let_4() {
        assert_satisfiable("benchmarks/date_format_days_sat.smt2");
    }

    #[test]
    fn test_define_fun1() {
        assert_satisfiable("benchmarks/simple_definefun_sat_1.smt2");
    }

    #[test]
    fn test_define_fun2() {
        assert_satisfiable("benchmarks/simple_definefun_sat_2.smt2");
    }
}
