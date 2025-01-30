//!
//! Display implementations for pretty printing
//!

use crate::classes::{
    AntimirovElement, AnySub, CharExpression, CharVar, GenRegex, MaybeCharExpression, Predicate,
    RangeConstr, SimpleSub, StringIndex, StringVar, SubExpr,
};
use std::fmt::{self, Display};

impl Display for SubExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for val in self.get_head() {
            write!(f, "{}", val)?;
        }
        write!(f, "{}", self.get_tail())
    }
}
impl Display for AnySub {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ANYSUB string_to: {{ ")?;
        for (key, value) in self.get_str_map() {
            for v in value {
                write!(f, "{} => {}, ", key, v)?;
            }
        }
        writeln!(f, "}}")?;

        write!(f, "char_to: {{ ")?;
        for (key, value) in self.get_char_map() {
            for v in value {
                write!(f, "{} => {}, ", key, v)?;
            }
        }
        write!(f, "}}")?;

        Ok(())
    }
}

impl Display for SimpleSub {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "string_to: {{ ")?;
        for (key, value) in self.get_str_map() {
            write!(f, "{} => {}, ", key, value)?;
        }
        writeln!(f, "}}")?;

        write!(f, "char_to: {{ ")?;
        for (key, value) in self.get_char_map() {
            write!(f, "{} => {}, ", key, value)?;
        }
        write!(f, "}}")?;

        Ok(())
    }
}

impl Display for RangeConstr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}]", self.get_start(), self.get_end())
    }
}

impl Display for AntimirovElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}", self.get_expr(), self.get_subs())?;
        for (var, range) in self.get_ranges() {
            write!(f, ", {}: {}", var, range)?;
        }
        write!(f, ")")
    }
}

impl Display for CharVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CHAR({})", self.name)
    }
}

impl Display for CharExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CharExpression::CharVar(name) => {
                write!(f, "{}", name)
            }
            CharExpression::Literal(name) => {
                write!(f, "{}", name)
            }
        }
    }
}

impl Display for MaybeCharExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaybeCharExpression::CharExpression(name) => {
                write!(f, "{}", name)
            }
            MaybeCharExpression::StringIndex(name) => {
                write!(f, "{}", name)
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
            Predicate::And(left, right) => {
                //let parts: Vec<String> = kids.iter().map(|p| format!("{}", p)).collect();
                write!(f, "({} v {})", left, right)
            }
            Predicate::Or(left, right) => {
                //let parts: Vec<String> = kids.iter().map(|p| format!("{}", p)).collect();
                write!(f, "({} ^ {})", left, right)
                //write!(f, "({})", parts.join(" OR "))
            }
            Predicate::Not(pred1) => {
                write!(f, "!({})", pred1)
            }
            Predicate::True => write!(f, "TRUE"),
            Predicate::False => write!(f, "FALSE"),
            Predicate::Equals(var1, var2) => {
                write!(f, "{} == {}", var1, var2)
            }
            Predicate::EqualLength(var, inte) => {
                write!(f, "|{}| == {}", var, inte)
            }
            Predicate::LessThan(var, val) =>{
                write!(f, "{} <= {}", var, val)
            }
            Predicate::GreaterThan(var, val) =>{
                write!(f, "{} >= {}", var, val)
            }
        }
    }
}

impl Display for GenRegex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", GenRegexPrintHelper::new(self))
    }
}

/*
    Helper struct to pretty-print GenRegexes
*/

struct GenRegexPrintHelper<'a> {
    g: &'a GenRegex,
    concat_ok: bool,
    union_ok: bool,
    inter_ok: bool,
    if_ok: bool,
}

impl<'a> GenRegexPrintHelper<'a> {
    fn new(g: &'a GenRegex) -> Self {
        Self {
            g,
            concat_ok: true,
            union_ok: true,
            inter_ok: true,
            if_ok: true,
        }
    }
    fn new_concat(g: &'a GenRegex) -> Self {
        Self {
            g,
            concat_ok: true,
            union_ok: false,
            inter_ok: false,
            if_ok: false,
        }
    }
    fn new_union(g: &'a GenRegex) -> Self {
        Self {
            g,
            concat_ok: true,
            union_ok: true,
            inter_ok: false,
            if_ok: true,
        }
    }
    fn new_inter(g: &'a GenRegex) -> Self {
        Self {
            g,
            concat_ok: true,
            union_ok: false,
            inter_ok: true,
            if_ok: true,
        }
    }
}

impl Display for GenRegexPrintHelper<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.g {
            // Base cases
            GenRegex::EmptySet => {
                write!(f, "∅")
            }
            GenRegex::Epsilon => {
                write!(f, "ε")
            }
            GenRegex::Sigma => {
                write!(f, ".")
            }
            GenRegex::Range(char1, char2) => {
                write!(f, "[{}, {}]", char1, char2)
            }
            GenRegex::StringVar(var) => {
                // Use Display on StringVar
                write!(f, "{}", var)
            }
            GenRegex::CharExpression(char_expr) => {
                // Use Display on CharExpression
                write!(f, "{}", char_expr)
            }
            GenRegex::StringIndex(string_index) => {
                // Use Display on StringIndex
                write!(f, "{}", string_index)
            }
            GenRegex::StringSlice(var, index) => {
                write!(f, "{}[{}:]", var, index)
            }

            // Inductive cases
            GenRegex::Union(gre1, gre2) => {
                let h1 = GenRegexPrintHelper::new_union(gre1);
                let h2 = GenRegexPrintHelper::new_union(gre2);
                if self.union_ok {
                    write!(f, "{} | {}", h1, h2)
                } else {
                    write!(f, "({} | {})", h1, h2)
                }
            }
            GenRegex::Intersect(gre1, gre2) => {
                let h1 = GenRegexPrintHelper::new_inter(gre1);
                let h2 = GenRegexPrintHelper::new_inter(gre2);
                if self.inter_ok {
                    write!(f, "{} & {}", h1, h2)
                } else {
                    write!(f, "({} & {})", h1, h2)
                }
            }
            GenRegex::Concatenation(gre1, gre2) => {
                let h1 = GenRegexPrintHelper::new_concat(gre1);
                let h2 = GenRegexPrintHelper::new_concat(gre2);
                if self.concat_ok {
                    write!(f, "{}{}", h1, h2)
                } else {
                    write!(f, "({}{})", h1, h2)
                }
            }
            GenRegex::Kleene(gre1) => {
                let h = GenRegexPrintHelper::new(gre1);
                write!(f, "({})*", h)
            }
            GenRegex::Complement(gre1) => {
                let h = GenRegexPrintHelper::new(gre1);
                write!(f, "({})^c", h)
            }
            GenRegex::IfThenElse(pred, gre1, gre2) => {
                let h1 = GenRegexPrintHelper::new(gre1);
                let h2 = GenRegexPrintHelper::new(gre2);
                if self.if_ok {
                    write!(f, "IF({}, {}, {})", pred, h1, h2)
                } else {
                    write!(f, "(IF({}, {}, {}))", pred, h1, h2)
                }
            }
        }
    }
}
