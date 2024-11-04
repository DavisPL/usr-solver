use std::rc::Rc;
use std::collections::HashMap;
use either::Either;
use crate::classes::{GenRegex, Predicate, CharExpression, StringVar, StringIndex};


#[derive(Clone)]
pub struct UnionFind {
    pub parent: HashMap<String, String>,
    rank: HashMap<String, i32>,
}

impl UnionFind {
    pub fn new() -> Self {
        UnionFind {
            parent: HashMap::new(),
            rank: HashMap::new(),
        }
    }

    fn get_key(&self, x: Either<Rc<CharExpression>, Rc<StringIndex>>) -> String {
    match x {
        Either::Left(exp) => match &*exp {
            CharExpression::Literal(val) => format!("Literal_{}", val),
            CharExpression::CharVar(name) => format!("CharVar_{}", name),
        },
        Either::Right(idx) => {
            format!("StringIndex_{}_{}", idx.var.name, idx.index)
        }
    }
    }
    pub fn final_find(&mut self, x: Either<Rc<CharExpression>, Rc<StringIndex>>) -> Either<Rc<CharExpression>, Rc<StringIndex>> {
        let key = self.get_key(x.clone());

        if !self.parent.contains_key(&key) {
            self.parent.insert(key.clone(), key.clone());
            self.rank.insert(key.clone(), 0);
        }

        let parent_key = self.parent.get(&key).unwrap().clone();
        if parent_key != key {
            let new_parent = self.final_find(self.string_to_object(&parent_key));
            let new_parent_key = self.get_key(new_parent.clone());
            self.parent.insert(key.clone(), new_parent_key);
        }

        if parent_key == key {
            if key.starts_with("Literal_") {
                return Either::Left(Rc::new(CharExpression::Literal(
                    key[8..].to_string(),
                )));
            } else if key.starts_with("CharVar_") {
                return Either::Left(Rc::new(CharExpression::CharVar(
                    key[8..].to_string(),
                )));
            } else if key.starts_with("StringIndex_") {
                let parts: Vec<&str> = key[12..].split('_').collect();
                let name = parts[0].to_string();
                let index = parts[1].parse::<i32>().unwrap();
                let string_var = Rc::new(StringVar { name });
                return Either::Right(Rc::new(StringIndex { var: string_var, index }));
            }
        }

        self.string_to_object(&parent_key)
    }
    pub fn string_to_object(&self, x: &String) -> Either<Rc<CharExpression>, Rc<StringIndex>> {
    if x.starts_with("Literal_") {
        Either::Left(Rc::new(CharExpression::Literal(
            x[8..].to_string(),
        )))
    } else if x.starts_with("CharVar_") {
        Either::Left(Rc::new(CharExpression::CharVar(
            x[8..].to_string(),
        )))
    } else if x.starts_with("StringIndex_") {
        let parts: Vec<&str> = x.split('_').collect();
        if parts.len() != 3 {
            panic!("Invalid StringIndex format");
        }
        let var = Rc::new(StringVar {
            name: parts[1].to_string(),
        });
        let index = parts[2].parse::<i32>().unwrap();
        Either::Right(Rc::new(StringIndex { var, index }))
    } else {
        panic!("Unknown format in string_to_object")
    }
}

pub fn find(&mut self, x: Either<Rc<CharExpression>, Rc<StringIndex>>) -> String {
    let key = self.get_key(x.clone());

    if !self.parent.contains_key(&key) {
        self.parent.insert(key.clone(), key.clone());
        self.rank.insert(key.clone(), 0);
    }

    let parent_key = self.parent.get(&key).unwrap().clone();
    if parent_key != key {
        let new_parent = self.find(self.string_to_object(&parent_key));
        self.parent.insert(key.clone(), new_parent.clone());
        new_parent
    } else {
        key
    }
}

    fn simplify(&self, new_root: Either<Rc<CharExpression>, Rc<StringIndex>>) -> String {
        self.get_key(new_root)
    }

pub fn union(
    &mut self,
    x: Either<Rc<CharExpression>, Rc<StringIndex>>,
    y: Either<Rc<CharExpression>, Rc<StringIndex>>,
) -> Result<Either<Rc<CharExpression>, Rc<StringIndex>>, String> {
    let root_x = self.find(x.clone());
    let root_y = self.find(y.clone());

    println!("{} {}", root_x, root_y);

    let x_obj = self.string_to_object(&root_x);
    let y_obj = self.string_to_object(&root_y);

    let new_root = match (&x_obj, &y_obj) {
        (Either::Left(x_exp), Either::Left(y_exp)) => match (&**x_exp, &**y_exp) {
            (CharExpression::Literal(val_x), CharExpression::Literal(val_y)) => {
                if val_x != val_y {
                    println!("union bad");
                    return Err("union bad".to_string());
                }
                return Ok(x_obj.clone());
            }
            (CharExpression::Literal(val), _) => {
                if val.is_empty() {
                    println!("union bad");
                    return Err("union bad".to_string());
                }
                Some(x_obj.clone())
            }
            (_, CharExpression::Literal(val)) => {
                if val.is_empty() {
                    println!("union bad");
                    return Err("union bad".to_string());
                }
                Some(y_obj.clone())
            }
            _ => None,
        },
        (Either::Left(x_exp), _) => match &**x_exp {
            CharExpression::Literal(val) => {
                if val.is_empty() {
                    println!("union bad");
                    return Err("union bad".to_string());
                }
                Some(x_obj.clone())
            }
            _ => None,
        },
        (_, Either::Left(y_exp)) => match &**y_exp {
            CharExpression::Literal(val) => {
                if val.is_empty() {
                    println!("union bad");
                    return Err("union bad".to_string());
                }
                Some(y_obj.clone())
            }
            _ => None,
        },
        (Either::Right(_), Either::Right(_)) => None,
        _ => None,
    };

    let root_x = self.simplify(x_obj.clone());
    let root_y = self.simplify(y_obj.clone());

    if root_x != root_y {
        if let Some(new_root) = new_root {
            let new_root_key = self.simplify(new_root.clone());
            if root_x != new_root_key {
                self.parent.insert(root_x, new_root_key);
            } else {
                self.parent.insert(root_y, new_root_key);
            }
            Ok(new_root)
        } else {
            let rank_x = *self.rank.get(&root_x).unwrap_or(&0);
            let rank_y = *self.rank.get(&root_y).unwrap_or(&0);

            if rank_x > rank_y {
                self.parent.insert(root_y.clone(), root_x.clone());
                Ok(self.string_to_object(&root_x))
            } else if rank_x < rank_y {
                self.parent.insert(root_x.clone(), root_y.clone());
                Ok(self.string_to_object(&root_y))
            } else {
                self.parent.insert(root_y, root_x.clone());
                *self.rank.entry(root_x.clone()).or_insert(0) += 1;
                Ok(self.string_to_object(&root_x))
            }
        }
    } else {
        Ok(self.string_to_object(&root_x))
    }
        }
}
