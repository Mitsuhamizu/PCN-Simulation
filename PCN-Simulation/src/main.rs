use core::{f32, panic, str};
use csv::Reader;
use petgraph::Graph;
use petgraph::IntoWeightedEdge;
use rand::Rng;
use serde::Deserialize;
use std::{collections::HashMap, error::Error, fs::File, io::BufReader, u32, u64};

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

    let (ln_edges, balance_map) = match generate_edges_from_csv(csv, params) {
        Ok((ln_edges, balance_map)) => (ln_edges, balance_map),
        Err(error) => panic!("{}", &error),
    };

    // generate graph from edges.
    let gr = Graph::<(), i32>::from_edges(ln_edges);
}

fn generate_edges_from_csv(
    mut csv_rdr: Reader<File>,
    params: &Paramaters,
) -> Result<(Vec<LnEdge>, HashMap<(String, String), u32>), Box<dyn Error>> {
    let amount = params.amount;
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
        let base_fee = base_fee as u32;
        let rate_fee: f32 = record[10].to_string().parse().unwrap();
        let rate_fee = rate_fee as u32;
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
                (base_fee as u64 / (1000 as u64)
                    + rate_fee as u64 * amount as u64 / u64::pow(10, 6)) as u32,
            ));
        }
    }
    // generate balance.
    let balance_map = match generate_balance(capacity_map) {
        Ok(balance_map) => balance_map,
        Err(error) => panic!("{}", &error),
    };

    Ok((ln_edges, balance_map))
}

// fn generate_balance() -> Result<HashMap<(String, String), u32>, Box<dyn Error>> {
fn generate_balance(
    capacity_map: HashMap<(String, String), u32>,
) -> Result<HashMap<(String, String), u32>, Box<dyn Error>> {
    let mut balance_map: HashMap<(String, String), u32> = HashMap::new();
    let mut rng = rand::thread_rng();
    for (id, capacity) in &capacity_map {
        // generate random number.
        let ratio = rng.gen_range(0, 101);

        // generate reversed id.
        let (src, trg) = id;
        let id_reversed = (trg.clone(), src.clone());

        // If the reversed channel is enabled.
        if let Some(capacity_reversed) = capacity_map.get(&id_reversed) {
            balance_map.insert(
                id.clone(),
                (ratio as u64 * *capacity as u64 / 100 as u64) as u32,
            );
            balance_map.insert(
                id_reversed.clone(),
                (*capacity_reversed as u64 * (100 - ratio) as u64 / 100) as u32,
            );
        } else {
            balance_map.insert(
                id.clone(),
                (ratio as u64 * *capacity as u64 / 100 as u64) as u32,
            );
        }
    }
    Ok(balance_map)
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
