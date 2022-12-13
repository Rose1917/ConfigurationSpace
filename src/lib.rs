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



#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigEle{
    #[serde(rename="type")]
    pub type_:String,
    pub rev_select:String,
    pub dep:String,
    pub restrict:String,
}

pub type _DepJson = HashMap<String,Vec<ConfigEle>>;
pub type DepJson = HashMap<String,ConfigEle>;

const TYPE_FILTER:&'static [&'static str] = &["bool", "tristate"];
const ILLEGAL_CHAR:&'static [&'static str] = &["[", "]", "=", "y"];

pub mod utils;
pub fn preprocess(raw_json:_DepJson) ->DepJson {
    let mut res = DepJson::new();
    debug!("before preprocess: {} items", raw_json.len());
    for (config,val) in raw_json.into_iter(){
        let key = config;
        let info = val[0].clone();

        // filter the type of configuration
        if TYPE_FILTER.contains(&info.type_.as_str()){
            res.insert(key, info); 
        }
    }
    debug!("after preprocess: {} items", res.len());
    res
}

//exact and sort
pub fn exact_config(dep:&DepJson) -> (Vec<String>,HashMap<String, usize>){
    let mut config_set:Vec<String> = dep.keys().cloned().collect();
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

    for ch in ILLEGAL_CHAR{
        if bool_expr.contains(ch){
            info!("unsupported string:{}, contains {}", bool_expr, ch);
            return None;
        }
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
        LogicNode::And(left_node, rigt_node) =>{
            let left_cnf = parse_cnf(left_node);
            let rigt_cnf = parse_cnf(rigt_node);
            return Box::new(LogicNode::And(left_cnf, rigt_cnf));
        },

        LogicNode::Or(left_node, rigt_node) =>{
            // Or(Variable("B"), And(Variable("D"), Variable("E")))
            let left_cnf = parse_cnf(left_node);
            let rigt_cnf = parse_cnf(rigt_node);

            let flatten_left = utils::flatten_cnf(left_cnf); 
            // B
            let flatten_rigt = utils::flatten_cnf(rigt_cnf);
            //D E

            let mut res_vec:Vec<LogicNode> = vec![];
            for left_item in &flatten_left{
                for rigt_item in &flatten_rigt{
                    res_vec.push(LogicNode::Or(left_item.to_owned(),rigt_item.to_owned()));
                }
            }
            //BD BE
            
            let res = res_vec.iter().
                fold(LogicNode::True, |acc, x|-> LogicNode{LogicNode::And(Box::new(acc), Box::new(x.clone()))});
            return utils::optimize(Box::new(res));
        },

        LogicNode::And(left_node, rigt_node) =>{
            let left_cnf = parse_cnf(left_node);
            let rigt_cnf = parse_cnf(rigt_node);
            return utils::optimize(Box::new(LogicNode::And(left_cnf, rigt_cnf)));
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
        let p = parse_formula("A&&(B||(D&&E))");
        println!("{:?}", p.clone().unwrap());
        let v = parse_cnf(Box::new(p.unwrap()));
        println!("{:?}", *v);
    }



}
