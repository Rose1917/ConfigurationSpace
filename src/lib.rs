#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use regex::Regex;
use regex::Captures;

use serde::Serialize;
use serde::Deserialize;

use cnfgen::boolexpr_creator::ExprCreator32;
use cnfgen::boolexpr::BoolExprNode;

use rustlogic::LogicNode;
use varisat::CnfFormula;
use varisat::ExtendFormula;
use varisat::dimacs::write_dimacs;
use varisat::{Var, Lit};


use crate::structs::DepJson;
use crate::structs::_DepJson;





pub mod utils;
pub mod structs;
pub mod parser;

pub fn preprocess(raw_json:_DepJson) ->DepJson {
    DepJson::new()
}

//exact and sort
pub fn exact_config(expr:Box<LogicNode>) -> (Vec<String>, HashMap<String, usize>){
    let mut names = HashMap::new();
    let mut un_resolved:Vec<Box<LogicNode>> = vec![];

    //init
    un_resolved.push(expr);
    while !un_resolved.is_empty() {
        let cur_node = un_resolved.pop().unwrap();
        match *cur_node{
            LogicNode::And(l,r) => {
                un_resolved.push(l);
                un_resolved.push(r);
            },
            LogicNode::Or(l,r) => {
                un_resolved.push(l);
                un_resolved.push(r);
            },
            LogicNode::Not(node) => {
                un_resolved.push(node);
            },
            LogicNode::Variable(name) =>{
                names.insert(name,true);
            },
            _ => {
                unreachable!();
            }
        }
    }

    let mut config_set:Vec<String> = names.keys().cloned().collect();
    config_set.sort();
    for config in config_set.iter(){
        debug!("{config}");
    }

    let mut config_index:HashMap<String, usize> = HashMap::new();
    for (i,config) in config_set.iter().enumerate(){
        config_index.insert(config.clone(), i);
    }
    (config_set,config_index)
}

///create all the needed variables
pub fn create_variables(n:usize) -> (Rc<RefCell<ExprCreator32>>,Vec<BoolExprNode<i32>>){
    let creator = ExprCreator32::new();
    let mut res = vec![];
    for _ in 0..n{
       res.push(BoolExprNode::variable(creator.clone()));
    }
    error!("variables size:{}", res.len());
    (creator, res)
}

pub fn parse_formula(bool_expr:&str) ->Option<rustlogic::LogicNode>{
    info!("raw expr:{bool_expr}");

    if bool_expr.is_empty(){
        info!("empty string");
        return None;
    }

    // make it compatible to the rustlogic library
    let re = Regex::new("[a-zA-Z_0-9]+").unwrap();
    let s = bool_expr.to_owned()
        .replace("&&", "&")
        .replace("||","|")
        .replace(" ", "")
        .replace("!", "~");

    let result = re.replace_all(&s, |caps: &Captures| {
        format!("[{}]", &caps[0])});
    error!("result:::{result}");

    let res = rustlogic::parse(&result);
    match res{
        Err(e) =>{
            error!("parsing the bool expression {bool_expr} failed");
            error!("{e}");
            return None;
            //
        },
        Ok(res) =>{
            error!("parsing the bool expression {bool_expr} success");
            return Some(res);
        }
    };


}

pub fn parse_cnf(cur_expr:Box<LogicNode>) ->Box<LogicNode>{
    let cur_expr = utils::optimize(cur_expr);
    match *(cur_expr.clone()) {
        LogicNode::Or(left_node, rigt_node) =>{
            // Or(Variable("B"), And(Variable("D"), Variable("E")))
            let left_cnf = parse_cnf(left_node);
            let rigt_cnf = parse_cnf(rigt_node);

            let flatten_left = utils::flatten_cnf(left_cnf); 
            let flatten_rigt = utils::flatten_cnf(rigt_cnf);

            let mut res_vec:Vec<LogicNode> = vec![];
            for left_item in &flatten_left{
                for rigt_item in &flatten_rigt{
                    res_vec.push(LogicNode::Or(left_item.to_owned(),rigt_item.to_owned()));
                }
            }
            
            println!("{:?}", res_vec);
            let init = res_vec[0].clone();
            println!("{:?}", init);
            let res = res_vec.iter().skip(1).
                fold(init, |acc:LogicNode, x:&LogicNode|->LogicNode{LogicNode::And(Box::new(acc), Box::new(x.clone()))});
                // reduce(|acc, x|-> LogicNode{LogicNode::And(&Box::new(acc), x.clone())});
            println!("{:?}", res);
            return Box::new(res);
        },

        LogicNode::And(left_node, rigt_node) =>{
            let left_cnf = parse_cnf(left_node);
            let rigt_cnf = parse_cnf(rigt_node);
            return Box::new(LogicNode::And(left_cnf, rigt_cnf));
        },

        LogicNode::Not(node) =>{
            match *node{
                LogicNode::Not(not_node) =>{
                    return not_node;
                },
                // not (A and B) => (not A) or (not B)
                LogicNode::And(left, rigt) =>{
                    return parse_cnf(Box::new(LogicNode::Or(Box::new(LogicNode::Not(left)), Box::new(LogicNode::Not(rigt)))));
                },
                //not (A or B) => 
                LogicNode::Or(left, rigt) =>{
                   return parse_cnf(Box::new(LogicNode::Or(Box::new(LogicNode::Not(left)), Box::new(LogicNode::Not(rigt)))));
                },

                //not true
                LogicNode::True => {
                    return Box::new(LogicNode::False);
                },

                //not false
                LogicNode::False => {
                    return Box::new(LogicNode::True);
                },

                LogicNode::Variable(_) => {
                    return cur_expr;
                },
            };
        },

        LogicNode::Variable(v) =>{
            return cur_expr;
        },

        LogicNode::True => {
            return cur_expr;
        },

        LogicNode::False => {
            return cur_expr;
        }
    }
}

pub fn dimacs_trans(dep_obj:DepJson) ->String{
    "todo".to_owned()
}


#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn test_parse(){
        let p = parse_formula("A&&(B||!(D&&E))");
        // A && (B || !(D&&E))
        // A && (B || (!D) || (!E))
        println!("{:?}", p.clone().unwrap());
        let (names, config2index) = exact_config(Box::new(p.clone().unwrap()));
        println!("names:{:?}", names);
        println!("config2index:{:?}", config2index);

        //transfer the bool expression to cnf
        let v = parse_cnf(Box::new(p.unwrap()));

        //construct the dimacs all the variables
        let mut formula = CnfFormula::new();
        


        let flat_cnf = utils::flatten_cnf(v.clone()) ;
        for clause in flat_cnf{
            // one clause
            let flat_dnf = utils::flatten_dnf(clause);
            let mut clause:Vec<varisat::Lit> = vec![];     
            for node in flat_dnf{
                match *node{
                    LogicNode::Variable(v) =>{
                        println!("variable {}", v);
                        clause.push(varisat::Lit::from_dimacs((config2index[&v]+1) as isize));            
                    },
                    LogicNode::Not(not_node) =>{
                        match *not_node {
                            LogicNode::Variable(not_s) => {
                               println!("not variable {}", not_s); 
                               clause.push(varisat::Lit::from_dimacs(-((config2index[&not_s]+1) as isize)));            
                            },
                            _ => unreachable!()
                        }
                    },

                    _ => unreachable!()
                }
            }

            formula.add_clause(clause.as_slice());
            println!("current clause: {:?}", clause);
        }
        println!("{:?}", *v);
        let mut implements_write = vec![];
        write_dimacs(&mut implements_write, &formula);
        let res = String::from_utf8(implements_write).unwrap();
        println!("write res:\n{}", res);
    }

}
