use crate::structs::DepJson;
use crate::structs::ConfigModel;

use rustlogic::LogicNode;

use std::collections::HashMap;



pub fn flatten_cnf(expr:Box<LogicNode>) -> Vec<Box<LogicNode>>{
   let mut res:Vec<Box<LogicNode>> = vec![];

   let mut un_resolved:Vec<Box<LogicNode>> = vec![];
   un_resolved.push(expr);

   while !un_resolved.is_empty() {
       let cur_node = un_resolved.pop().unwrap();
       match *cur_node.clone(){
           LogicNode::And(l,r) => {
                un_resolved.push(l);
                un_resolved.push(r);
           },
           LogicNode::Or(_, _) => {
                res.push(cur_node);
           },
           LogicNode::Not(_) => {
               res.push(cur_node);
           },
           LogicNode::Variable(_) =>{
               res.push(cur_node);
           },
           _ => {
               unreachable!();
           }
       }
   }
   return res;
}

pub fn flatten_dnf(expr:Box<LogicNode>) -> Vec<Box<LogicNode>>{
   let mut res:Vec<Box<LogicNode>> = vec![];

   let mut un_resolved:Vec<Box<LogicNode>> = vec![];
   un_resolved.push(expr);

   while !un_resolved.is_empty() {
       let cur_node = un_resolved.pop().unwrap();
       match *cur_node.clone(){
           LogicNode::Or(l, r) => {
               un_resolved.push(l);
               un_resolved.push(r);
           },
           LogicNode::Not(_)|LogicNode::Variable(_) => {
               res.push(cur_node);
           },
           //
           // LogicNode::And(_,_) => {
           //     unreachable!();
           // },
           _ => {
               unreachable!();
           }
       }
   }
   return res;
}

pub fn optimize(expr:Box<LogicNode>) -> Box<LogicNode>{
    match *expr.clone() {
        LogicNode::And(left, rigt) => {
            if let LogicNode::True = *left{
                return rigt;
            }
            if let LogicNode::True = *rigt{
                return left;
            }
            let left_optimized = optimize(left);
            let rigt_optimized = optimize(rigt);
            return Box::new(LogicNode::And(left_optimized, rigt_optimized));
        }

        LogicNode::Or(left, rigt) => {
            if let LogicNode::False = *left{
                return rigt;
            }
            if let LogicNode::False = *rigt{
                return left;
            }
            let left_optimized = optimize(left);
            let rigt_optimized = optimize(rigt);
            return Box::new(LogicNode::Or(left_optimized, rigt_optimized));
        }

        _ => {
            return expr;
        }
    }
}

//exact and sort
pub fn gen_configs(json_obj:DepJson) -> ConfigModel{
    let mut config_set:HashMap<String, bool> = HashMap::new();
    let mut visible_set:HashMap<String, bool> = HashMap::new();
    let mut tristate_set:HashMap<String, bool> = HashMap::new();

    for (config, info) in &json_obj{
        config_set.insert(config.to_owned(), true);
        if info.type_ == "tristate"{
            config_set.insert(format!("{}_MODULE",config), true);
            tristate_set.insert(config.to_owned(), true);
        }

        if !info.dep.trim().is_empty(){
            config_set.insert(format!("{}_VISIBLE", config), true);
            visible_set.insert(config.to_owned(), true);
        }
    }

    let mut config_set:Vec<String> = config_set.keys().cloned().collect();
    config_set.sort();

    for config in config_set.iter(){
        trace!("{config}");
    }

    let mut config2index:HashMap<String, usize> = HashMap::new();
    for (i,config) in config_set.iter().enumerate(){
        config2index.insert(config.clone(), i);
    }

    ConfigModel { 
        json_obj,
        config2index,
        tristates: tristate_set.keys().cloned().collect(),
        config_set,
        visible_set:visible_set.keys().cloned().collect()
    }
}
