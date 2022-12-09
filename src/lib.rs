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
use cnfgen::writer::CNFWriter;

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

pub fn parse_cnf(dep_formula:LogicNode, vars:&Vec<BoolExprNode<i32>>,config2index:&HashMap<String,usize>) ->BoolExprNode<i32>{
    let dep_clone = dep_formula.clone();
    match dep_formula{
        LogicNode::And(left_node, rigt_node) =>{
            error!("and parse_cnf:{:?}", dep_clone);
            let left_cnf = parse_cnf(*left_node, vars, config2index);
            let rigt_cnf = parse_cnf(*rigt_node, vars, config2index);
            error!("and left:{:?}", left_cnf);
            error!("and right:{:?}", rigt_cnf);
            let res = (left_cnf & rigt_cnf);
            res.write(&mut CNFWriter::new(std::io::stdout()));
            return res;
        },
        LogicNode::Or(left_node, rigt_node) =>{
            error!("or parse_cnf:{:?}", dep_clone);
            let left_cnf = parse_cnf(*left_node, vars, config2index);
            let rigt_cnf = parse_cnf(*rigt_node, vars, config2index);
            return left_cnf|rigt_cnf;
        },
        LogicNode::Not(node) =>{
            error!("not parse_cnf:{:?}", dep_clone);
            return !parse_cnf(*node, vars, config2index);
        },

        LogicNode::Variable(v) =>{
            error!("var parse_cnf:{:?}", dep_clone);
            let var_index = config2index.get(&v);
            if var_index.is_none(){
                error!("{} is in rev or select string, but can not be found in the entry json", v);
                return vars[0].clone();
            }

            return vars[*var_index.unwrap()].clone();
        }

        _ => {
            error!("unknown type of logic node");
            return vars[0].clone();
        }
    }
}

pub fn dimacs_trans(dep_obj:DepJson) ->String{
    "todo".to_owned()
}
