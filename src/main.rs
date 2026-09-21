const MAX_NODE_SIZE: usize = 24;
const MAX_DATA_INPUT: u32 = 100_00;
const NO_OF_SEARCHES: u32 = MAX_DATA_INPUT * 75 / 100;
fn main() {
    let mut root = Node::new();
    let mut generated_ids: Vec<u32> = Vec::new();

    let mut con = TcpStream::connect("127.0.0.1:2345").unwrap();
    let mut logs: Vec<String> = Vec::new();

    for _ in 1..MAX_DATA_INPUT {
        let now = Instant::now();
        let r = random();
        generated_ids.push(r);
        let r_element = Element {
            id: r,
            name: Name().fake(),
        };
        (root, _) = insert(root, r_element);
        con.write(format!("{},{}", r, now.elapsed().as_nanos()).as_bytes())
            .unwrap();
    }

    fs::write("logs.csv", logs.join("\n")).unwrap();

    let json_string = serde_json::to_string_pretty(&root).unwrap();

    fs::write("out.json", json_string).unwrap();

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

use std::{fs, io::Write, net::TcpStream, time::Instant};

use fake::{Fake, faker::name::en::Name};
use rand::{random, seq::SliceRandom};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    keys: Vec<Element>,
    children: Vec<Node>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Element {
    id: u32,
    name: String,
}

impl Node {
    fn new() -> Self {
        Node {
            keys: Vec::new(),
            children: Vec::new(),
        }
    }
}

fn find(root: &Node, id: u32) {
    if let Some(n) = root.keys.iter().find(|k| k.id == id) {
        // println!("Results: ");
        // println!("{}: {}", id, n.name)
    } else {
        if !root.children.is_empty() {
            if let Some(n) = root.children.get(root.index(id)) {
                find(n, id);
            }
        }
    }
}

fn insert(mut root: Node, v: Element) -> (Node, bool) {
    let i = root.index(v.id);
    let mut split_again = false;
    if root.children.is_empty() {
        //When child overflows, split. We reached the tail of the
        //recxursion
        root.keys.insert(i, v);
        if root.keys.len() > MAX_NODE_SIZE {
            root = split_node(root);
            split_again = true; //Let the caller know that there is a split
        }
    } else {
        let next_node = root.children.remove(i); //Remove the node next in line and
        //take full ownership
        let (mut next_node, split) = insert(next_node, v);
        if split {
            //In case of split, we expect a node with only one key. Deconstruct the node, extract
            //the key and insert in parent
            root.keys.insert(i, next_node.keys.pop().unwrap());
            //Get the grand-children node and insert them in parent relative to the index
            //calculated
            let e_1 = next_node.children.pop().unwrap();
            let e_2 = next_node.children.pop().unwrap();
            root.children.insert(i, e_2);
            root.children.insert(i + 1, e_1);
            if root.keys.len() > MAX_NODE_SIZE {
                //In case of overflow, split again and let the
                //caller know
                root = split_node(root);
                split_again = true;
            }
        } else {
            root.children.insert(i, next_node); //In case there is no split, insert
            //the previously removed node in the same place. Since there is no new nodes, no need
            //to further process the tree.
        }
    }
    (root, split_again)
}

fn split_node(mut root: Node) -> Node {
    let mut left_node = Node::new();
    let mut right_node = Node::new();

    let right_keys = root.keys.split_off(MAX_NODE_SIZE / 2 + 1);
    let parent = root.keys.pop().unwrap();
    let left_keys = root.keys;

    root.keys = vec![parent];
    left_node.keys = left_keys;
    right_node.keys = right_keys;

    if !root.children.is_empty() {
        right_node.children = root.children.split_off(MAX_NODE_SIZE / 2 + 1);
        left_node.children = root.children;
    }

    root.children = vec![left_node, right_node];

    root
}

impl Node {
    fn index(&self, id: u32) -> usize {
        let mut i: usize = 0;
        let mut ir = self.keys.iter();

        while let Some(x) = ir.next()
            && x.id < id
        {
            i += 1;
        }
        i
    }
}
