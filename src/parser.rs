use serde_json;

use crate::DepJson;
use crate::_DepJson;
use crate::structs::ConfigEle;

const ILLEGAL_CHAR:&'static [&'static str] = &["[", "]", "=", "y"];
const TYPE_FILTER:&'static [&'static str] = &["bool", "tristate"];

/// deserde the json and return an DepJson. Additionally, it does two following things:
/// (1) filter bool and tristate. for the unspported type, it will only report a trace and skip
/// (2) for the unsupported dep and rev, it will report an warn and just skip it respectively
///

pub fn parse_json(json_str:&str) -> Result<DepJson, serde_json::Error>{
   let parse_res:Result<_DepJson, serde_json::Error> = serde_json::from_str(&json_str);
   if let Err(e) = parse_res{
       return Err(e);
   }
   let parse_res = parse_res.unwrap();

   let mut res = DepJson::new();
   for (config,val) in parse_res.into_iter(){
       let key = config.clone();
       let info = val[0].clone();

       let mut if_dep_valid = true;
       let mut if_rev_valid = true;
       // filter the type of configuration
       if TYPE_FILTER.contains(&info.type_.as_str()){
           debug!("unsupported data type:{} while parsing the config:{}", info.type_.as_str(), config.clone());
           debug!("skipping...");
           continue;
       }

       // TODO: do we need to filter the invalid dependencies or rev_selects
       // here we do a trace
        for ch in ILLEGAL_CHAR{
            let depend_str = &info.dep;
            let select_str = &info.rev_select;

            if depend_str.contains(ch){
                warn!("unsupported depdend string:{}, contains {} for {}", depend_str, ch, key);
                warn!("this error will be ignored here");
                if_dep_valid = false;
            }

            if select_str.contains(ch){
                warn!("unsupported select string:{}, contains {} for {}", depend_str, ch, key);
                warn!("this error will be ignored here");
                if_rev_valid = false;
            }
        }

        res.insert(key,ConfigEle {
            type_: info.type_, 
            rev_select: if if_rev_valid {info.rev_select} else {String::new()},
            dep: if if_rev_valid {info.dep} else {String::new()}, 
            restrict: info.restrict,
        });
   }
   trace!("after preprocess: {} items", res.len());
   Ok(res)
}
