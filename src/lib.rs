#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use serde::Serialize;
use serde::Deserialize;

use cnfgen::boolexpr_creator::ExprCreator32;
use cnfgen::boolexpr::BoolExprNode;

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

pub fn preprocess(raw_json:_DepJson) ->DepJson {
    let mut res = DepJson::new();
    error!("before preprocess: {} items", raw_json.len());
    for (config,val) in raw_json.into_iter(){
        let key = config;
        let info = val[0].clone();

        // filter the type of configuration
        if TYPE_FILTER.contains(&info.type_.as_str()){
            res.insert(key, info); 
        }
    }
    error!("after preprocess: {} items", res.len());
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
    (creator, res)
}

pub fn parse_formula(config:&String, cur_iterm:&ConfigEle, vars:&Vec<BoolExprNode<i32>>, index2config:&HashMap<usize, String>, config2index:&Vec<String>){
    // process dep return a expr

    // process select return another expr
}

pub fn dimacs_trans(dep_obj:DepJson) ->String{
    "todo".to_owned()
}
