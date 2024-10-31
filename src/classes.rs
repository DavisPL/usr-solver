use std::rc::Rc;

pub enum GenRegex {
    EmptySet,
    CharExpression(CharExpression),
    StringObject(StringObject),
    Union(Rc<GenRegex>, Rc<GenRegex>),
    Intersect(Rc<GenRegex>, Rc<GenRegex>),
    Concatenation(Rc<GenRegex>, Rc<GenRegex>),
    Kleene(Rc<GenRegex>),
    Complement(Rc<GenRegex>),
    IfThenElse(Rc<Predicate>, Rc<GenRegex>, Rc<GenRegex>)
}

pub enum Predicate {
    And(Rc<Predicate>, Rc<Predicate>),
    Or(Rc<Predicate>, Rc<Predicate>),
    Not(Rc<Predicate>),
    True,
    False,
    Equals(CharExpression, CharExpression),
    EqualLength(StringObject, i32)
}

pub enum CharExpression {
    CharVar(String),
    Literal(String),
    StringIndex(StringVar, i32)
}

pub enum StringObject{
    StringSlice(StringVar, i32),
    StringVar(String)
}

pub struct StringVar {
    pub name: String
}
