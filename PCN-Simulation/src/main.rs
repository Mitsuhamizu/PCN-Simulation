use relative_path::RelativePath;
use serde::Deserialize;
use std::env;
use std::path::Path;
use std::{collections::HashMap, error::Error};
use std::{fs::File, u32};
use std::{io::BufReader, path};

fn main() {
    // read params
    let params_path = format!("{}/src/data/params.json", env!("CARGO_MANIFEST_DIR"));
    let params = match read_json_from_file(&&params_path) {
        Ok(params) => params,
        Err(error) => panic!("{:?}", error),
    };
    // read csv
}

fn read_json_from_file(file_path: &String) -> Result<HashMap<String, u32>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}
