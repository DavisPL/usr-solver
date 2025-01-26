//!
//! Parsing for SMTLib files
//!

use super::classes::GenRegex;

use regex::Regex;

use lexpr::{self, Value};

use std::collections::{HashSet,HashMap};
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
            SmtParseError::BadCheckSat() => {
                write!(f, "Expected (check-sat) statement at end of file")
            }
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

fn hex_to_char(number: u64) -> char {
    char::from_u32(number as u32).expect("hex_to_char:number needs to be a valid char")
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

enum RegexToken{
    Var(String),
    Val(Rc<GenRegex>),
}
enum StringToken{
    Var(String),
    Val(String),
}

pub struct SmtParser {
    found_assert: bool,
    found_check_sat: bool,
    str_var_names: HashSet<String>,
    re_var_names: HashMap<String, Option<Rc<GenRegex>>>,
    let_var_names: HashMap<String, Option<Rc<GenRegex>>>,
    regex_result: Option<GenRegex>,
    brzozowski_flag: bool,
}

impl SmtParser {
    pub fn new() -> Self {
        Self {
            found_assert: false,
            found_check_sat: false,
            str_var_names: HashSet::new(),
            re_var_names: HashMap::new(),
            let_var_names: HashMap::new(),
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
            Ok(result.expect("Regex result should have been set by parser earlier!"))
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
        // 3 cases here: (declare-const), (assert), (check-sat)
        if let Some((head, tail)) = v.as_pair() {
            match head.as_symbol().ok_or(SmtParseError::unrecog(head))? {
                "set-logic" => Ok(()),
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
        // Declare const should not occur after check-sat
        if self.found_check_sat {
            return Err(SmtParseError::BadCheckSat());
        }
        // Add variable name to self.var_names
        let (var_name, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (var_type, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        match var_type.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "String" => {
                self.str_var_names.insert(var_name.to_string());
                Ok(())
            }
            "RegLan" => {
                self.re_var_names.insert(var_name.to_string(),None);
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
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        let result = self.parse_assert_arg(assert_arg)?;
        if let Some(r) = &self.regex_result {
            self.regex_result = Some(GenRegex::Concatenation(Rc::new(r.clone()), result));
            Ok(())
        } else {
            self.regex_result = Some(Rc::try_unwrap(result).unwrap());
            Ok(())
        }
    }

    fn parse_assert_arg(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Parse the command. Going to assume the command always is Cons
        let (cmd, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        match cmd.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "str.in_re" => self.parse_str_in_re(tail),
            "and" => self.parse_and(tail),
            "=" => self.parse_equals(tail),
            "let" => self.parse_let(tail),
            _ => Err(SmtParseError::Unsupported(format!(
                "Unsupported SMTLib command: {}",
                cmd
            ))),
        }
    }

    fn parse_check_sat(&mut self, v: &Value) -> Result<(), SmtParseError> {
        // check-sat should occur only once
        if self.found_check_sat {
            return Err(SmtParseError::BadCheckSat());
        }
        // Set flag
        self.found_check_sat = true;
        if !v.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        Ok(())
    }

    /*
        Parsing functions for specific SMTLib assertions
    */

    fn parse_equals(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        //Assumes RegLan on both sides of =
        //Todo: support String equality?
        self.brzozowski_flag = true;
        let (regex1, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex2, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        let parsed1 = self.parse_reglan_type(regex1)?;
        let parsed2 = self.parse_reglan_type(regex2)?;
        //Initializes variables if its var=Regex
        //Asserts equality if Regex=Regex
        //Will return epsilin in case of initialization
        match (parsed1,parsed2){
            (RegexToken::Var(_), RegexToken::Var(_)) => Err(SmtParseError::Unsupported(format!("Equality of uninitialzied RegLan variables not supported."))),
            (RegexToken::Var(name), RegexToken::Val(gen_regex)) => {
                let res=self.re_var_names.get(&name);
                if let Some(found)=res{
                    match found{
                        Some(_) => Err(SmtParseError::Unsupported(format!("Conflicting RegLan initilizations are caught here instead of solver."))),
                        None => {
                            self.re_var_names.insert(name, Some(gen_regex));
                            Ok(GenRegex::epsilon())
                        },
                    }
                }
                else{
                    Err(SmtParseError::Unrecognized(format!(
                        "RegLan variable not declared/found: {}",
                        name
                    )))
                }
            },
            (RegexToken::Val(gen_regex), RegexToken::Var(name)) => {
                let res=self.re_var_names.get(&name);
                if let Some(found)=res{
                    match found{
                        Some(_) => Err(SmtParseError::Unsupported(format!("Conflicting RegLan initilizations are caught here instead of solver."))),
                        None => {
                            self.re_var_names.insert(name, Some(gen_regex));
                            Ok(GenRegex::epsilon())
                        },
                    }
                }
                else{
                    Err(SmtParseError::Unrecognized(format!(
                        "RegLan variable not declared/found: {}",
                        name
                    )))
                }
            },
            (RegexToken::Val(gen_regex1), RegexToken::Val(gen_regex2)) => {
                Ok(GenRegex::union(
                    &GenRegex::intersect(&gen_regex1, &GenRegex::complement(&gen_regex2)),
                    &GenRegex::intersect(&GenRegex::complement(&gen_regex1), &gen_regex2),
                ))
            },
        }
    }

    fn parse_and(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (and cmd cmd)
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

    fn parse_str_in_re(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (str.in_re x R)
        let (str_var, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (regex, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        //Check str_var is in var_names
        let str_var = str_var.as_symbol().ok_or(SmtParseError::unrecog(str_var))?;
        if self.str_var_names.contains(str_var) {
            //Construct str_var \cap R and return
            let str_var = GenRegex::create_gre_str_var(str_var);
            let regex_tok = self.parse_reglan_type(regex)?;
            match regex_tok{
                RegexToken::Var(name) => Err(SmtParseError::Unsupported(format!("{:?} in str.in_re needs to be initialzied beforehand.", name))),
                RegexToken::Val(gen_regex) => Ok(GenRegex::intersect(&str_var, &gen_regex)),
            }
        } else {
            Err(SmtParseError::Unrecognized(format!(
                "String variable not declared/found: {}",
                str_var
            )))
        }
    }

    fn parse_let(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        println!("VALUE: {:?}", v);
        let (let_var, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        unimplemented!()
    }

    /*
        Parsing functions with output RegLan
     */

    //parse_reglan_type must be used in all places that take reglan as input
    fn parse_reglan_type(&mut self,v:&Value)->Result<RegexToken,SmtParseError>{
        //If is a variable returns var name if uninitialized and initialized value o.w.
        //If not variable parses the regex
        if let Some(name) = v.as_symbol(){
            let res=self.re_var_names.get(name);
            match res{
                Some(found) => {
                    match found{
                        Some(re) => Ok(RegexToken::Val(re.clone())),
                        None => Ok(RegexToken::Var(name.to_string())),
                    }
                },
                None => Err(SmtParseError::unrecog(v)),
            }
        }
        else{
            Ok(RegexToken::Val(self.parse_regex(v)?))
        }
    }

    fn parse_regex(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Handles base case regex
        if let Some(re_type) = v.as_symbol() {
            return match re_type {
                "re.all" => self.parse_re_all(),
                "re.none" => self.parse_re_none(),
                "re.allchar" => self.parse_re_allchar(),
                _ => Err(SmtParseError::unrecog(v)),
            };
        }
        // Handles recursive case

        let (re_type, args) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;

        if let Some((head, tail)) = re_type.as_pair() {
            return match head.as_symbol().ok_or(SmtParseError::unrecog(v))? {
                "_" => self.parse_re_func(tail, args),
                _ => Err(SmtParseError::unrecog(v)),
            };
        }
        // Handles recursive regex
        match re_type.as_symbol().ok_or(SmtParseError::unrecog(v))? {
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
        }
    }

    fn parse_re_union(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (re.union R1 R2 ...)
        let args = SmtParser::get_args(v)?;
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
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        Ok(GenRegex::intersect(
            &self.parse_regex(regex1)?,
            &GenRegex::complement(&self.parse_regex(regex2)?),
        ))
    }

    fn parse_re_inter(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (re.inter R1 R2 ...)
        let args = SmtParser::get_args(v)?;
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

    fn parse_re_concat(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        // Syntax: (re.++ R1 R2 ...)
        let args = SmtParser::get_args(v)?;
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
        println!("{}, 2{}, tail {}", char1, char2, tail);
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
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
                if !tail.is_null() {
                    return Err(SmtParseError::unrecog(func_params));
                }
                let (regex, tail) = args.as_pair().ok_or(SmtParseError::unrecog(args))?;
                if !tail.is_null() {
                    return Err(SmtParseError::unrecog(tail));
                }
                self.parse_re_loop(param1_val, param2_val, regex)
            }
            "char" => {
                println!("what the heckles"); // Lol
                todo!();
            }
            "re.^" => {
                let (param1_val, tail) = func_params
                    .as_pair()
                    .ok_or(SmtParseError::unrecog(func_params))?;
                if !tail.is_null() {
                    return Err(SmtParseError::unrecog(func_params));
                }
                let (regex, tail) = args.as_pair().ok_or(SmtParseError::unrecog(args))?;
                if !tail.is_null() {
                    return Err(SmtParseError::unrecog(tail));
                }
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
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        Ok(GenRegex::complement(&self.parse_regex(regex)?))
    }

    pub fn parse_re_opt(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        let (regex, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        Ok(GenRegex::union(
            &self.parse_regex(regex)?,
            &GenRegex::create_gre_char_lit(""),
        ))
    }

    pub fn parse_re_plus(&mut self, v: &Value) -> Result<Rc<GenRegex>, SmtParseError> {
        let (regex, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
        let regex = self.parse_regex(regex)?;
        Ok(GenRegex::concat(&regex, &GenRegex::star(&regex)))
    }

    /*
        Parsing functions with output String/Char
     */

    fn parse_char_obj(&self, v: &Value) -> Result<char, SmtParseError> {
        println!("char_obj: {:?}", v);
        if v.is_string() {
            if v.as_str().unwrap().chars().count() == 1 {
                return Ok(v.as_str().unwrap().chars().next().expect("ERROR"));
            } else {
                return Err(SmtParseError::unrecog(v));
            }
        } else if v.is_cons() {
            // Removes underscore
            let (_underscore, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
            let (_, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
            let (hex, _tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
            if hex.is_number() {
                return Ok(hex_to_char(hex.as_u64().expect("ERROR")));
            }
            // TODO
        }

        todo!()
    }

    fn parse_str_at(&self, v: &Value) -> Result<String, SmtParseError> {
        let (string, tail) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;
        let (index, tail) = tail.as_pair().ok_or(SmtParseError::unrecog(v))?;
        if !tail.is_null() {
            return Err(SmtParseError::unrecog(v));
        }
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

    fn parse_str_func(&mut self, v: &Value) -> Result<String, SmtParseError> {
        let (str_type, args) = v.as_pair().ok_or(SmtParseError::unrecog(v))?;

        // Handles recursive regex
        match str_type.as_symbol().ok_or(SmtParseError::unrecog(v))? {
            "str.at" => self.parse_str_at(args),
            _ => Err(SmtParseError::unrecog(str_type)),
        }
    }



    /*
        Helper Functions
     */

    fn get_args(v: &Value) -> Result<Vec<&Value>, SmtParseError> {
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
        let result = satisfiable(&Rc::new(gen_regex_unwrapped));
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
        let expected = GenRegex::Intersect(
            GenRegex::create_gre_str_var("x"),
            GenRegex::re_range(&hex_to_char(0x0), &'/'),
        );

        assert_eq!(gen_regex_unwrapped, expected);
        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), true);
    }

    #[test]
    fn unicode_hex_test() {
        let smt_result = parse_smtlib_file("benchmarks/hex_syntax_test_sat.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
        let gen_regex_unwrapped = gen_regex.unwrap();
        assert_eq!(satisfiable(&Rc::new(gen_regex_unwrapped.clone())), true);
    }

    #[ignore]
    #[test]
    fn intersect_test1() {
        let smt_result = parse_smtlib_file("benchmarks/intersect_0_0_sat.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
    }

    #[test]
    fn parse_exp(){
        let smt_result = parse_smtlib_file("benchmarks/reglan_var_test.smt2");
        println!("Parsed s-expression: {:?}", smt_result);

        assert!(smt_result.is_ok());
        let s_expr = smt_result.unwrap();

        // Parse the s-expression as a GenRegex
        let mut parser = SmtParser::new();
        let gen_regex = parser.parse_s_expr(&s_expr);
        println!("Parsed GenRegex: {:?}", gen_regex);

        assert!(gen_regex.is_ok());
    }

    #[test]
    fn test_let_1() {
        assert_satisfiable("benchmarks/simple_let_sat_1.smt2");
    }

    #[ignore]
    #[test]
    fn test_let_2() {
        assert_satisfiable("benchmarks/simple_let_sat_2.smt2");
    }

    #[test]
    fn test_let_3() {
        assert_satisfiable("benchmarks/date_format_days_sat.smt2");
    }
}
