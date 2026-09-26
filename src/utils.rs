use fake::{Fake, faker::name::en::Name};
use rand::{random_range, seq::SliceRandom};
use std::{fs, time::Instant};

use crate::{NO_OF_SEARCHES, Node, components::Element, delete_node, find, insert};

pub fn generate_tree(mut root: Node) -> (Node, Vec<u32>) {
    let mut generated_ids: Vec<u32> = Vec::new();

    let mut logs: Vec<String> = Vec::new();

    for _ in 1..crate::MAX_DATA_INPUT {
        let now = Instant::now();
        let r = random_range(1..300);
        generated_ids.push(r);
        let r_element = Element {
            id: r,
            name: Name().fake(),
        };
        (root, _) = insert(root, r_element);
        logs.push(format!("{},{}", r, now.elapsed().as_nanos()));
    }

    fs::write("logs.csv", logs.join("\n")).unwrap();
    (root, generated_ids)
}

pub fn benchmark_utils(root: &Node, mut generated_ids: Vec<u32>) {
    generated_ids.shuffle(&mut rand::rng());

    println!("RANDOM");

    let mut sorted_array = generated_ids.clone();

    sorted_array.sort();

    let mut logs: Vec<String> = Vec::new();

    let now = Instant::now();
    for _ in 1..NO_OF_SEARCHES {
        let pick_random_generated_id = generated_ids.pop().unwrap();

        let now = Instant::now();
        // println!("Searching for {}....", pick_random_generated_id);
        find(&root, pick_random_generated_id);
        logs.push(format!(
            "{},{}",
            pick_random_generated_id,
            now.elapsed().as_nanos()
        ));
    }

    fs::write("search_logs.csv", logs.join("\n")).unwrap();
    println!(
        "{} records retrieved in {} ms",
        NO_OF_SEARCHES,
        now.elapsed().as_millis()
    );

    println!("SEQUENTIAL");
    let now = Instant::now();
    for _ in 1..NO_OF_SEARCHES {
        let pick_random_generated_id = sorted_array.pop().unwrap();

        // println!("Searching for {}....", pick_random_generated_id);
        find(&root, pick_random_generated_id);
    }

    println!(
        "{} records retrieved in {} ms",
        NO_OF_SEARCHES,
        now.elapsed().as_millis()
    );
}

pub fn delete_random_nodes(mut root: &mut Node, generated_ids: &mut Vec<u32>) -> Vec<u32> {
    let mut deleted_nodes: Vec<u32> = Vec::new();
    for id_to_be_deleted in generated_ids {
        println!("Deleting node {id_to_be_deleted}");
        delete_node(&mut root, id_to_be_deleted.clone());
        deleted_nodes.push(id_to_be_deleted.clone());
    }
    deleted_nodes
}
