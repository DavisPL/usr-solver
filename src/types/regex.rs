//!
//! Generalized regular expressions
//! (USRs)
//!

use super::expr::{CharExpression, CharVar, StringIndex, StringVar};
use super::predicate::Predicate;

use std::cmp::Ordering;
use std::rc::Rc;

// TODO: add a GenRegex::StringLiteral
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GenRegex {
    EmptySet,
    Epsilon,
    Sigma,
    SigmaStar,
    Range(char, char),
    CharExpression(CharExpression),
    StringVar(StringVar),
    StringSlice(StringVar, i32),
    Union(Rc<GenRegex>, Rc<GenRegex>),
    Intersect(Rc<GenRegex>, Rc<GenRegex>),
    Concatenation(Rc<GenRegex>, Rc<GenRegex>),
    Kleene(Rc<GenRegex>),
    Complement(Rc<GenRegex>),
    IfThenElse(Rc<Predicate>, Rc<GenRegex>, Rc<GenRegex>),
    StringIndex(StringIndex),
}

impl GenRegex {
    pub fn create_sigma() -> Rc<GenRegex> {
        Rc::new(GenRegex::Sigma)
    }
    pub fn sigma_star() -> Rc<GenRegex> {
        Rc::new(GenRegex::SigmaStar)
    }
    pub fn epsilon() -> Rc<GenRegex> {
        Rc::new(GenRegex::Epsilon)
    }
    pub fn create_gre_char_lit(lit: char) -> Rc<GenRegex> {
        let lit = CharExpression::Literal(lit);
        Rc::new(GenRegex::CharExpression(lit))
    }
    pub fn create_gre_char_var(var_name: &str) -> Rc<GenRegex> {
        let char_var = CharExpression::CharVar(CharVar {
            name: var_name.to_string(),
        });
        Rc::new(GenRegex::CharExpression(char_var))
    }
    pub fn create_gre_str_var(var_name: &str) -> Rc<GenRegex> {
        let str_var = StringVar {
            name: var_name.to_string(),
        };
        Rc::new(GenRegex::StringVar(str_var))
    }
    pub fn concat(gre1: &Rc<GenRegex>, gre2: &Rc<GenRegex>) -> Rc<GenRegex> {
        Rc::new(GenRegex::Concatenation(gre1.clone(), gre2.clone()))
    }
    pub fn intersect(gre1: &Rc<GenRegex>, gre2: &Rc<GenRegex>) -> Rc<GenRegex> {
        Rc::new(GenRegex::Intersect(gre1.clone(), gre2.clone()))
    }
    pub fn union(gre1: &Rc<GenRegex>, gre2: &Rc<GenRegex>) -> Rc<GenRegex> {
        Rc::new(GenRegex::Union(gre1.clone(), gre2.clone()))
    }
    pub fn complement(gre: &Rc<GenRegex>) -> Rc<GenRegex> {
        Rc::new(GenRegex::Complement(gre.clone()))
    }
    pub fn if_then_else(
        pred: &Rc<Predicate>,
        gre1: &Rc<GenRegex>,
        gre2: &Rc<GenRegex>,
    ) -> Rc<GenRegex> {
        Rc::new(GenRegex::IfThenElse(
            pred.clone(),
            gre1.clone(),
            gre2.clone(),
        ))
    }
    pub fn union_many(args: &[Rc<GenRegex>]) -> Rc<GenRegex> {
        if args.is_empty() {
            GenRegex::empty_set()
        } else if args.len() == 1 {
            args[0].clone()
        } else {
            GenRegex::union(&args[0], &GenRegex::union_many(&args[1..]))
        }
    }

    pub fn intersect_many(args: &[Rc<GenRegex>]) -> Rc<GenRegex> {
        if args.is_empty() {
            GenRegex::empty_set()
        } else if args.len() == 1 {
            args[0].clone()
        } else {
            GenRegex::intersect(&args[0], &GenRegex::intersect_many(&args[1..]))
        }
    }

    pub fn concat_many(args: &[Rc<GenRegex>]) -> Rc<GenRegex> {
        if args.is_empty() {
            GenRegex::empty_set()
        } else if args.len() == 1 {
            args[0].clone()
        } else {
            GenRegex::concat(&args[0], &GenRegex::concat_many(&args[1..]))
        }
    }
    pub fn diff(gre1: &Rc<GenRegex>, gre2: &Rc<GenRegex>) -> Rc<GenRegex> {
        GenRegex::intersect(gre1, &GenRegex::complement(gre2))
    }
    pub fn star(gre: &Rc<GenRegex>) -> Rc<GenRegex> {
        Rc::new(GenRegex::Kleene(gre.clone()))
    }
    pub fn empty_set() -> Rc<GenRegex> {
        Rc::new(GenRegex::EmptySet)
    }
    pub fn str_to_re(str: &str) -> Rc<GenRegex> {
        //Needs something better than chars() for more support
        let mut char_list = str.chars().rev();
        let Some(c) = char_list.next() else {
            return GenRegex::epsilon();
        };
        let mut retval = GenRegex::create_gre_char_lit(c);
        for c in char_list {
            retval = GenRegex::concat(&GenRegex::create_gre_char_lit(c), &retval);
        }
        retval
    }
    pub fn re_range(start: char, end: char) -> Rc<GenRegex> {
        // WITH RANGE OPTIMIZATION:
        Rc::new(GenRegex::Range(start, end))
        // OLD IMPL:
        // let mut retval = GenRegex::create_gre_char_lit(end);
        // for c in (start..=end).rev().skip(1) {
        //     retval = GenRegex::union(&GenRegex::create_gre_char_lit(c), &retval)
        // }
        // retval
    }
    pub fn caret(n: u64, gre: &Rc<GenRegex>) -> Rc<GenRegex> {
        if n == 0 {
            return GenRegex::epsilon();
        }
        let mut retval = gre.clone();
        for _ in 1..n {
            retval = GenRegex::concat(&gre.clone(), &retval);
        }
        retval
    }
    pub fn re_loop(n1: u64, n2: u64, gre: &Rc<GenRegex>) -> Rc<GenRegex> {
        if n1 > n2 {
            return GenRegex::empty_set();
        }
        let mut retval = GenRegex::caret(n2, gre);
        for i in (n1..n2).rev() {
            retval = GenRegex::union(&GenRegex::caret(i, gre), &retval);
        }
        retval
    }

    /*
        Rewriter constructors

        These could be moved to a rewriter module if expanded.
    */

    pub fn make_concatenation(left: Rc<GenRegex>, right: Rc<GenRegex>) -> Rc<GenRegex> {
        if let GenRegex::Epsilon = left.as_ref() {
            right
        } else if let GenRegex::Epsilon = right.as_ref() {
            left
        } else if let GenRegex::EmptySet = left.as_ref() {
            left
        } else if let GenRegex::EmptySet = right.as_ref() {
            right
        } else {
            Rc::new(GenRegex::Concatenation(left, right))
        }
    }

    pub fn make_union(left: Rc<GenRegex>, right: Rc<GenRegex>) -> Rc<GenRegex> {
        if let GenRegex::EmptySet = left.as_ref() {
            right
        } else if let GenRegex::EmptySet = right.as_ref() {
            left
        } else if let GenRegex::SigmaStar = left.as_ref() {
            left
        } else if let GenRegex::SigmaStar = right.as_ref() {
            right
        } else if left == right {
            left
        } else {
            Rc::new(GenRegex::Union(left, right))
        }
    }

    pub fn make_intersection(left: Rc<GenRegex>, right: Rc<GenRegex>) -> Rc<GenRegex> {
        if let GenRegex::EmptySet = left.as_ref() {
            left
        } else if let GenRegex::EmptySet = right.as_ref() {
            right
        } else if let GenRegex::SigmaStar = left.as_ref() {
            right
        } else if let GenRegex::SigmaStar = right.as_ref() {
            left
        } else if left == right {
            left
        } else {
            Rc::new(GenRegex::Intersect(left, right))
        }
    }

    pub fn make_star(gre: Rc<GenRegex>) -> Rc<GenRegex> {
        if let GenRegex::EmptySet = gre.as_ref() {
            GenRegex::epsilon()
        } else if let GenRegex::Epsilon = gre.as_ref() {
            GenRegex::epsilon()
        } else if let GenRegex::Sigma = gre.as_ref() {
            GenRegex::sigma_star()
        } else if let GenRegex::SigmaStar = gre.as_ref() {
            GenRegex::sigma_star()
        } else if let GenRegex::Kleene(gre) = gre.as_ref() {
            gre.clone()
        } else {
            Rc::new(GenRegex::Kleene(gre))
        }
    }

    pub fn make_equals(gre1: Rc<GenRegex>, gre2: Rc<GenRegex>) -> Rc<GenRegex> {
        let c1 = GenRegex::make_union(gre1.clone(), GenRegex::complement(&gre2.clone()));
        let c2 = GenRegex::make_union(gre2.clone(), GenRegex::complement(&gre1.clone()));
        return GenRegex::make_intersection(c1.clone(), c2.clone());
    }

    /*
        Helper for regex length
    */
    pub fn length(&self) -> usize {
        match self {
            GenRegex::IfThenElse(p, a, b) => p.length() + a.length() + b.length(),
            GenRegex::Complement(inner) | GenRegex::Kleene(inner) => 1 + inner.length(),
            GenRegex::Union(gre1, gre2)
            | GenRegex::Intersect(gre1, gre2)
            | GenRegex::Concatenation(gre1, gre2) => 1 + gre1.length() + gre2.length(),
            GenRegex::EmptySet
            | GenRegex::Epsilon
            | GenRegex::Sigma
            | GenRegex::SigmaStar
            | GenRegex::Range(_, _)
            | GenRegex::CharExpression(_)
            | GenRegex::StringVar(_)
            | GenRegex::StringIndex(_)
            | GenRegex::StringSlice(_, _) => 1,
        }
    }

    /*
        Helper for what the regex contains as a subexpression
    */

    pub fn contains_ite_complement_or_str_index(&self) -> bool {
        match self {
            GenRegex::IfThenElse(_, _, _) => true,
            GenRegex::Complement(_) => true,
            GenRegex::StringIndex(_) => true,
            GenRegex::StringSlice(_, _) => true,
            GenRegex::Union(gre1, gre2) => {
                gre1.contains_ite_complement_or_str_index()
                    || gre2.contains_ite_complement_or_str_index()
            }
            GenRegex::Intersect(gre1, gre2) => {
                gre1.contains_ite_complement_or_str_index()
                    || gre2.contains_ite_complement_or_str_index()
            }
            GenRegex::Concatenation(gre1, gre2) => {
                gre1.contains_ite_complement_or_str_index()
                    || gre2.contains_ite_complement_or_str_index()
            }
            GenRegex::Kleene(gre1) => gre1.contains_ite_complement_or_str_index(),
            GenRegex::EmptySet
            | GenRegex::Epsilon
            | GenRegex::Sigma
            | GenRegex::SigmaStar
            | GenRegex::Range(_, _)
            | GenRegex::CharExpression(_)
            | GenRegex::StringVar(_) => false,
        }
    }
}

/*
    Ordering logic
*/
impl PartialOrd for GenRegex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.length().cmp(&other.length()))
    }
}

impl Ord for GenRegex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.length().cmp(&other.length())
    }
}

/*
    Pretty printing
*/

use std::fmt::{self, Display};

impl Display for GenRegex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", GenRegexPrintHelper::new(self))
    }
}

// Helper struct for pretty printing
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
            #[cfg(target_os = "macos")]
            GenRegex::EmptySet => {
                write!(f, "∅")
            }
            #[cfg(not(target_os = "macos"))]
            GenRegex::EmptySet => {
                write!(f, "{{}}")
            }
            #[cfg(target_os = "macos")]
            GenRegex::Epsilon => {
                write!(f, "ε")
            }
            #[cfg(not(target_os = "macos"))]
            GenRegex::Epsilon => {
                write!(f, "\"\"")
            }
            GenRegex::Sigma => {
                write!(f, ".")
            }
            GenRegex::SigmaStar => {
                write!(f, "(.*)")
            }
            GenRegex::Range(char1, char2) => {
                write!(f, "[{}-{}]", char1, char2)
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
