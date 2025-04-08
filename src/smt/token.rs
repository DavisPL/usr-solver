use super::parse2::SmtParseError;
use crate::types::regex::GenRegex;
use smt2parser::concrete::{Constant, Term};
use std::rc::Rc;

pub enum Token {
    Val(Constant),
    Var(String),
    RegLanTok(RegexToken),
    StringTok(StringToken),
}
impl Token {
    pub fn as_string_tok(&self) -> Result<StringToken, SmtParseError> {
        match self {
            Token::Val(constant) => {
                let Constant::String(val) = constant else {
                    return Err(SmtParseError::Unexpected(
                        format!("Non-string constant"),
                        format!("String constant"),
                    ));
                };
                Ok(StringToken::Val(val.to_string()))
            }
            Token::Var(var_name) => Ok(StringToken::Var(var_name.to_string())),
            Token::StringTok(string_token) => Ok(string_token.clone()),
            _ => Err(SmtParseError::Unexpected(
                format!("Token unconvertable to StringToken"),
                format!("Convertable token"),
            )),
        }
    }
    pub fn as_re_tok(&self) -> Result<RegexToken, SmtParseError> {
        match self {
            Token::Var(var_name) => match var_name.as_str() {
                "re.all" => Ok(RegexToken::Val(GenRegex::sigma_star())),
                "re.none" => Ok(RegexToken::Val(GenRegex::empty_set())),
                "re.allchar" => Ok(RegexToken::Val(GenRegex::create_sigma())),
                _ => Ok(RegexToken::Var(var_name.to_string())),
            },
            Token::RegLanTok(regex_token) => Ok(regex_token.clone()),
            _ => Err(SmtParseError::Unexpected(
                format!("Token unconvertable to RegLanToken"),
                format!("Convertable token"),
            )),
        }
    }
}
#[derive(Clone)]
pub enum StringToken {
    Val(String),
    Var(String),
    Conditional {
        assertion: Rc<Term>,
        true_string: Rc<StringToken>,
        false_string: Rc<StringToken>,
    },
}
#[derive(Clone)]
pub enum RegexToken {
    Val(Rc<GenRegex>),
    Var(String),
    Conditional {
        assertion: Rc<Term>,
        true_re: Rc<RegexToken>,
        false_re: Rc<RegexToken>,
    },
}
impl RegexToken {
    pub fn create_val(re: &Rc<GenRegex>) -> RegexToken {
        RegexToken::Val(re.clone())
    }
    pub fn create_var(var_name: &str) -> RegexToken {
        RegexToken::Var(var_name.to_string())
    }
    pub fn diff(tok1: &RegexToken, tok2: &RegexToken) -> Result<RegexToken, SmtParseError> {
        match tok1 {
            RegexToken::Val(gen_regex1) => match tok2 {
                RegexToken::Val(gen_regex2) => {
                    Ok(RegexToken::Val(GenRegex::diff(gen_regex1, gen_regex2)))
                }
                RegexToken::Conditional {
                    assertion,
                    true_re,
                    false_re,
                } => {
                    let true_re = Rc::new(RegexToken::diff(tok1, true_re)?);
                    let false_re = Rc::new(RegexToken::diff(tok1, false_re)?);
                    Ok(RegexToken::Conditional {
                        assertion: assertion.clone(),
                        true_re,
                        false_re,
                    })
                }
                RegexToken::Var(_) => Err(SmtParseError::Unsupported(format!(
                    "RegLan operations not supported with variables."
                ))),
            },
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                let true_re = Rc::new(RegexToken::diff(true_re, tok2)?);
                let false_re = Rc::new(RegexToken::diff(false_re, tok2)?);
                Ok(RegexToken::Conditional {
                    assertion: assertion.clone(),
                    true_re,
                    false_re,
                })
            }
            RegexToken::Var(_) => Err(SmtParseError::Unsupported(format!(
                "RegLan operations not supported with variables."
            ))),
        }
    }
    pub fn concat(tok1: &RegexToken, tok2: &RegexToken) -> Result<RegexToken, SmtParseError> {
        match tok1 {
            RegexToken::Val(gen_regex1) => match tok2 {
                RegexToken::Val(gen_regex2) => {
                    Ok(RegexToken::Val(GenRegex::concat(gen_regex1, gen_regex2)))
                }
                RegexToken::Conditional {
                    assertion,
                    true_re,
                    false_re,
                } => {
                    let true_re = Rc::new(RegexToken::concat(tok1, true_re)?);
                    let false_re = Rc::new(RegexToken::concat(tok1, false_re)?);
                    Ok(RegexToken::Conditional {
                        assertion: assertion.clone(),
                        true_re,
                        false_re,
                    })
                }
                RegexToken::Var(_) => Err(SmtParseError::Unsupported(format!(
                    "RegLan operations not supported with variables."
                ))),
            },
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                let true_re = Rc::new(RegexToken::concat(true_re, tok2)?);
                let false_re = Rc::new(RegexToken::concat(false_re, tok2)?);
                Ok(RegexToken::Conditional {
                    assertion: assertion.clone(),
                    true_re,
                    false_re,
                })
            }
            RegexToken::Var(_) => Err(SmtParseError::Unsupported(format!(
                "RegLan operations not supported with variables."
            ))),
        }
    }
    pub fn union(tok1: &RegexToken, tok2: &RegexToken) -> Result<RegexToken, SmtParseError> {
        match tok1 {
            RegexToken::Val(gen_regex1) => match tok2 {
                RegexToken::Val(gen_regex2) => {
                    Ok(RegexToken::Val(GenRegex::union(gen_regex1, gen_regex2)))
                }
                RegexToken::Conditional {
                    assertion,
                    true_re,
                    false_re,
                } => {
                    let true_re = Rc::new(RegexToken::union(tok1, true_re)?);
                    let false_re = Rc::new(RegexToken::union(tok1, false_re)?);
                    Ok(RegexToken::Conditional {
                        assertion: assertion.clone(),
                        true_re,
                        false_re,
                    })
                }
                RegexToken::Var(_) => Err(SmtParseError::Unsupported(format!(
                    "RegLan operations not supported with variables."
                ))),
            },
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                let true_re = Rc::new(RegexToken::union(true_re, tok2)?);
                let false_re = Rc::new(RegexToken::union(false_re, tok2)?);
                Ok(RegexToken::Conditional {
                    assertion: assertion.clone(),
                    true_re,
                    false_re,
                })
            }
            RegexToken::Var(_) => Err(SmtParseError::Unsupported(format!(
                "RegLan operations not supported with variables."
            ))),
        }
    }
    pub fn inter(tok1: &RegexToken, tok2: &RegexToken) -> Result<RegexToken, SmtParseError> {
        match tok1 {
            RegexToken::Val(gen_regex1) => match tok2 {
                RegexToken::Val(gen_regex2) => {
                    Ok(RegexToken::Val(GenRegex::intersect(gen_regex1, gen_regex2)))
                }
                RegexToken::Conditional {
                    assertion,
                    true_re,
                    false_re,
                } => {
                    let true_re = Rc::new(RegexToken::inter(tok1, true_re)?);
                    let false_re = Rc::new(RegexToken::inter(tok1, false_re)?);
                    Ok(RegexToken::Conditional {
                        assertion: assertion.clone(),
                        true_re,
                        false_re,
                    })
                }
                RegexToken::Var(_) => Err(SmtParseError::Unsupported(format!(
                    "RegLan operations not supported with variables."
                ))),
            },
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                let true_re = Rc::new(RegexToken::inter(true_re, tok2)?);
                let false_re = Rc::new(RegexToken::inter(false_re, tok2)?);
                Ok(RegexToken::Conditional {
                    assertion: assertion.clone(),
                    true_re,
                    false_re,
                })
            }
            RegexToken::Var(_) => Err(SmtParseError::Unsupported(format!(
                "RegLan operations not supported with variables."
            ))),
        }
    }
    pub fn caret(num: u64, tok: &RegexToken) -> Result<RegexToken, SmtParseError> {
        match tok {
            RegexToken::Val(gen_regex) => Ok(RegexToken::Val(GenRegex::caret(num, gen_regex))),
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                let true_re = Rc::new(RegexToken::caret(num, true_re)?);
                let false_re = Rc::new(RegexToken::caret(num, false_re)?);
                Ok(RegexToken::Conditional {
                    assertion: assertion.clone(),
                    true_re,
                    false_re,
                })
            }
            RegexToken::Var(_) => todo!(),
        }
    }
    pub fn tok_loop(num1: u64, num2: u64, tok: &RegexToken) -> Result<RegexToken, SmtParseError> {
        match tok {
            RegexToken::Val(gen_regex) => {
                Ok(RegexToken::Val(GenRegex::re_loop(num1, num2, gen_regex)))
            }
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                let true_re = Rc::new(RegexToken::tok_loop(num1, num2, true_re)?);
                let false_re = Rc::new(RegexToken::tok_loop(num1, num2, false_re)?);
                Ok(RegexToken::Conditional {
                    assertion: assertion.clone(),
                    true_re,
                    false_re,
                })
            }
            RegexToken::Var(_) => todo!(),
        }
    }
    pub fn star(tok: &RegexToken) -> Result<RegexToken, SmtParseError> {
        match tok {
            RegexToken::Val(gen_regex) => Ok(RegexToken::Val(GenRegex::star(gen_regex))),
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                let assertion = assertion.clone();
                let true_re = Rc::new(RegexToken::star(true_re)?);
                let false_re = Rc::new(RegexToken::star(false_re)?);
                Ok(RegexToken::Conditional {
                    assertion,
                    true_re,
                    false_re,
                })
            }
            RegexToken::Var(_) => todo!(),
        }
    }
    pub fn plus(tok: &RegexToken) -> Result<RegexToken, SmtParseError> {
        match tok {
            RegexToken::Val(gen_regex) => Ok(RegexToken::Val(GenRegex::concat(
                gen_regex,
                &GenRegex::star(gen_regex),
            ))),
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                let assertion = assertion.clone();
                let true_re = Rc::new(RegexToken::plus(true_re)?);
                let false_re = Rc::new(RegexToken::plus(false_re)?);
                Ok(RegexToken::Conditional {
                    assertion,
                    true_re,
                    false_re,
                })
            }
            RegexToken::Var(_) => todo!(),
        }
    }
    pub fn comp(tok: &RegexToken) -> Result<RegexToken, SmtParseError> {
        match tok {
            RegexToken::Val(gen_regex) => Ok(RegexToken::Val(GenRegex::complement(gen_regex))),
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                let assertion = assertion.clone();
                let true_re = Rc::new(RegexToken::comp(true_re)?);
                let false_re = Rc::new(RegexToken::comp(false_re)?);
                Ok(RegexToken::Conditional {
                    assertion,
                    true_re,
                    false_re,
                })
            }
            RegexToken::Var(_) => todo!(),
        }
    }
    pub fn opt(tok: &RegexToken) -> Result<RegexToken, SmtParseError> {
        match tok {
            RegexToken::Val(gen_regex) => Ok(RegexToken::Val(GenRegex::union(
                gen_regex,
                &GenRegex::epsilon(),
            ))),
            RegexToken::Conditional {
                assertion,
                true_re,
                false_re,
            } => {
                let assertion = assertion.clone();
                let true_re = Rc::new(RegexToken::opt(true_re)?);
                let false_re = Rc::new(RegexToken::opt(false_re)?);
                Ok(RegexToken::Conditional {
                    assertion,
                    true_re,
                    false_re,
                })
            }
            RegexToken::Var(_) => todo!(),
        }
    }
}
