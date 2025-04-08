use smt2parser::concrete::{Command, Constant, QualIdentifier, Sort, Symbol, Term};
use smt2parser::visitors::{Identifier, Index};
use smt2parser::{self, concrete, CommandStream};

use num_traits::cast::ToPrimitive;

use super::token::{RegexToken, StringToken, Token};
use super::util::{hex_to_char, parse_unicode_escape};
use crate::types::regex::GenRegex;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;

#[derive(Debug)]
pub enum SmtParseError {
    FileError(String),          // File not found
    SexprError(String),         // Error parsing S-expression
    MissingAssertion(),         // Missing (assert) statement in SMTLib file
    BadCheckSat(),              // Bad or missing (check-sat) statement in SMTLib file
    Unsupported(String),        // Unsupported SMTLib feature
    Unrecognized(String),       // Unrecognized SMTLib feature
    Unimplemented(String),      // Unimplemented SMTLib feature
    BadLiteral(String),         // Bad literal in SMTLib file
    Unexpected(String, String), // Unexpected S-expression
}

impl From<smt2parser::concrete::Error> for SmtParseError {
    fn from(e: smt2parser::concrete::Error) -> Self {
        match e {
            concrete::Error::SyntaxError(position, val) => SmtParseError::SexprError(format!(
                "Invalid syntax at pos:{} with val:{}",
                position, val
            )),
            concrete::Error::ParsingError(position, val) => SmtParseError::SexprError(format!(
                "Unable to parse at pos:{} with val:{}",
                position, val
            )),
        }
    }
}

impl From<std::io::Error> for SmtParseError {
    fn from(e: std::io::Error) -> Self {
        SmtParseError::FileError(e.to_string())
    }
}

pub fn parse_smtlib_file(file_path: &str) -> Result<Rc<GenRegex>, SmtParseError> {
    // Read in the file
    let smt_string = std::fs::read_to_string(file_path)?;

    // Parse S-expression
    let stream = CommandStream::new(smt_string.as_bytes(), concrete::SyntaxBuilder, None);
    let commands = stream.collect::<Result<Vec<_>, _>>()?;
    for cmd in &commands {
        println!("{:?}", cmd);
    }
    let mut parser = SmtParser::new();
    parser.parse(commands)
}

pub struct SmtParser {
    found_assert: bool,
    found_check_sat: bool,
    sort_types: HashMap<String, HashSet<String>>,
    func_names: HashMap<String, String>,
    regex_result: Option<Rc<GenRegex>>,
    not_flag: bool,
}

impl SmtParser {
    pub fn new() -> Self {
        let mut rv = Self {
            found_assert: false,
            found_check_sat: false,
            sort_types: HashMap::new(),
            func_names: HashMap::new(),
            regex_result: None,
            not_flag: false,
        };
        rv.sort_types.insert("String".to_string(), HashSet::new());
        rv.sort_types.insert("RegLan".to_string(), HashSet::new());
        rv.sort_types.insert("Int".to_string(), HashSet::new());
        rv
    }

    pub fn parse(&mut self, cmd_list: Vec<Command>) -> Result<Rc<GenRegex>, SmtParseError> {
        for cmd in cmd_list {
            self.parse_cmd(cmd)?;
        }
        if !self.found_assert {
            return Err(SmtParseError::MissingAssertion());
        }
        if !self.found_check_sat {
            return Err(SmtParseError::BadCheckSat());
        }
        let result = self.regex_result.take();
        Ok(result.expect("Regex result should have been set by parser earlier!"))
    }

    fn parse_cmd(&mut self, cmd: Command) -> Result<(), SmtParseError> {
        match cmd {
            Command::Assert { term } => self.parse_assert(&term),
            Command::CheckSat => self.parse_check_sat(),
            Command::CheckSatAssuming { literals } => todo!(),
            Command::DeclareConst { symbol, sort } => self.parse_declare_const(&symbol, &sort),
            Command::DeclareDatatype { symbol, datatype } => todo!(),
            Command::DeclareDatatypes { datatypes } => todo!(),
            Command::DeclareFun {
                symbol,
                parameters,
                sort,
            } => todo!(),
            Command::DeclareSort { symbol, arity } => todo!(),
            Command::DefineFun { sig, term } => todo!(),
            Command::DefineFunRec { sig, term } => todo!(),
            Command::DefineFunsRec { funs } => todo!(),
            Command::DefineSort {
                symbol,
                parameters,
                sort,
            } => todo!(),
            Command::Echo { message } => todo!(),
            Command::Exit => todo!(),
            Command::GetAssertions => todo!(),
            Command::GetAssignment => todo!(),
            Command::GetInfo { flag } => todo!(),
            Command::GetModel => todo!(),
            Command::GetOption { keyword } => todo!(),
            Command::GetProof => todo!(),
            Command::GetUnsatAssumptions => todo!(),
            Command::GetUnsatCore => todo!(),
            Command::GetValue { terms } => todo!(),
            Command::Pop { level } => todo!(),
            Command::Push { level } => todo!(),
            Command::Reset => todo!(),
            Command::ResetAssertions => todo!(),
            Command::SetInfo { keyword, value } => todo!(),
            Command::SetLogic { .. } => Ok(()),
            Command::SetOption { keyword, value } => todo!(),
        }
    }

    fn parse_assert(&mut self, term: &Term) -> Result<(), SmtParseError> {
        self.found_assert = true;
        let Token::RegLanTok(RegexToken::Val(re_assertion)) = self.parse_term(term)? else {
            return Err(SmtParseError::SexprError(format!(
                "parse_assert should yield concrete value Regextoken"
            )));
        };
        match &self.regex_result {
            Some(re) => self.regex_result = Some(GenRegex::concat(&re, &re_assertion)),
            None => self.regex_result = Some(re_assertion.clone()),
        };
        Ok(())
    }
    fn parse_check_sat(&mut self) -> Result<(), SmtParseError> {
        self.found_check_sat = true;
        Ok(())
    }
    fn parse_declare_const(&mut self, symbol: &Symbol, sort: &Sort) -> Result<(), SmtParseError> {
        let var_name = self.parse_symbol(symbol)?;
        let sort_name = self.parse_sort(sort)?;
        let sort_vars = self
            .sort_types
            .get_mut(&sort_name)
            .expect("parse_sort() should have checked Sort existence");
        sort_vars.insert(var_name);
        Ok(())
    }
    fn parse_symbol(&self, symbol: &Symbol) -> Result<String, SmtParseError> {
        Ok(symbol.0.clone())
    }
    fn parse_sort(&mut self, sort: &Sort) -> Result<String, SmtParseError> {
        let sort_name = match sort {
            Sort::Simple { identifier } => self.parse_sort_identifier(identifier),
            Sort::Parameterized { .. } => Err(SmtParseError::Unsupported(format!(
                "Parameterized Sorts not supported."
            ))),
        }?;
        if !self.sort_types.contains_key(&sort_name) {
            return Err(SmtParseError::Unrecognized(format!(
                "Sort: {} not found/defined",
                sort_name
            )));
        }
        Ok(sort_name)
    }
    fn parse_sort_identifier(&self, identifier: &Identifier) -> Result<String, SmtParseError> {
        match identifier {
            Identifier::Simple { symbol } => self.parse_symbol(symbol),
            Identifier::Indexed { .. } => Err(SmtParseError::Unsupported(format!(
                "Indexed identifiers for Sort not supported."
            ))),
        }
    }
    fn parse_term(&mut self, term: &Term) -> Result<Token, SmtParseError> {
        match term {
            Term::Constant(constant) => Ok(Token::Val(constant.clone())),
            Term::QualIdentifier(qual_identifier) => {
                let res = self.parse_qual_identifier(qual_identifier)?;
                if !res.1.is_none() {
                    return Err(SmtParseError::Unsupported(format!(
                        "Indexed identifier not supported in argument terms."
                    )));
                }
                Ok(Token::Var(res.0))
            }
            Term::Application {
                qual_identifier,
                arguments,
            } => self.parse_application(qual_identifier, arguments),
            Term::Let { var_bindings, term } => todo!(),
            Term::Forall { .. } => todo!(),
            Term::Exists { .. } => todo!(),
            Term::Match { .. } => todo!(),
            Term::Attributes { .. } => todo!(),
        }
    }
    fn parse_application(
        &mut self,
        qual_identifier: &QualIdentifier,
        arguments: &Vec<Term>,
    ) -> Result<Token, SmtParseError> {
        let func = self.parse_qual_identifier(&qual_identifier)?;
        let res = match func.0.as_str() {
            //Top level functions
            "and"=> {
                if self.not_flag{
                    self.parse_or(arguments)
                }
                else{
                    self.parse_and(arguments)
                }
            },
            "or" => {
                if self.not_flag{
                    self.parse_and(arguments)
                }
                else{
                    self.parse_or(arguments)
                }
            },
            "not"=> self.parse_not(arguments),
            //Str functions
            "str.in_re" => self.parse_str_in_re(arguments),
            "str.to_re" => self.parse_str_to_re(arguments),
            //Re functions
            "re.++" => self.parse_re_concat(arguments),
            "re.union" => self.parse_re_union(arguments),
            "re.diff" => self.parse_re_diff(arguments),
            "re.*" => self.parse_re_star(arguments),
            "re.inter" => self.parse_re_inter(arguments),
            "re.range" => self.parse_re_range(arguments),
            "re.comp" => self.parse_re_comp(arguments),
            "re.+" => self.parse_re_plus(arguments),
            "re.opt" => self.parse_re_opt(arguments),
            "re.^"=>self.parse_re_caret(arguments, func.1),
            _ => {
                println!("Unimplemented/Unsupported: {}",func.0);
                todo!()
            },
        }?;
        Ok(res)
    }
    fn parse_qual_identifier<'a>(
        &self,
        qual_identifier: &'a QualIdentifier,
    ) -> Result<(String, Option<&'a Vec<Index>>), SmtParseError> {
        match qual_identifier {
            QualIdentifier::Simple { identifier } => self.parse_term_identifier(identifier),
            QualIdentifier::Sorted { .. } => Err(SmtParseError::Unsupported(format!(
                "Sorted identifiers in Term not supported."
            ))),
        }
    }
    fn parse_term_identifier<'a>(
        &self,
        identifier: &'a Identifier,
    ) -> Result<(String, Option<&'a Vec<Index>>), SmtParseError> {
        match identifier {
            Identifier::Simple { symbol } => Ok((self.parse_symbol(symbol)?, None)),
            //Indexed would be like (_ re.loop 3)
            Identifier::Indexed { symbol, indices } => {
                Ok((self.parse_symbol(symbol)?, Some(indices)))
            }
        }
    }
    fn parse_and(&mut self ,args: &Vec<Term>)->Result<Token,SmtParseError>{
        if args.len() < 2 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in \"and\"", args.len()),
                format!(">=2 args"),
            ));
        }
        let mut re_args: Vec<RegexToken> = Vec::new();
        for term in args {
            let tok = self.parse_term(term)?;
            let re_tok = tok.as_re_tok()?;
            re_args.push(re_tok);
        }
        let mut cur = re_args.pop().unwrap();
        while let Some(next) = re_args.pop() {
            cur = RegexToken::concat(&next, &cur).unwrap();
        }
        Ok(Token::RegLanTok(cur))
    }
    fn parse_or(&mut self ,args: &Vec<Term>)->Result<Token,SmtParseError>{
        if args.len() < 2 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in \"or\"", args.len()),
                format!(">=2 args"),
            ));
        }
        let mut re_args: Vec<RegexToken> = Vec::new();
        for term in args {
            let tok = self.parse_term(term)?;
            let re_tok = tok.as_re_tok()?;
            re_args.push(re_tok);
        }
        let mut cur = re_args.pop().unwrap();
        while let Some(next) = re_args.pop() {
            cur = RegexToken::union(&next, &cur).unwrap();
        }
        Ok(Token::RegLanTok(cur))
    }
    fn parse_not(&mut self,args: &Vec<Term>)->Result<Token, SmtParseError>{
        if args.len() != 1 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in \"not\"", args.len()),
                format!("1 arg"),
            ));
        }
        self.not_flag=!self.not_flag;
        let res=self.parse_term(&args[0])?;
        self.not_flag=!self.not_flag;
        Ok(res)
    }
    fn parse_str_in_re(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() != 2 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in str.in_re.", args.len()),
                format!("3 args"),
            ));
        }
        let tok1 = self.parse_term(&args[0])?;
        let tok2 = self.parse_term(&args[1])?;
        let membership_str = tok1.as_string_tok()?;
        let asserted_re = tok2.as_re_tok()?;
        self.parse_str_in_re_helper(&asserted_re, &membership_str)
    }
    fn parse_str_in_re_helper(
        &mut self,
        re_tok: &RegexToken,
        string: &StringToken,
    ) -> Result<Token, SmtParseError> {
        match re_tok {
            RegexToken::Var(_) => Err(SmtParseError::Unsupported(format!(
                "RegLan variable in str.in_re needs to be initialzied beforehand."
            ))),
            RegexToken::Val(gen_regex) => {
                let gen_regex = if self.not_flag {
                    GenRegex::complement(gen_regex)
                } else {
                    gen_regex.clone()
                };
                match string {
                    StringToken::Var(var_name) => {
                        if !self.is_valid_var("String", var_name) {
                            return Err(SmtParseError::SexprError(format!(
                                "{} has not been declared.",
                                var_name
                            )));
                        }
                        let rv = GenRegex::intersect(
                            &GenRegex::create_gre_str_var(var_name),
                            &gen_regex,
                        );
                        Ok(Token::RegLanTok(RegexToken::Val(rv)))
                    }
                    StringToken::Val(string) => {
                        let rv = GenRegex::intersect(&GenRegex::str_to_re(string), &gen_regex);
                        Ok(Token::RegLanTok(RegexToken::Val(rv)))
                    }
                    StringToken::Conditional {
                        assertion,
                        true_string,
                        false_string,
                    } => {
                        todo!()
                    }
                }
            }
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                //Remember to do not_flag stuff
                todo!()
            }
        }
    }
    fn parse_str_to_re(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() != 1 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in str.to_re.", args.len()),
                format!("1 args"),
            ));
        }
        let tok = self.parse_term(&args[0])?;
        let str_tok = tok.as_string_tok()?;
        match str_tok {
            StringToken::Val(val) => {
                let usr = GenRegex::str_to_re(&val);
                Ok(Token::RegLanTok(RegexToken::create_val(&usr)))
            }
            StringToken::Var(var_name) => {
                if !self.is_valid_var("String", &var_name) {
                    return Err(SmtParseError::SexprError(format!(
                        "{} has not been declared.",
                        var_name
                    )));
                }
                let usr = GenRegex::create_gre_str_var(&var_name);
                Ok(Token::RegLanTok(RegexToken::create_val(&usr)))
            }
            StringToken::Conditional {
                assertion,
                true_string,
                false_string,
            } => todo!(),
        }
    }
    fn parse_re_concat(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() < 2 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in re.++.", args.len()),
                format!(">=2 args"),
            ));
        }
        let mut re_args: Vec<RegexToken> = Vec::new();
        for term in args {
            let tok = self.parse_term(term)?;
            let re_tok = tok.as_re_tok()?;
            re_args.push(re_tok);
        }
        let mut cur = re_args.pop().unwrap();
        while let Some(next) = re_args.pop() {
            cur = RegexToken::concat(&next, &cur).unwrap();
        }
        Ok(Token::RegLanTok(cur))
    }
    fn parse_re_union(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() < 2 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in re.union.", args.len()),
                format!(">=2 args"),
            ));
        }
        let mut re_args: Vec<RegexToken> = Vec::new();
        for term in args {
            let tok = self.parse_term(term)?;
            let re_tok = tok.as_re_tok()?;
            re_args.push(re_tok);
        }
        let mut cur = re_args.pop().unwrap();
        while let Some(next) = re_args.pop() {
            cur = RegexToken::union(&next, &cur).unwrap();
        }
        Ok(Token::RegLanTok(cur))
    }
    fn parse_re_diff(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() != 2 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in re.diff.", args.len()),
                format!("2 args"),
            ));
        }
        let tok1 = self.parse_term(&args[0])?;
        let tok2 = self.parse_term(&args[1])?;
        let re_tok1 = tok1.as_re_tok()?;
        let re_tok2 = tok2.as_re_tok()?;
        let res = RegexToken::diff(&re_tok1, &re_tok2)?;
        Ok(Token::RegLanTok(res))
    }
    fn parse_re_star(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() != 1 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in re.diff.", args.len()),
                format!("1 arg"),
            ));
        }
        let tok = self.parse_term(&args[0])?;
        let re_tok = tok.as_re_tok()?;
        let res = RegexToken::star(&re_tok)?;
        Ok(Token::RegLanTok(res))
    }
    fn parse_re_inter(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() < 2 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in re.inter.", args.len()),
                format!(">=2 args"),
            ));
        }
        let mut re_args: Vec<RegexToken> = Vec::new();
        for term in args {
            let tok = self.parse_term(term)?;
            let re_tok = tok.as_re_tok()?;
            re_args.push(re_tok);
        }
        let mut cur = re_args.pop().unwrap();
        while let Some(next) = re_args.pop() {
            cur = RegexToken::inter(&next, &cur).unwrap();
        }
        Ok(Token::RegLanTok(cur))
    }
    fn parse_re_range(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() != 2 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in re.diff.", args.len()),
                format!("2 args"),
            ));
        }
        let tok1 = self.parse_term(&args[0])?;
        let tok2 = self.parse_term(&args[1])?;
        let re_tok1 = tok1.as_string_tok()?;
        let re_tok2 = tok2.as_string_tok()?;
        match (re_tok1, re_tok2) {
            (StringToken::Val(str1), StringToken::Val(str2)) => {
                let mut as_char1 = str1.chars();
                let mut as_char2 = str2.chars();
                let char1 = as_char1.next();
                let char2 = as_char2.next();
                let (Some(char1), Some(char2)) = (char1, char2) else {
                    return Err(SmtParseError::SexprError(format!(
                        "re.range must take strings of length 1."
                    )));
                };
                if as_char1.next().is_some() || as_char2.next().is_some() {
                    Err(SmtParseError::SexprError(format!(
                        "re.range must take strings of length 1."
                    )))
                } else {
                    let res = RegexToken::Val(GenRegex::re_range(char1, char2));
                    Ok(Token::RegLanTok(res))
                }
            }
            _ => Err(SmtParseError::Unsupported(format!(
                "re.range only supports concrete characters"
            ))),
        }
    }
    fn parse_re_comp(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() != 1 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in re.comp.", args.len()),
                format!("1 arg"),
            ));
        }
        let tok = self.parse_term(&args[0])?;
        let re_tok = tok.as_re_tok()?;
        let res = RegexToken::comp(&re_tok)?;
        Ok(Token::RegLanTok(res))
    }
    fn parse_re_plus(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() != 1 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in re.plus.", args.len()),
                format!("1 arg"),
            ));
        }
        let tok = self.parse_term(&args[0])?;
        let re_tok = tok.as_re_tok()?;
        let res = RegexToken::plus(&re_tok)?;
        Ok(Token::RegLanTok(res))
    }
    fn parse_re_opt(&mut self, args: &Vec<Term>) -> Result<Token, SmtParseError> {
        if args.len() != 1 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in re.opt.", args.len()),
                format!("1 arg"),
            ));
        }
        let tok = self.parse_term(&args[0])?;
        let re_tok = tok.as_re_tok()?;
        let res = RegexToken::opt(&re_tok)?;
        Ok(Token::RegLanTok(res))
    }
    fn parse_re_caret(&mut self, args: &Vec<Term>, index:Option<&Vec<Index>>)-> Result<Token,SmtParseError>{
        if args.len() != 1 {
            return Err(SmtParseError::Unexpected(
                format!("{} args in re.opt.", args.len()),
                format!("1 arg"),
            ));
        }
        let Some(index)=index else{
            return Err(SmtParseError::SexprError(format!("re.^ should have an index.")));
        };
        if index.len()!=1{
            return Err(SmtParseError::Unexpected(
                format!("{} indicies in re.^.", args.len()),
                format!("1 index"),
            ));
        }
        let Index::Numeral(ref num)=index[0] else{
            return Err(SmtParseError::Unexpected(
                format!("{} in re.^.", args.len()),
                format!("numeral index"),
            ));
        };
        let tok = self.parse_term(&args[0])?;
        let re_tok = tok.as_re_tok()?;
        // TODO: Does not support BigUInt, current just casts to u64, panics if too big
        let res=RegexToken::caret(num.to_u64().unwrap(), &re_tok)?;
        Ok(Token::RegLanTok(res))
    }
    fn is_valid_var(&self, sort_type: &str, var_name: &String) -> bool {
        let Some(var_list) = self.sort_types.get(sort_type) else {
            return false;
        };
        if var_list.contains(var_name) {
            return true;
        }
        false
    }
}
