#[macro_use]
extern crate log;

use std::collections::HashMap;

use serde::Serialize;
use serde::Deserialize;

// use serde_json::Result;
// use serde_json::Deserializer;

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigEle{
    #[serde(rename="type")]
    type_:String,
    rev_select:String,
    dep:String,
    restrict:String,
}

pub type _DepJson = HashMap<String,Vec<ConfigEle>>;
pub type DepJson = HashMap<String,ConfigEle>;

const TYPE_FILTER:&'static [&'static str] = &["bool", "tristate"];

pub fn preprocess(raw_json:_DepJson) ->DepJson{
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

