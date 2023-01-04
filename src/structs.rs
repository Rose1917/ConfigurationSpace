use std::collections::HashMap;

use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigEle{
    #[serde(rename="type")]
    pub type_:String,
    pub rev_select:String,
    pub dep:String,
    pub restrict:String,
}

pub struct ConfigSet{
    pub json_obj:DepJson,
    pub config2index:HashMap<String,usize>,
    pub tristate2index:HashMap<String,usize>,
}

pub type _DepJson = HashMap<String,Vec<ConfigEle>>;
pub type DepJson = HashMap<String,ConfigEle>;
