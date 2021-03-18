extern crate csv;
use csv::{Reader, Writer};
use serde::Deserialize;
use std::{collections::HashMap, env, error::Error, fs::File, io, io::BufReader, u32, vec};

#[derive(Deserialize, Debug)]
struct Paramaters {
    snapshot: u32,
    count: u32,
    amount: u32,
}

#[derive(Deserialize, Debug)]
struct Ln_edges {
    snapshot_id: u32,
    src: String,
    trg: String,
    last_update: String,
    channel_id: String,
    capacity: u32,
    disabled: bool,
    fee_base_msat: f32,
    fee_rate_milli_msat: f32,
    min_htlc: String,
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
    let mut csv_wtr = csv::Writer::from_writer(io::stdout());
    csv_wtr.write_record(csv_rdr.headers()?)?;
    for result in csv_rdr.records() {
        let record: Ln_edges = result?;
        println!("{:?}", &record);
    }
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
