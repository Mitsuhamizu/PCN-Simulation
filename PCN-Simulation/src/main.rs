use core::{f32, str};
use csv::Reader;
use rand::Rng;
use serde::Deserialize;
use std::{collections::HashMap, error::Error, fs::File, io::BufReader, u32};

#[derive(Deserialize, Debug)]
struct Paramaters {
    count: u32,
    amount: u32,
}

#[derive(Deserialize, Debug)]
struct LnEdge {
    src: String,
    trg: String,
    fee: u32,
}
impl LnEdge {
    pub fn new(src: &str, trg: &str, fee: u32) -> Self {
        LnEdge {
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
    let ln_edges_path = format!("{}/src/data/ln_edges_0.csv", env!("CARGO_MANIFEST_DIR"));
    load_graph(&ln_edges_path, &params);
}

fn load_graph(data_path: &String, params: &Paramaters) {
    // read csv
    let csv = match read_csv(data_path) {
        Ok(file) => file,
        Err(error) => panic!("{}", &error),
    };

    // filter csv
    // let csv_filtered = filter_csv(csv, &params);

    filter_csv(csv, params);

    //
}

fn filter_csv(mut csv_rdr: Reader<File>, params: &Paramaters) -> Result<(), Box<dyn Error>> {
    let amount = params.amount;
    let amount_flot = amount as f32;
    let mut ln_edges: Vec<LnEdge> = vec![];
    let mut capacity_map: HashMap<(String, String), u32> = HashMap::new();
    for result in csv_rdr.records() {
        let record = result?;

        // check the sdnapshot_id && disabled.
        let capacity: u32 = *&record[7].to_string().parse().unwrap();

        // Drop the edges with insufficient capacity.
        if capacity < amount {
            continue;
        }

        // load info.
        let src = record[3].to_string();
        let trg = record[4].to_string();
        let base_fee: f32 = record[9].to_string().parse().unwrap();
        let rate_fee: f32 = record[10].to_string().parse().unwrap();
        let id = (src.clone(), trg.clone());

        // load
        if let Some(current_capacity) = capacity_map.get_mut(&id) {
            *current_capacity += capacity;
        } else {
            capacity_map.insert(id, capacity);

            // Insert Ln edges.
            ln_edges.push(LnEdge::new(
                &src,
                &trg,
                (base_fee / 1000.0 + rate_fee * amount_flot / (u32::pow(10, 6) as f32)) as u32,
            ));
        }
    }
    // Init capacity.

    Ok(())
}

// fn generate_balance() -> Result<HashMap<(String, String), u32>, Box<dyn Error>> {
fn generate_balance(capacity_map: HashMap<(String, String), u32>) {
    let mut balance_map: HashMap<(String, String), u32> = HashMap::new();
    let mut rng = rand::thread_rng();
    for (id, capacity) in &capacity_map {
        let (src, trg) = id;
        let id_reversed = (trg.clone(), src.clone());

        // If the reversed channel is enabled.
        let ratio = rng.gen::<f32>();
        if let Some(capacity_reversed) = capacity_map.get(&id_reversed) {
            if capacity_reversed > capacity {
                capacity_map.insert(*id, capacity * ((ratio * 10000.0).round() as u32));
            } else if capacity_reversed < capacity {
            } else {
            }
        } else {
        }
    }
}

fn read_csv(data_path: &String) -> Result<Reader<File>, Box<dyn Error>> {
    let rdr = Reader::from_path(data_path)?;
    Ok(rdr)
}

fn read_json_from_file(file_path: &String) -> Result<Paramaters, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}
