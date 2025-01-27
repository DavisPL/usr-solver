use crate::antimirov::{derivative, nullable};
use crate::classes::{
    CharExpression, CharVar, GenRegex,
};
use std::collections::HashSet;
use std::rc::Rc;

pub struct SatChecker{
    fresh_var_ind: i32,
}

impl SatChecker{
    pub fn new()-> Self{
        Self { 
            fresh_var_ind:0,
        }
    }
    pub fn satisfiable(&mut self, expr: &Rc<GenRegex>)-> bool{
        let mut sat_stack=vec![expr.clone()];
        let mut visted:HashSet<Rc<GenRegex>>=HashSet::new();
        while let Some(gre)=sat_stack.pop(){
            if !nullable(&gre).is_empty(){
                return true;
            }
            let deriv=derivative(&gre, &self.get_fresh_var());
            for ele in deriv{
                sat_stack.push(ele.get_expr().clone());
            }
            visted.insert(gre);
        }
        false
    }
    fn get_fresh_var(&mut self)->Rc<CharExpression>{
        let var_name=format!("f.{}",self.fresh_var_ind.to_string());
        self.fresh_var_ind+=1;
        Rc::new(CharExpression::CharVar(CharVar{name:var_name}))
    }
}