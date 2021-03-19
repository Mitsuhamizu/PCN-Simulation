extern crate csv;
use core::{f32, str};
use csv::Reader;
use serde::Deserialize;
use std::{collections::HashMap, env, error::Error, fs::File, io, io::BufReader, u32};

#[derive(Deserialize, Debug)]
struct Paramaters {
    snapshot: u32,
    count: u32,
    amount: u32,
}

#[derive(Deserialize, Debug)]
struct Ln_edges {
    src: String,
    trg: String,
    fee: u32,
}
impl Ln_edges {
    pub fn new(src: &str, trg: &str, fee: u32) -> Self {
        Ln_edges {
            src: src.to_string(),
            trg: trg.to_string(),
            fee,
        }
    }
}

fn main() {
    // read params
    let params_path = format!("{}/src/data/params.json", env!("CARGO_MANIFEST_DIR"));
    let params = match read_json_from_file(&&params_path) {
        Ok(params) => params,
        Err(error) => panic!("{:?}", error),
    };

    // generate graph
    let ln_edges_path = format!("{}/src/data/ln_edges.csv", env!("CARGO_MANIFEST_DIR"));
    load_graph(&ln_edges_path, &params);
}

fn load_graph(data_path: &String, params: &Paramaters) {
    // read csv
    let mut csv = match read_csv(data_path) {
        Ok(file) => file,
        Err(error) => panic!("{}", &error),
    };

    // filter csv
    // let csv_filtered = filter_csv(csv, &params);

    filter_csv(csv, params);

    //
}

fn filter_csv(mut csv_rdr: Reader<File>, params: &Paramaters) -> Result<(), Box<dyn Error>> {
    // read the snapshot && the amount
    let snapshot_id = params.snapshot;
    let amount = params.amount;
    let amount_flot = amount as f32;
    let mut csv_wtr = csv::Writer::from_writer(io::stdout());
    let mut ln_edges: Vec<Ln_edges> = vec![];
    let mut capacity_map: HashMap<(String, String), (u32, u32)> = HashMap::new();
    for result in csv_rdr.records() {
        let record = result?;

        // check the sdnapshot_id && disabled.
        let capacity: u32 = *&record[6].to_string().parse().unwrap();

        if &record[1] != "0" || &record[7] == "True" || capacity < amount {
            continue;
        }

        // load info.
        let src = record[2].to_string();
        let trg = record[3].to_string();
        let base_fee: f32 = record[8].to_string().parse().unwrap();
        let rate_fee: f32 = record[9].to_string().parse().unwrap();
        let id = (src.clone(), trg.clone());
        let id_revesed = (src.clone(), trg.clone());

        // Insert Ln edges.
        ln_edges.push(Ln_edges::new(
            &src,
            &trg,
            (base_fee / 1000.0 + rate_fee * amount_flot / (u32::pow(10, 6) as f32)) as u32,
        ));

        // Init capacity

        if capacity_map.contains_key(&id) {
        } else if capacity_map.contains_key(&id_revesed) {
        } else {
        }
        break;
        // capacity_map.insert((src,trg), );
    }
    // println!("{:?}", ln_edges);
    Ok(())
}

fn read_csv(data_path: &String) -> Result<Reader<File>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(data_path)?;
    Ok((rdr))
}

fn read_json_from_file(file_path: &String) -> Result<Paramaters, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;

    Ok(u)
}
