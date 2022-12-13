#[macro_use]
extern crate log;

use std::path::Path;
use std::fs;


use serde_json::Result;

use json2model::{ConfigEle,DepJson,_DepJson};
use json2model::{preprocess, exact_config, create_variables, parse_formula, parse_cnf, dimacs_trans};

use cnfgen::boolexpr::BoolExprNode;
use cnfgen::boolexpr::BoolImpl;
use cnfgen::writer::CNFWriter;

fn parse(file_path:&Path) ->Option<String>{
    let file_path = file_path.to_str().unwrap();
    let file_str = fs::read_to_string(file_path)
        .expect(&format!("an error occurred while reading {}, skipping...", file_path));

    debug!("file_str:{file_str}");

    let parse_res:Result<_DepJson> = serde_json::from_str(&file_str);
    
    if let Err(e) = parse_res{
        error!("parsing the file file_path failed");
        error!("error:{:?}, skipping {}.", e, file_path); 
        return None;
    };

    //do the basic filter and unwrap
    let parse_res = parse_res.unwrap();
    let parse_res = preprocess(parse_res);

    //debug
    for config in parse_res.keys(){
        info!("{}", config);
    }

    //extract all the configurations
    let (index2config,config2index) = exact_config(&parse_res);

    //create as many variables as the config set
    let (creator,variables) = create_variables(index2config.len());

    //TODO:tristate
    let mut res_cnf:Vec<BoolExprNode<i32>> = vec![];

    for config in index2config.iter(){
        error!("cur config:{}", config);
        let cur_item = &parse_res[config];
        
        let dep_formula = parse_formula(&cur_item.dep);
        let rev_formula = parse_formula(&cur_item.rev_select);
        error!("dep_formula:{:?}", dep_formula);

        if dep_formula.is_some(){
            let dep_nodes = parse_cnf(Box::new(dep_formula.unwrap()));
        }else if cur_item.dep.is_empty(){
            info!("empty depency");
            // do nothing
        }
        else{
            warn!("we have encounter an error while parsing dep_str for config {}", config);
            warn!("the error expr is {}", &cur_item.dep);
            warn!("skipping...");
        }

        if rev_formula.is_some(){
            let rev_nodes = parse_cnf(Box::new(rev_formula.unwrap()));
            // error!("{:?}", rev_nodes);
        }else if cur_item.rev_select.is_empty(){
            info!("empty rev select");
            // do nothing
        }else{
            warn!("we have encounter an error while parsing rev_select for config {}", config);
            warn!("the error expr is {}", &cur_item.rev_select);
            warn!("skipping...");
        }
    }

    let mut final_dimacs = res_cnf[0].clone();
    for i in 1..res_cnf.len(){
        error!("i:{}", i);
        final_dimacs = final_dimacs.clone() ^ res_cnf[i].clone();
    }

    error!("{:?}", final_dimacs);
    Some("".to_owned())
}

fn main() {
    env_logger::init();

    let dep_jsons: Vec<String> = std::env::args().collect();
    let dep_jsons = &dep_jsons[1..].to_owned(); //since the json format only has one
                                                              //element in the disctionary for each
                                                              
    if dep_jsons.len() == 0{
        println!("json2model is used to convert upstream json files to models");
        println!("usage:json2model dep1.json [dep2.lua [dep3.json]]..");
        std::process::exit(0);
    }

    debug!("captured {} files to process", dep_jsons.len());

    /* process every single file */
    for file in dep_jsons{
        let file_path = Path::new(&file);
        if !file_path.exists(){
            println!("file {} does not exist.",file);
            continue;
        }
        parse(&file_path);
    }
}
