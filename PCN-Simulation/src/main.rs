mod generator;
mod reader;
mod structure;

use petgraph::{
    algo,
    graph::{self, node_index},
    Graph,
};
use rand::Rng;
use std::{collections::HashMap, error::Error};
use structure::Paramaters;

fn main() {
    // read params
    let params_path = format!("{}/src/data/params.json", env!("CARGO_MANIFEST_DIR"));
    let params = match reader::read_json_from_file(&&params_path) {
        Ok(params) => params,
        Err(error) => panic!("{:?}", error),
    };

    // generate graph
    let ln_edges_path = format!("{}/src/data/ln_edges_0.csv", env!("CARGO_MANIFEST_DIR"));
    let (ln_network, balance_map) = match load_graph(&ln_edges_path, &params) {
        Ok((ln_network, balance_map)) => (ln_network, balance_map),
        Err(error) => panic!("{:?}", error),
    };

    // generate tx pair.
    let node_number = ln_network.node_count();

    // try to find path

    let src: graph::NodeIndex<u32> = node_index(111);
    let trg: graph::NodeIndex<u32> = node_index(1845);
    let path = algo::astar(&ln_network, src, |n| n == trg, |e| *e.weight(), |_| 0);
    println!("{:?}", path);
    // for edge in ln_network.edges(a) {
    //     println!("{:?}", edge);
    // }
}

fn load_graph(
    data_path: &String,
    params: &Paramaters,
) -> Result<(Graph<(), u32>, HashMap<(u32, u32), u32>), Box<dyn Error>> {
    // read csv
    let csv = match reader::read_csv(data_path) {
        Ok(file) => file,
        Err(error) => panic!("{}", &error),
    };

    // filter csv
    // let csv_filtered = filter_csv(csv, &params);

    let (ln_edges, balance_map) = match generator::generate_edges_from_csv(csv, params) {
        Ok((ln_edges, balance_map)) => (ln_edges, balance_map),
        Err(error) => panic!("{}", &error),
    };

    // map address.
    let address_mapping = match mapping_address(&ln_edges) {
        Ok(address_mapping) => address_mapping,
        Err(error) => panic!("{}", &error),
    };

    // convert string to u32.
    let mut ln_edges_numerical: Vec<(u32, u32, u32)> = vec![];
    for edge_string in ln_edges {
        ln_edges_numerical.push((
            *address_mapping.get(&edge_string.0).unwrap(),
            *address_mapping.get(&edge_string.1).unwrap(),
            edge_string.2,
        ));
    }
    let ln_network = Graph::<(), u32>::from_edges(ln_edges_numerical);

    // convert balance map.
    let mut balance_map_numerical: HashMap<(u32, u32), u32> = HashMap::new();
    for (key, value) in &balance_map {
        balance_map_numerical.insert(
            (
                *address_mapping.get(&key.0).unwrap(),
                *address_mapping.get(&key.1).unwrap(),
            ),
            *value,
        );
    }

    Ok((ln_network, balance_map_numerical))
}

fn mapping_address(
    ln_edges: &Vec<(String, String, u32)>,
) -> Result<HashMap<String, u32>, Box<dyn Error>> {
    // convert string to usize.
    let mut address_mapping: HashMap<String, u32> = HashMap::new();
    let mut counter = 0;
    for edges in ln_edges {
        let (src, trg, _) = edges;
        for address in vec![src, trg] {
            if let None = address_mapping.get(address) {
                address_mapping.insert(address.clone(), counter);
                counter += 1;
            }
        }
    }
    Ok(address_mapping)
}
