mod generator;
mod ln_errors;
mod reader;
mod structure;

use adjacent_pair_iterator::AdjacentPairIterator;
use core::panic;
use generator::generate_payment;
use ln_errors::LnError;
use petgraph::{
    algo,
    graph::{self, node_index, EdgeIndex, NodeIndex},
    Graph,
};
use std::{collections::HashMap, error::Error, u32, vec};
use structure::{Config, Paramaters};

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
    let node_number = ln_network.node_count() as u32;
    let payments = match generate_payment(params.count, node_number, params.amount) {
        Ok(payments) => payments,
        Err(error) => panic!("{:?}", error),
    };
    // simulation

    let config = Config { retry_times: 10 };
    simulation(ln_network, payments, balance_map, config);
    // let src: graph::NodeIndex<u32> = node_index(111);
    // let trg: graph::NodeIndex<u32> = node_index(1845);
    // let path = algo::astar(&ln_network, src, |n| n == trg, |e| *e.weight(), |_| 0);
    // println!("{:?}", path);
    // for edge in ln_network.edges(a) {
    //     println!("{:?}", edge);
    // }
}

fn process_payment(
    balance_map: &mut HashMap<(u32, u32), u32>,
    path: Vec<NodeIndex>,
    amounts: Vec<u32>,
) -> Result<(), LnError> {
    // unwrap the path.
    let mut path_numerical: Vec<u32> = vec![];
    for nodeindex in &path {
        path_numerical.push(nodeindex.index() as u32);
    }

    let mut amounts_iter = amounts.into_iter();
    // check the amount.
    for edge_index in path_numerical.clone().into_iter().adjacent_pairs() {
        let current_amount = amounts_iter.next().unwrap();
        if balance_map.get(&edge_index).unwrap() < &current_amount {
            return Err(LnError::InsufficientBalance((edge_index.0, edge_index.1)));
        }
    }

    Ok(())
}
fn simulation(
    ln_network: Graph<(), u32>,
    payments: Vec<(u32, u32, u32)>,
    mut balance_map: HashMap<(u32, u32), u32>,
    config: Config,
) {
    let successful_payment: Vec<u32> = vec![];
    for payment in &payments {
        let attempt_time = 0;
        let (src, trg, amount): (graph::NodeIndex<u32>, graph::NodeIndex<u32>, u32) = (
            node_index(payment.0 as usize),
            node_index(payment.1 as usize),
            payment.2,
        );

        while true {
            if let Some((fee_total, path)) =
                algo::astar(&ln_network, src, |n| n == trg, |e| *e.weight(), |_| 0)
            {
                // generate amount.
                let mut amount_accumulated = amount + fee_total;
                let mut amounts: Vec<u32> = vec![];
                for edge_index in path.clone().into_iter().adjacent_pairs() {
                    let current_edge_index =
                        ln_network.find_edge(edge_index.0, edge_index.1).unwrap();
                    let current_fee = ln_network.edge_weight(current_edge_index).unwrap();
                    amount_accumulated -= current_fee;
                    amounts.push(amount_accumulated);
                }
                process_payment(&mut balance_map, path, amounts);
            // There is path, so just try to do the payment.
            } else {
                // There is no path, just fails.
            }
            break;
        }
        break;
    }
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
