#[macro_use]
extern crate log;

use std::path::Path;
use std::fs;


use serde_json::Result;

use json2model::{ConfigEle,DepJson,_DepJson};
use json2model::{preprocess, dimacs_trans};

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

    for config in parse_res.keys(){
        info!("{}", config);
    }

    let dimacs_res = dimacs_trans(parse_res);
    debug!("{dimacs_res}");
    Some(dimacs_res)
}

fn main() {
    env_logger::init();

    let dep_jsons: Vec<String> = std::env::args().collect();
    let dep_jsons = &dep_jsons[1..].to_owned(); //since the json format only has one
                                                              //element in the disctionary for each
                                                              //config
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
